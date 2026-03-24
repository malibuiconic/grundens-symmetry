// src/pricing/mod.rs
//
// Grundens Pricing Export — web module
//
// Fetches the live Shopify product catalog (with real IDs), merges it against
// NAV18 pricing data, applies the four pricing rules, and writes a
// Matrixify-ready CSV to disk.  Up to MAX_CSVS files are kept; oldest is
// auto-pruned when the limit is reached.
//
// Routes exposed (registered in main.rs):
//   GET  /pricing                      – management UI (HTML)
//   POST /api/pricing/generate         – run export, save CSV
//   GET  /api/pricing/csvs             – list saved CSVs (JSON)
//   GET  /api/pricing/csv/:filename    – download a CSV
//   DELETE /api/pricing/csv/:filename  – delete a CSV

#![allow(unused)]

use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use csv::Writer;
use malibus_shopifyapi_tooling::{
    shopify_graphql_request_with_retries, PageInfo, RateLimiter, ShopifyCreds,
    SHOPIFY_PLUS_GRAPHQL_RATE,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel};
use tokio::net::TcpStream;
use tokio::time::Duration;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use warp::{http::Response, Rejection, Reply};

use crate::sqlite::databaseFunctionality::Database;
use crate::EnvConfig;

// ── Constants ────────────────────────────────────────────────────────────────

const CSV_DIR: &str = "data/pricing_csvs";
const MAX_CSVS: usize = 10;

/// Tags fully controlled by this tool.  They are stripped from every product
/// on each run and re-added only when a pricing rule applies.
const PRICING_TAGS: &[&str] = &["outlet", "outlet-mens", "outlet-womens", "sale"];

// ── Shopify GQL response types (deserialization only) ────────────────────────

#[derive(Debug, Deserialize)]
struct ProductsQueryResp {
    products: ProductsConn,
}

