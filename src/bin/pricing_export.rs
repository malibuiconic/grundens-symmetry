//! Grundens Pricing Export — Matrixify CSV Generator
//!
//! Queries NAV18 for all active items, applies pricing rules, and writes a
//! Matrixify-compatible Products CSV ready for import into the Grundens Shopify store.
//!
//! # Pricing rules (applied in priority order)
//!
//! 1. **PFAS replacement SKUs**   → full MSRP, no outlet treatment (override RFD)
//! 2. **Active sale window**      → discounted price + compare-at MSRP  (from sale_overrides.csv)
//! 3. **Past Retail Final Date**  → 25% markdown, compare-at MSRP, + outlet/gendered tags
//! 4. **Default**                 → NAV unit price, no compare-at
//!
//! # Override files (optional — relative to your working directory)
//!
//! - `pfas_skus.txt`      — one SKU per line; keeps item at full MSRP even if past RFD
//! - `sale_overrides.csv` — two columns: `sku,discount_pct`  (e.g. `10040-800-0020,20`)
//!   Lines starting with `#` are ignored in both files.
//!
//! # Usage
//!
//! ```bash
//! # Output to auto-named file (pricing_export_YYYY-MM-DD.csv)
//! cargo run --bin pricing_export
//!
//! # Specify output path
//! cargo run --bin pricing_export -- --output /tmp/may_pricing.csv
//!
//! # Print CSV to stdout (useful for piping / inspection)
//! cargo run --bin pricing_export -- --stdout
//! ```
//!
//! # Matrixify import notes
//!
//! The CSV uses `Variant SKU` as the match key — configure your Matrixify import
//! to merge/update by Variant SKU.  The `Handle` column is intentionally left
//! blank; Matrixify will resolve the product from the SKU match.
//! Tags in the output are *additive* — set Matrixify to "Merge" tags, not replace,
//! unless you want to strip existing tags on non-outlet items.

use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use csv::Writer;
use dotenv::dotenv;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

// ── Env / connection config ──────────────────────────────────────────────────

struct NavCfg {
    host:     String,
    port:     u16,
    user:     String,
    password: String,
    db_nav18: String,
}

impl NavCfg {
    fn from_env() -> Result<Self> {
        dotenv().ok();
        Ok(NavCfg {
            host:     std::env::var("NAV_SQL_HOST").context("NAV_SQL_HOST not set")?,
            port:     std::env::var("NAV_SQL_PORT")
                        .unwrap_or_else(|_| "1433".into())
                        .parse()
                        .context("NAV_SQL_PORT is not a valid port number")?,
            user:     std::env::var("NAV_SQL_USER").context("NAV_SQL_USER not set")?,
            password: std::env::var("NAV_SQL_PASSWORD").context("NAV_SQL_PASSWORD not set")?,
            db_nav18: std::env::var("NAV_SQL_DB_NAV18").context("NAV_SQL_DB_NAV18 not set")?,
        })
    }
}

// ── Raw NAV row ──────────────────────────────────────────────────────────────

struct NavItem {
    sku:                String,
    description:        String,
    unit_price:         f64,
    retail_final_date:  Option<NaiveDate>,
    item_category_code: String,
}

// ── Gender detection (outlet tagging) ───────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum Gender { Mens, Womens, Unknown }

/// Infers gender from the item category code.
/// Grundens codes like "MEN-FISHING", "WMNS-DECK", etc. — adjust patterns as needed.
fn detect_gender(category: &str) -> Gender {
    let upper = category.to_uppercase();
    if upper.contains("WOMEN") || upper.contains("WMNS") || upper.contains("WOM") {
        Gender::Womens
    } else if upper.contains("MEN") || upper.contains("MENS") {
        Gender::Mens
    } else {
        Gender::Unknown
    }
}

// ── Pricing logic ────────────────────────────────────────────────────────────

struct PricedItem {
    sku:              String,
    description:      String,
    price:            f64,
    compare_at_price: Option<f64>,
    tags:             Vec<String>,
}