#[derive(Debug, Deserialize)]
struct ProductsConn {
    edges: Vec<ProductEdge>,
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
struct ProductEdge {
    node: GqlProduct,
    cursor: String,
}

#[derive(Debug, Deserialize)]
struct GqlProduct {
    id: String,
    handle: String,
    title: String,
    tags: Vec<String>,
    variants: VariantsConn,
}

#[derive(Debug, Deserialize)]
struct VariantsConn {
    edges: Vec<VariantEdge>,
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
struct VariantEdge {
    node: GqlVariant,
}

#[derive(Debug, Deserialize)]
struct GqlVariant {
    id: String,
    position: u32,
    sku: Option<String>,
    price: String,
    #[serde(rename = "compareAtPrice")]
    compare_at_price: Option<String>,
    #[serde(rename = "inventoryQuantity")]
    inventory_quantity: i64,
    #[serde(rename = "inventoryItem")]
    inventory_item: GqlInventoryItem,
    /// custom.contains_pfas — null if not set on the variant
    metafield: Option<GqlMetafield>,
}

#[derive(Debug, Deserialize)]
struct GqlInventoryItem {
    id: String,
}

#[derive(Debug, Deserialize)]
struct GqlMetafield {
    value: String,
}

// ── Application types ─────────────────────────────────────────────────────────

/// Flat representation of one Shopify variant (product context included).
struct FlatVariant {
    product_id: String,
    product_handle: String,
    product_title: String,
    product_tags: Vec<String>,
    inventory_item_id: String,
    variant_id: String,
    position: u32,
    sku: String,
    shopify_price: String,
    shopify_compare_at: String,
    inventory_qty: i64,
    /// Raw value of the custom.contains_pfas metafield ("true", "false", or "")
    pfas_metafield: String,
}

struct NavItem {
    sku: String,
    unit_price: f64,
    retail_final_date: Option<NaiveDate>,
    item_category_code: String,
}

enum Gender {
    Mens,
    Womens,
    Unknown,
}

// ── Matrixify CSV row — column names must match the template exactly ──────────

#[derive(Serialize)]
struct MatrixifyRow {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Handle")]
    handle: String,
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Tags")]
    tags: String,
    #[serde(rename = "Tags Command")]
    tags_command: String,
    #[serde(rename = "Variant Inventory Item ID")]
    variant_inventory_item_id: String,
    #[serde(rename = "Variant ID")]
    variant_id: String,
    #[serde(rename = "Variant Command")]
    variant_command: String,
    #[serde(rename = "Variant Position")]
    variant_position: String,
    #[serde(rename = "Variant SKU")]
    variant_sku: String,
    #[serde(rename = "Variant Price")]
    variant_price: String,
    #[serde(rename = "Variant Compare At Price")]
    variant_compare_at_price: String,
    #[serde(rename = "Variant Inventory Qty")]
    variant_inventory_qty: String,
    #[serde(rename = "Variant Inventory Adjust")]
    variant_inventory_adjust: String,
    #[serde(rename = "Variant Metafield: custom.contains_pfas [boolean]")]
    pfas_metafield: String,
}

// ── API response types ────────────────────────────────────────────────────────

#[derive(Serialize)]
struct CsvInfo {
    filename: String,
    created_at: String,
    size_bytes: u64,
    row_count: usize,
}

#[derive(Serialize)]
struct PricingSummary {
    filename: String,
    total_variants: usize,
    full_msrp: usize,
    pfas_override: usize,
    sale: usize,
    outlet: usize,
    no_nav_match: usize,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Strip Shopify GID to its numeric tail.
/// `gid://shopify/Product/7617831371001` → `"7617831371001"`
fn gid_to_id(gid: &str) -> String {
    gid.rsplit('/').next().unwrap_or("").to_string()
}

fn detect_gender(category: &str) -> Gender {
    let up = category.to_uppercase();
    if up.contains("WOMEN") || up.contains("WMNS") {
        Gender::Womens
    } else if up.contains("MEN") || up.contains("MENS") {
        Gender::Mens
    } else {
        Gender::Unknown
    }
}

#[inline]
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Compute the final tag string for a Matrixify row.
///
/// - `strip_pricing_tags`: true when we have a NAV match and own the pricing
///   decision; false for unmatched variants (leave tags untouched).
fn compute_tags(existing: &[String], add_pricing: &[&str], strip_pricing_tags: bool) -> String {
    let mut tags: Vec<String> = if strip_pricing_tags {
        existing
            .iter()
            .filter(|t| !PRICING_TAGS.contains(&t.as_str()))
            .cloned()
            .collect()
    } else {
        existing.to_vec()
    };
    for &tag in add_pricing {
        let owned = tag.to_string();
        if !tags.contains(&owned) {
            tags.push(owned);
        }
    }
    tags.join(", ")
}

/// Returns `true` if `name` is a safe CSV filename (no path traversal).
fn is_safe_filename(name: &str) -> bool {
    name.ends_with(".csv")
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        && !name.contains("..")
}

// ── Shopify: paginated product + variant fetch ────────────────────────────────

const PRODUCTS_GQL: &str = r#"
query getProducts($cursor: String) {
    products(first: 50, after: $cursor) {
        edges {
            cursor
            node {
                id
                handle
                title
                tags
                variants(first: 100) {
                    edges {
                        node {
                            id
                            position
                            sku
                            price
                            compareAtPrice
                            inventoryQuantity
                            inventoryItem { id }
                            metafield(namespace: "custom", key: "contains_pfas") {
                                value
                            }
                        }
                    }
                    pageInfo { hasNextPage endCursor }
                }
            }
        }
        pageInfo { hasNextPage endCursor }
    }
}
"#;

async fn fetch_all_variants(
    creds: &ShopifyCreds,
    rate_limiter: &RateLimiter,
) -> Result<Vec<FlatVariant>> {
    let mut all: Vec<FlatVariant> = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let vars = json!({ "cursor": cursor });
        let (resp, _) = shopify_graphql_request_with_retries::<
            ProductsQueryResp,
            serde_json::Value,
        >(
            creds.gql_client_builder(),
            PRODUCTS_GQL,
            Some(vars),
            3,
            rate_limiter,
        )
        .await
        .context("Shopify products GQL failed")?;

        let has_more = resp
            .products
            .page_info
            .has_next_page
            .unwrap_or(false);

        for edge in resp.products.edges {
            let p = edge.node;
            let pid = gid_to_id(&p.id);

            if p.variants
                .page_info
                .has_next_page
                .unwrap_or(false)
            {
                eprintln!(
                    "[pricing] WARNING: product '{}' has >100 variants — only first 100 fetched",
                    p.handle
                );
            }

            for ve in p.variants.edges {
                let v = ve.node;
                let pfas = v
                    .metafield
                    .as_ref()
                    .map(|m| m.value.as_str())
                    .unwrap_or("")
                    .to_string();

                all.push(FlatVariant {
                    product_id: pid.clone(),
                    product_handle: p.handle.clone(),
                    product_title: p.title.clone(),
                    product_tags: p.tags.clone(),
                    inventory_item_id: gid_to_id(&v.inventory_item.id),
                    variant_id: gid_to_id(&v.id),
                    position: v.position,
                    sku: v.sku.unwrap_or_default(),
                    shopify_price: v.price,
                    shopify_compare_at: v.compare_at_price.unwrap_or_default(),
                    inventory_qty: v.inventory_quantity,
                    pfas_metafield: pfas,
                });
            }

            // Update cursor to the last product edge cursor
            cursor = Some(edge.cursor);
        }

        if !has_more {
            break;
        }
    }