fn apply_pricing_rules(
    item:           &NavItem,
    today:          NaiveDate,
    pfas_skus:      &HashSet<String>,
    sale_overrides: &HashMap<String, f64>,
) -> PricedItem {
    let msrp = item.unit_price;

    // Rule 1 — PFAS replacement: stay at full MSRP regardless of RFD
    if pfas_skus.contains(&item.sku) {
        return PricedItem {
            sku:              item.sku.clone(),
            description:      item.description.clone(),
            price:            msrp,
            compare_at_price: None,
            tags:             vec![],
        };
    }

    // Rule 2 — Active sale override
    if let Some(&discount_pct) = sale_overrides.get(&item.sku) {
        let sale_price = round2(msrp * (1.0 - discount_pct / 100.0));
        return PricedItem {
            sku:              item.sku.clone(),
            description:      item.description.clone(),
            price:            sale_price,
            compare_at_price: Some(msrp),
            tags:             vec!["sale".into()],
        };
    }

    // Rule 3 — Past Retail Final Date: 25% markdown + outlet tags
    let past_rfd = item.retail_final_date.map(|d| today > d).unwrap_or(false);
    if past_rfd {
        let outlet_price = round2(msrp * 0.75);
        let mut tags = vec!["outlet".to_string()];
        match detect_gender(&item.item_category_code) {
            Gender::Mens   => tags.push("outlet-mens".into()),
            Gender::Womens => tags.push("outlet-womens".into()),
            Gender::Unknown => {}
        }
        return PricedItem {
            sku:              item.sku.clone(),
            description:      item.description.clone(),
            price:            outlet_price,
            compare_at_price: Some(msrp),
            tags,
        };
    }

    // Rule 4 — Default: NAV price as-is
    PricedItem {
        sku:              item.sku.clone(),
        description:      item.description.clone(),
        price:            msrp,
        compare_at_price: None,
        tags:             vec![],
    }
}

#[inline]
fn round2(v: f64) -> f64 { (v * 100.0).round() / 100.0 }

// ── Matrixify CSV row ────────────────────────────────────────────────────────

/// Matches the Matrixify Products CSV column headers exactly.
/// Configure your Matrixify import to "Merge" by `Variant SKU`.
#[derive(Serialize)]
struct MatrixifyRow {
    /// Leave blank — Matrixify resolves the product via Variant SKU match.
    #[serde(rename = "Handle")]
    handle: String,

    #[serde(rename = "Title")]
    title: String,

    #[serde(rename = "Variant SKU")]
    variant_sku: String,

    /// Calculated price after applying pricing rules.
    #[serde(rename = "Variant Price")]
    variant_price: String,

    /// Original MSRP shown as crossed-out price. Empty = no compare-at.
    #[serde(rename = "Variant Compare At Price")]
    variant_compare_at_price: String,

    /// Comma-separated tags to add (e.g. "outlet, outlet-mens").
    #[serde(rename = "Tags")]
    tags: String,
}

impl From<PricedItem> for MatrixifyRow {
    fn from(p: PricedItem) -> Self {
        MatrixifyRow {
            handle:                   String::new(),
            title:                    p.description,
            variant_sku:              p.sku,
            variant_price:            format!("{:.2}", p.price),
            variant_compare_at_price: p.compare_at_price
                                        .map(|v| format!("{:.2}", v))
                                        .unwrap_or_default(),
            tags:                     p.tags.join(", "),
        }
    }
}

// ── Override file loaders ────────────────────────────────────────────────────

fn load_pfas_skus(path: &str) -> HashSet<String> {
    let content = fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}

fn load_sale_overrides(path: &str) -> HashMap<String, f64> {
    let mut map = HashMap::new();
    for line in fs::read_to_string(path).unwrap_or_default().lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let mut parts = line.splitn(2, ',');
        if let (Some(sku), Some(pct)) = (parts.next(), parts.next()) {
            if let Ok(discount) = pct.trim().parse::<f64>() {
                map.insert(sku.trim().to_string(), discount);
            }
        }
    }
    map
}

// ── CLI arg helpers ──────────────────────────────────────────────────────────

struct Args {
    stdout: bool,
    output: String,
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    let stdout = args.iter().any(|a| a == "--stdout");
    let output = args.windows(2)
        .find(|w| w[0] == "--output")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| format!("pricing_export_{}.csv", Utc::now().format("%Y-%m-%d")));
    Args { stdout, output }
}

// ── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args();

    // Load optional override files
    let pfas_skus = load_pfas_skus("pfas_skus.txt");
    let sale_overrides = load_sale_overrides("sale_overrides.csv");
    let today = Utc::now().date_naive();

    eprintln!("[pricing_export] PFAS SKUs loaded:   {}", pfas_skus.len());
    eprintln!("[pricing_export] Sale overrides:      {}", sale_overrides.len());
    eprintln!("[pricing_export] Pricing date (today): {today}");

    // ── NAV connection ───────────────────────────────────────────────────────
    let cfg = NavCfg::from_env()?;

    let mut tib_cfg = Config::new();
    tib_cfg.host(&cfg.host);
    tib_cfg.port(cfg.port);
    tib_cfg.database(&cfg.db_nav18);
    tib_cfg.authentication(AuthMethod::sql_server(&cfg.user, &cfg.password));
    tib_cfg.encryption(EncryptionLevel::Required);
    tib_cfg.trust_cert();

    eprintln!("[pricing_export] Connecting to NAV18 ({})…", cfg.db_nav18);
    let tcp = TcpStream::connect(tib_cfg.get_addr()).await
        .context("TCP connect failed — is the SSH tunnel running? (ssh -N -L 1433:…)")?;
    tcp.set_nodelay(true)?;
    let mut client = Client::connect(tib_cfg, tcp.compat_write()).await
        .context("SQL Server handshake failed — check credentials and VPN")?;

    eprintln!("[pricing_export] Connected. Querying items…");

    // ── NAV query ────────────────────────────────────────────────────────────
    //
    // IMPORTANT — verify these column names before running in production:
    //
    //   [Retail Final Date]   The date after which an item is discontinued.
    //                         Run: SELECT TOP 1 * FROM [GRUS$Item]
    //                         in SSMS to confirm the exact column name.
    //                         Common alternatives: [RFD Date], [Last Date Modified].
    //
    //   [Item Category Code]  Used to infer gender for outlet tag routing.
    //                         Patterns matched: MEN/MENS → mens, WOMEN/WMNS → womens.
    //                         Update detect_gender() above if Grundens codes differ.
    //
    // Dates are read as varchar(10) YYYY-MM-DD strings to avoid needing the
    // tiberius `chrono` feature flag.
    let sql = r#"
        SELECT
            [No_]                                               AS sku,
            [Description]                                       AS description,
            CAST([Unit Price] AS float)                         AS unit_price,
            CONVERT(varchar(10), [Retail Final Date], 23)       AS retail_final_date,
            ISNULL([Item Category Code], '')                    AS item_category_code
        FROM [GRUS$Item]
        WHERE [Blocked] = 0
        ORDER BY [No_]
    "#;

    let stream = client.simple_query(sql).await
        .context("NAV query failed — check column names (see IMPORTANT comment in source)")?;
    let rows = stream.into_first_result().await
        .context("Failed to read NAV result set")?;

    eprintln!("[pricing_export] Fetched {} items from NAV.", rows.len());

    // ── Parse rows ───────────────────────────────────────────────────────────
    let items: Vec<NavItem> = rows.iter().map(|row| {
        let rfd_str: &str = row.get::<&str, _>("retail_final_date").unwrap_or("");
        let retail_final_date = NaiveDate::parse_from_str(rfd_str, "%Y-%m-%d").ok();

        NavItem {
            sku:                row.get::<&str, _>("sku").unwrap_or("").to_string(),
            description:        row.get::<&str, _>("description").unwrap_or("").to_string(),
            unit_price:         row.get::<f64, _>("unit_price").unwrap_or(0.0),
            retail_final_date,
            item_category_code: row.get::<&str, _>("item_category_code").unwrap_or("").to_string(),
        }
    }).collect();

    // ── Apply pricing rules ──────────────────────────────────────────────────
    let mut count_outlet   = 0usize;
    let mut count_sale     = 0usize;
    let mut count_pfas     = 0usize;
    let mut count_full     = 0usize;

    let csv_rows: Vec<MatrixifyRow> = items.iter().map(|item| {
        let priced = apply_pricing_rules(item, today, &pfas_skus, &sale_overrides);

        // Tally for the summary
        if priced.tags.iter().any(|t| t == "outlet") { count_outlet += 1; }
        else if priced.tags.iter().any(|t| t == "sale") { count_sale += 1; }
        else if pfas_skus.contains(&item.sku) { count_pfas += 1; }
        else { count_full += 1; }

        MatrixifyRow::from(priced)
    }).collect();

    eprintln!("[pricing_export] Pricing breakdown:");
    eprintln!("  Full MSRP (default):  {count_full}");
    eprintln!("  PFAS replacement:     {count_pfas}");
    eprintln!("  Sale window:          {count_sale}");
    eprintln!("  Outlet (RFD -25%):    {count_outlet}");

    // ── Write CSV ────────────────────────────────────────────────────────────
    if args.stdout {
        let mut wtr = Writer::from_writer(std::io::stdout());
        for row in csv_rows { wtr.serialize(row)?; }
        wtr.flush()?;
    } else {
        let mut wtr = Writer::from_path(&args.output)
            .with_context(|| format!("Cannot create output file: {}", args.output))?;
        for row in csv_rows { wtr.serialize(row)?; }
        wtr.flush()?;
        eprintln!("[pricing_export] Written → {}", args.output);
    }

    Ok(())
}