    Ok(all)
}

// ── NAV: pricing + RFD fetch ──────────────────────────────────────────────────

const NAV_TIMEOUT_SECS: u64 = 30;

async fn fetch_nav_pricing(cfg: &EnvConfig) -> Result<HashMap<String, NavItem>> {
    tokio::time::timeout(
        tokio::time::Duration::from_secs(NAV_TIMEOUT_SECS),
        fetch_nav_pricing_inner(cfg),
    )
    .await
    .unwrap_or_else(|_| Err(anyhow::anyhow!(
        "NAV18 query timed out after {NAV_TIMEOUT_SECS}s — check the SSH tunnel and VPN"
    )))
}

async fn fetch_nav_pricing_inner(cfg: &EnvConfig) -> Result<HashMap<String, NavItem>> {
    let port = cfg.nav_sql_port.parse::<u16>().unwrap_or(1433);

    let mut tib = Config::new();
    tib.host(&cfg.nav_sql_host);
    tib.port(port);
    tib.database(&cfg.nav_sql_db_nav18);
    tib.authentication(AuthMethod::sql_server(
        &cfg.nav_sql_user,
        &cfg.nav_sql_password,
    ));
    tib.encryption(EncryptionLevel::Required);
    tib.trust_cert();

    let tcp = TcpStream::connect(tib.get_addr())
        .await
        .context("NAV TCP connect failed — is the SSH tunnel running?")?;
    tcp.set_nodelay(true)?;
    let mut client = Client::connect(tib, tcp.compat_write())
        .await
        .context("NAV SQL Server handshake failed")?;

    // NOTE: Verify [Retail Final Date] column name in SSMS:
    //   SELECT TOP 1 * FROM [GRUS$Item]
    // Common alternatives: [RFD Date], [Last Sales Date].
    // Dates are read as varchar(10) YYYY-MM-DD to avoid needing tiberius `chrono` feature.
    let sql = r#"
        SELECT
            [No_]                                               AS sku,
            CAST([Unit Price] AS float)                         AS unit_price,
            CONVERT(varchar(10), [Retail Final Date], 23)       AS retail_final_date,
            ISNULL([Item Category Code], '')                    AS item_category_code
        FROM [GRUS$Item] WITH (NOLOCK)
        WHERE [Blocked] = 0
        ORDER BY [No_]
    "#;

    let stream = client.simple_query(sql).await.context("NAV pricing query failed")?;
    let rows = stream
        .into_first_result()
        .await
        .context("NAV result read failed")?;

    let mut map = HashMap::with_capacity(rows.len());
    for row in &rows {
        let sku = row.get::<&str, _>("sku").unwrap_or("").to_string();
        if sku.is_empty() {
            continue;
        }
        let rfd_str = row.get::<&str, _>("retail_final_date").unwrap_or("");
        let retail_final_date = NaiveDate::parse_from_str(rfd_str, "%Y-%m-%d").ok();

        map.insert(
            sku.clone(),
            NavItem {
                sku,
                unit_price: row.get::<f64, _>("unit_price").unwrap_or(0.0),
                retail_final_date,
                item_category_code: row
                    .get::<&str, _>("item_category_code")
                    .unwrap_or("")
                    .to_string(),
            },
        );
    }

    Ok(map)
}

// ── Pricing logic ─────────────────────────────────────────────────────────────

fn build_matrixify_row(
    variant: &FlatVariant,
    nav_item: Option<&NavItem>,
    today: NaiveDate,
    sale_overrides: &HashMap<String, f64>,
) -> (MatrixifyRow, &'static str) {
    let sku = &variant.sku;

    let (price, compare_at, pricing_tags, rule_label): (String, String, Vec<&str>, &str) =
        match nav_item {
            // ── Variant not in NAV — preserve Shopify price as-is ─────────────
            None => (
                variant.shopify_price.clone(),
                variant.shopify_compare_at.clone(),
                vec![],
                "no_nav_match",
            ),

            Some(item) => {
                let msrp = item.unit_price;

                // Rule 1: PFAS replacement — full MSRP even past RFD
                if variant.pfas_metafield == "true" {
                    (format!("{:.2}", msrp), String::new(), vec![], "pfas")
                }
                // Rule 2: Active sale window override
                else if let Some(&pct) = sale_overrides.get(sku) {
                    let sale_price = round2(msrp * (1.0 - pct / 100.0));
                    (
                        format!("{:.2}", sale_price),
                        format!("{:.2}", msrp),
                        vec!["sale"],
                        "sale",
                    )
                }
                // Rule 3: Past Retail Final Date — 25% markdown + outlet tags
                else if item.retail_final_date.map(|d| today > d).unwrap_or(false) {
                    let outlet_price = round2(msrp * 0.75);
                    let gender_tags: Vec<&str> = match detect_gender(&item.item_category_code) {
                        Gender::Mens => vec!["outlet", "outlet-mens"],
                        Gender::Womens => vec!["outlet", "outlet-womens"],
                        Gender::Unknown => vec!["outlet"],
                    };
                    (
                        format!("{:.2}", outlet_price),
                        format!("{:.2}", msrp),
                        gender_tags,
                        "outlet",
                    )
                }
                // Rule 4: Default — NAV price at full MSRP
                else {
                    (format!("{:.2}", msrp), String::new(), vec![], "full_msrp")
                }
            }
        };

    let strip = nav_item.is_some();
    let tags_str = compute_tags(&variant.product_tags, &pricing_tags, strip);

    let row = MatrixifyRow {
        id:                        variant.product_id.clone(),
        handle:                    variant.product_handle.clone(),
        title:                     variant.product_title.clone(),
        tags:                      tags_str,
        tags_command:              "REPLACE".into(),
        variant_inventory_item_id: variant.inventory_item_id.clone(),
        variant_id:                variant.variant_id.clone(),
        variant_command:           "MERGE".into(),
        variant_position:          variant.position.to_string(),
        variant_sku:               sku.clone(),
        variant_price:             price,
        variant_compare_at_price:  compare_at,
        variant_inventory_qty:     variant.inventory_qty.to_string(),
        variant_inventory_adjust:  "0".into(),
        pfas_metafield:            variant.pfas_metafield.clone(),
    };

    (row, rule_label)
}

// ── Sale overrides file loader ────────────────────────────────────────────────

fn load_sale_overrides() -> HashMap<String, f64> {
    let mut map = HashMap::new();
    for line in fs::read_to_string("sale_overrides.csv")
        .unwrap_or_default()
        .lines()
    {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, ',');
        if let (Some(sku), Some(pct)) = (parts.next(), parts.next()) {
            if let Ok(d) = pct.trim().parse::<f64>() {
                map.insert(sku.trim().to_string(), d);
            }
        }
    }
    map
}

// ── CSV file management ───────────────────────────────────────────────────────

fn ensure_csv_dir() {
    fs::create_dir_all(CSV_DIR).ok();
}

/// Deletes the oldest CSV(s) until the count is below MAX_CSVS.
fn prune_old_csvs() {
    let Ok(mut entries) = fs::read_dir(CSV_DIR) else { return };
    let mut files: Vec<(std::time::SystemTime, PathBuf)> = fs::read_dir(CSV_DIR)
        .unwrap()
        .filter_map(|e| {
            let e = e.ok()?;
            let path = e.path();
            if path.extension()?.to_str()? != "csv" { return None; }
            let mtime = e.metadata().ok()?.modified().ok()?;
            Some((mtime, path))
        })
        .collect();

    if files.len() < MAX_CSVS {
        return;
    }

    // Sort oldest-first
    files.sort_by_key(|(t, _)| *t);
    let to_delete = files.len() + 1 - MAX_CSVS; // +1 because we're about to add one
    for (_, path) in files.iter().take(to_delete) {
        fs::remove_file(path).ok();
    }
}

fn list_csv_files() -> Vec<CsvInfo> {
    let Ok(dir) = fs::read_dir(CSV_DIR) else { return vec![] };
    let mut infos: Vec<CsvInfo> = dir
        .filter_map(|e| {
            let e = e.ok()?;
            let path = e.path();
            if path.extension()?.to_str()? != "csv" { return None; }
            let meta = e.metadata().ok()?;
            let size_bytes = meta.len();
            let mtime = meta.modified().ok()?;
            let filename = path.file_name()?.to_str()?.to_string();

            // Count data rows (total lines minus header line)
            let content = fs::read_to_string(&path).unwrap_or_default();
            let row_count = content.lines().count().saturating_sub(1);

            // Convert mtime to ISO 8601
            let secs = mtime
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();
            let dt = chrono::DateTime::from_timestamp(secs as i64, 0)?;
            let created_at = dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();

            Some(CsvInfo { filename, created_at, size_bytes, row_count })
        })
        .collect();

    // Newest first
    infos.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    infos
}

// ── Main orchestrator ─────────────────────────────────────────────────────────

async fn run_pricing_export(
    cfg: &EnvConfig,
    db: Arc<Database>,
) -> Result<PricingSummary> {
    ensure_csv_dir();

    // ── Resolve Shopify access token ─────────────────────────────────────────
    // Prefer the OAuth token stored in SQLite (from the app install), fall back
    // to the static env token.
    let shop_domain = &cfg.shopify_shop_name;
    let token = match db.get_store(shop_domain).await {
        Ok(Some(store)) if !store.access_token.is_empty() => store.access_token,
        _ => cfg.shopify_app_client_password.clone(),
    };

    let creds = ShopifyCreds {
        shopify_token:           token,
        shopify_api_version:     cfg.shopify_api_version.clone(),
        myshopify_url:           format!("https://{}", cfg.shopify_shop_name),
        storefront_access_token: String::new(),
    };

    let rate_limiter = RateLimiter::new(
        SHOPIFY_PLUS_GRAPHQL_RATE,
        Duration::from_secs(1),
    );

    // ── Fetch from both sources in parallel ──────────────────────────────────
    eprintln!("[pricing] Fetching Shopify catalog...");
    let shopify_variants = fetch_all_variants(&creds, &rate_limiter).await?;
    eprintln!("[pricing] {} variants from Shopify", shopify_variants.len());

    eprintln!("[pricing] Fetching NAV18 pricing...");
    let nav_map = fetch_nav_pricing(cfg).await?;
    eprintln!("[pricing] {} items from NAV18", nav_map.len());

    // ── Load sale overrides ──────────────────────────────────────────────────
    let sale_overrides = load_sale_overrides();
    let today = Utc::now().date_naive();

    // ── Apply pricing rules ──────────────────────────────────────────────────
    let mut rows: Vec<MatrixifyRow> = Vec::with_capacity(shopify_variants.len());
    let mut counts: HashMap<&str, usize> = HashMap::new();

    for variant in &shopify_variants {
        let nav_item = if variant.sku.is_empty() {
            None
        } else {
            nav_map.get(&variant.sku)
        };

        let (row, label) = build_matrixify_row(variant, nav_item, today, &sale_overrides);
        *counts.entry(label).or_insert(0) += 1;
        rows.push(row);
    }

    // ── Write CSV ────────────────────────────────────────────────────────────
    prune_old_csvs();

    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("pricing_{timestamp}.csv");
    let path = Path::new(CSV_DIR).join(&filename);

    let mut wtr = Writer::from_path(&path)
        .with_context(|| format!("Cannot create {}", path.display()))?;
    for row in &rows {
        wtr.serialize(row)?;
    }
    wtr.flush()?;

    eprintln!("[pricing] Written → {}", path.display());

    Ok(PricingSummary {
        filename,
        total_variants: rows.len(),
        full_msrp:    *counts.get("full_msrp").unwrap_or(&0),
        pfas_override: *counts.get("pfas").unwrap_or(&0),
        sale:          *counts.get("sale").unwrap_or(&0),
        outlet:        *counts.get("outlet").unwrap_or(&0),
        no_nav_match:  *counts.get("no_nav_match").unwrap_or(&0),
    })
}

// ── Route handlers ────────────────────────────────────────────────────────────

/// POST /api/pricing/generate
pub async fn handle_generate(
    app_env: EnvConfig,
    db: Arc<Database>,
) -> Result<impl Reply, Rejection> {
    match run_pricing_export(&app_env, db).await {
        Ok(summary) => Ok(warp::reply::with_status(
            warp::reply::json(&summary),
            warp::http::StatusCode::OK,
        )),
        Err(e) => {
            eprintln!("[pricing] generate error: {e}");
            let body = serde_json::json!({ "error": e.to_string() });
            Ok(warp::reply::with_status(
                warp::reply::json(&body),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// GET /api/pricing/csvs
pub async fn handle_list_csvs() -> Result<impl Reply, Rejection> {
    ensure_csv_dir();
    Ok(warp::reply::json(&list_csv_files()))
}

/// GET /api/pricing/csv/:filename
pub async fn handle_download_csv(filename: String) -> Result<impl Reply, Rejection> {
    if !is_safe_filename(&filename) {
        return Err(warp::reject::not_found());
    }
    let path = Path::new(CSV_DIR).join(&filename);
    match fs::read_to_string(&path) {
        Ok(content) => {
            let response = Response::builder()
                .status(200)
                .header("Content-Type", "text/csv; charset=utf-8")
                .header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{filename}\""),
                )
                .body(content)
                .unwrap();
            Ok(response)
        }
        Err(_) => Err(warp::reject::not_found()),
    }
}

/// DELETE /api/pricing/csv/:filename
pub async fn handle_delete_csv(filename: String) -> Result<impl Reply, Rejection> {
    if !is_safe_filename(&filename) {
        return Err(warp::reject::not_found());
    }
    let path = Path::new(CSV_DIR).join(&filename);
    match fs::remove_file(&path) {
        Ok(_) => Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({ "deleted": filename })),
            warp::http::StatusCode::OK,
        )),
        Err(_) => Err(warp::reject::not_found()),
    }
}

/// GET /pricing — serves the HTML management UI
pub async fn handle_pricing_ui() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_header(
        PRICING_UI_HTML,
        "Content-Type",
        "text/html; charset=utf-8",
    ))
}

// ── HTML UI ───────────────────────────────────────────────────────────────────

const PRICING_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Grundens — Pricing Export</title>
<style>
  :root {
    --bg:       #0a0a1a;
    --surface:  #0f0f22;
    --surface2: #14142e;
    --border:   rgba(0,255,231,0.25);
    --accent:   #00ffe7;
    --accent2:  #00b8a8;
    --text:     #dde1e7;
    --text-dim: #6b7280;
    --danger:   #ff5555;
    --success:  #50fa7b;
    --warn:     #ffb86c;
  }
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    background: var(--bg);
    color: var(--text);
    font-family: 'Segoe UI', system-ui, sans-serif;
    font-size: 14px;
    min-height: 100vh;
    padding: 32px 24px;
  }
  h1 {
    font-size: 22px;
    font-weight: 600;
    color: var(--accent);
    letter-spacing: 0.03em;
    margin-bottom: 4px;
  }
  .subtitle {
    color: var(--text-dim);
    font-size: 12px;
    margin-bottom: 28px;
  }
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px 24px;
    margin-bottom: 24px;
  }
  .card-title {
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--accent2);
    margin-bottom: 14px;
  }
  /* Generate button */
  #btn-generate {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    background: var(--accent);
    color: #0a0a1a;
    font-weight: 700;
    font-size: 13px;
    border: none;
    border-radius: 6px;
    padding: 10px 20px;
    cursor: pointer;
    transition: background 0.15s, opacity 0.15s;
  }
  #btn-generate:hover:not(:disabled) { background: #33ffef; }
  #btn-generate:disabled { opacity: 0.5; cursor: not-allowed; }
  .spinner {
    display: none;
    width: 14px; height: 14px;
    border: 2px solid #0a0a1a;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  /* Status bar */
  #status {
    margin-top: 12px;
    font-size: 12px;
    min-height: 18px;
    transition: color 0.2s;
  }
  #status.ok   { color: var(--success); }
  #status.err  { color: var(--danger); }
  #status.info { color: var(--accent2); }
  /* Summary pills */
  .pills { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 10px; }
  .pill {
    background: var(--surface2);
    border: 1px solid var(--border);
    border-radius: 20px;
    padding: 3px 12px;
    font-size: 11px;
    color: var(--text-dim);
  }
  .pill span { color: var(--text); font-weight: 600; }
  /* CSV table */
  .table-wrap { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 13px; }
  th {
    text-align: left;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-dim);
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
  }
  td {
    padding: 10px 12px;
    border-bottom: 1px solid rgba(0,255,231,0.08);
    vertical-align: middle;
  }
  tr:last-child td { border-bottom: none; }
  tr:hover td { background: var(--surface2); }
  .filename {
    font-family: 'Cascadia Code', 'Fira Code', monospace;
    font-size: 12px;
    color: var(--accent);
  }
  .size  { color: var(--text-dim); }
  .rows  { color: var(--text-dim); }
  .ts    { color: var(--text-dim); font-size: 12px; }
  .actions { display: flex; gap: 8px; }
  .btn-dl {
    display: inline-flex; align-items: center; gap: 5px;
    padding: 5px 12px; border-radius: 5px; font-size: 12px; font-weight: 600;
    background: rgba(0,255,231,0.1); color: var(--accent);
    border: 1px solid rgba(0,255,231,0.3); cursor: pointer;
    text-decoration: none; transition: background 0.15s;
  }
  .btn-dl:hover { background: rgba(0,255,231,0.2); }
  .btn-del {
    display: inline-flex; align-items: center; gap: 5px;
    padding: 5px 12px; border-radius: 5px; font-size: 12px; font-weight: 600;
    background: rgba(255,85,85,0.1); color: var(--danger);
    border: 1px solid rgba(255,85,85,0.3); cursor: pointer;
    transition: background 0.15s;
  }
  .btn-del:hover { background: rgba(255,85,85,0.2); }
  .empty-state {
    text-align: center;
    padding: 32px;
    color: var(--text-dim);
    font-size: 13px;
  }
  .max-note {
    font-size: 11px;
    color: var(--text-dim);
    margin-top: 10px;
  }
  .badge-count {
    display: inline-block;
    background: var(--surface2);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 1px 8px;
    font-size: 11px;
    color: var(--accent2);
    margin-left: 8px;
    vertical-align: middle;
  }
</style>
</head>
<body>

<h1>Pricing Export</h1>
<p class="subtitle">Grundens · Shopify + NAV18 · Matrixify CSV Generator</p>

<!-- Generate card -->
<div class="card">
  <div class="card-title">Generate New CSV</div>
  <p style="color:var(--text-dim);font-size:12px;margin-bottom:14px;line-height:1.6">
    Fetches all products &amp; variants from Shopify, applies NAV18 pricing rules
    (PFAS override → sale window → RFD markdown → full MSRP), and saves a
    Matrixify-ready CSV.  Oldest file is auto-deleted when the 10-file limit is reached.
  </p>
  <button id="btn-generate" onclick="generate()">
    <div class="spinner" id="spinner"></div>
    <span id="btn-label">Generate Price Update CSV</span>
  </button>
  <div id="status"></div>
  <div class="pills" id="pills"></div>
</div>

<!-- CSV directory card -->
<div class="card">
  <div class="card-title">
    Saved CSVs <span class="badge-count" id="csv-count">–</span>
    <span style="font-size:10px;color:var(--text-dim);font-weight:400;margin-left:8px">max 10</span>
  </div>
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Filename</th>
          <th>Generated</th>
          <th>Rows</th>
          <th>Size</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody id="csv-tbody">
        <tr><td colspan="5" class="empty-state">Loading…</td></tr>
      </tbody>
    </table>
  </div>
</div>

<script>
// ── helpers ──────────────────────────────────────────────────────────────────
function fmtSize(bytes) {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024*1024) return (bytes/1024).toFixed(1) + ' KB';
  return (bytes/(1024*1024)).toFixed(2) + ' MB';
}
function fmtDate(iso) {
  const d = new Date(iso);
  return d.toLocaleString(undefined, {
    year:'numeric', month:'short', day:'2-digit',
    hour:'2-digit', minute:'2-digit'
  });
}
function setStatus(msg, type) {
  const el = document.getElementById('status');
  el.textContent = msg;
  el.className = type;
}

// ── load CSV list ─────────────────────────────────────────────────────────────
async function loadCsvs() {
  const tbody = document.getElementById('csv-tbody');
  try {
    const r = await fetch('/api/pricing/csvs');
    const list = await r.json();
    document.getElementById('csv-count').textContent = list.length + ' / 10';

    if (list.length === 0) {
      tbody.innerHTML = '<tr><td colspan="5" class="empty-state">No CSVs yet. Click Generate to create one.</td></tr>';
      return;
    }

    tbody.innerHTML = list.map(f => `
      <tr>
        <td><span class="filename">${f.filename}</span></td>
        <td class="ts">${fmtDate(f.created_at)}</td>
        <td class="rows">${f.row_count.toLocaleString()}</td>
        <td class="size">${fmtSize(f.size_bytes)}</td>
        <td class="actions">
          <a class="btn-dl" href="/api/pricing/csv/${f.filename}" download="${f.filename}">
            ↓ Download
          </a>
          <button class="btn-del" onclick="deleteCsv('${f.filename}')">
            ✕ Delete
          </button>
        </td>
      </tr>
    `).join('');
  } catch(e) {
    tbody.innerHTML = '<tr><td colspan="5" class="empty-state" style="color:var(--danger)">Failed to load CSV list.</td></tr>';
  }
}

// ── generate ──────────────────────────────────────────────────────────────────
async function generate() {
  const btn   = document.getElementById('btn-generate');
  const spin  = document.getElementById('spinner');
  const label = document.getElementById('btn-label');
  const pills = document.getElementById('pills');

  btn.disabled = true;
  spin.style.display = 'block';
  label.textContent = 'Generating…';
  setStatus('Fetching Shopify catalog and NAV18 pricing — this may take 20–60 seconds…', 'info');
  pills.innerHTML = '';

  try {
    const r   = await fetch('/api/pricing/generate', { method: 'POST' });
    const res = await r.json();

    if (!r.ok) {
      setStatus('Error: ' + (res.error || 'Unknown error'), 'err');
      return;
    }

    setStatus('✓ ' + res.filename + ' created successfully', 'ok');
    pills.innerHTML = [
      ['Total', res.total_variants],
      ['Full MSRP', res.full_msrp],
      ['PFAS', res.pfas_override],
      ['Sale', res.sale],
      ['Outlet -25%', res.outlet],
      ['No NAV match', res.no_nav_match],
    ].map(([k,v]) => `<div class="pill">${k}: <span>${v.toLocaleString()}</span></div>`).join('');

    await loadCsvs();
  } catch(e) {
    setStatus('Network error — is the server reachable?', 'err');
  } finally {
    btn.disabled = false;
    spin.style.display = 'none';
    label.textContent = 'Generate Price Update CSV';
  }
}

// ── delete ────────────────────────────────────────────────────────────────────
async function deleteCsv(filename) {
  if (!confirm('Delete ' + filename + '?')) return;
  try {
    const r = await fetch('/api/pricing/csv/' + filename, { method: 'DELETE' });
    if (r.ok) {
      await loadCsvs();
    } else {
      alert('Delete failed');
    }
  } catch(e) {
    alert('Network error');
  }
}

// ── init ──────────────────────────────────────────────────────────────────────
loadCsvs();
</script>
</body>
</html>"#;
