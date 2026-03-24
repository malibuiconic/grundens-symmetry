use anyhow::Result;
use serde::Serialize;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use warp::{Rejection, Reply};
use crate::EnvConfig;

// ── Types ─────────────────────────────────────────────────────────────────────

type NavClient = Client<Compat<TcpStream>>;

#[derive(Serialize)]
pub struct DbConnectionStatus {
    pub database: String,
    pub connected: bool,
    pub message: String,
    pub server_version: Option<String>,
}

#[derive(Serialize)]
pub struct NavStatusResponse {
    pub nav18: DbConnectionStatus,
    pub integration: DbConnectionStatus,
}

#[derive(Serialize)]
pub struct NavItem {
    pub sku: String,
    pub description: String,
    pub unit_price: f64,
}

#[derive(Serialize)]
pub struct NavInventoryRow {
    pub sku: String,
    pub qty_on_hand: i64,
}

// ── Config builder ────────────────────────────────────────────────────────────

struct NavEnv {
    host: String,
    port: u16,
    user: String,
    password: String,
}

impl NavEnv {
    fn from_config(cfg: &EnvConfig) -> Self {
        let port = cfg.nav_sql_port.parse::<u16>().unwrap_or(1433);
        NavEnv {
            host:     cfg.nav_sql_host.clone(),
            port,
            user:     cfg.nav_sql_user.clone(),
            password: cfg.nav_sql_password.clone(),
        }
    }
}

fn build_config(env: &NavEnv, database: &str) -> Config {
    let mut cfg = Config::new();
    cfg.host(&env.host);
    cfg.port(env.port);
    cfg.database(database);
    cfg.authentication(AuthMethod::sql_server(&env.user, &env.password));
    cfg.encryption(EncryptionLevel::Required);
    cfg.trust_cert();
    cfg
}

// ── Connection helper ─────────────────────────────────────────────────────────

async fn make_client(env: &NavEnv, database: &str) -> Result<NavClient> {
    let config = build_config(env, database);
    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;
    let client = Client::connect(config, tcp.compat_write()).await?;
    Ok(client)
}

async fn probe(env: &NavEnv, database: &str) -> DbConnectionStatus {
    match make_client(env, database).await {
        Ok(mut client) => match client.simple_query("SELECT @@VERSION").await {
            Ok(stream) => {
                let version = stream
                    .into_row()
                    .await
                    .ok()
                    .flatten()
                    .and_then(|row| row.get::<&str, _>(0).map(|s| s.to_string()));

                DbConnectionStatus {
                    database: database.to_string(),
                    connected: true,
                    message: "Connection successful".to_string(),
                    server_version: version,
                }
            }
            Err(e) => DbConnectionStatus {
                database: database.to_string(),
                connected: false,
                message: format!("Query failed: {e}"),
                server_version: None,
            },
        },
        Err(e) => DbConnectionStatus {
            database: database.to_string(),
            connected: false,
            message: format!("Connection failed: {e}"),
            server_version: None,
        },
    }
}

// ── Startup check ─────────────────────────────────────────────────────────────

pub async fn check_nav_connections(cfg: &EnvConfig) {
    let env = NavEnv::from_config(cfg);
    let (nav18, integration) = tokio::join!(
        probe(&env, &cfg.nav_sql_db_nav18),
        probe(&env, &cfg.nav_sql_db_integration),
    );
    if nav18.connected {
        info!("NAV18 DB connected ({})", nav18.database);
    } else {
        error!("NAV18 DB connection failed: {}", nav18.message);
    }
    if integration.connected {
        info!("Integration DB connected ({})", integration.database);
    } else {
        error!("Integration DB connection failed: {}", integration.message);
    }
}

// ── Route handlers ─────────────────────────────────────────────────────────────

/// GET /api/nav/status
/// Tests connectivity to both NAV18 and Integration DB in parallel.
/// Requires Cisco Secure Connect VPN active.
pub async fn handle_nav_status(app_env: EnvConfig) -> Result<impl Reply, Rejection> {
    let env = NavEnv::from_config(&app_env);

    let (nav18, integration) = tokio::join!(
        probe(&env, &app_env.nav_sql_db_nav18),
        probe(&env, &app_env.nav_sql_db_integration),
    );

    Ok(warp::reply::json(&NavStatusResponse { nav18, integration }))
}

/// GET /api/nav/items
/// Returns first 20 active items from [GRUS$Item] — SKU, description, price.
pub async fn handle_nav_items(app_env: EnvConfig) -> Result<impl Reply, Rejection> {
    let env = NavEnv::from_config(&app_env);
    let db  = &app_env.nav_sql_db_nav18;

    let mut client = match make_client(&env, db).await {
        Ok(c) => c,
        Err(e) => return Ok(warp::reply::json(&serde_json::json!({ "error": e.to_string() }))),
    };

    let sql = "
        SELECT TOP 20
            [No_]                        AS sku,
            [Description]                AS description,
            CAST([Unit Price] AS float)  AS unit_price
        FROM [GRUS$Item] WITH (NOLOCK)
        WHERE [Blocked] = 0
        ORDER BY [No_]
    ";

    match client.simple_query(sql).await {
        Ok(stream) => {
            let rows = stream.into_first_result().await.unwrap_or_default();
            let items: Vec<NavItem> = rows.iter().map(|row| NavItem {
                sku:         row.get::<&str, _>("sku").unwrap_or("").to_string(),
                description: row.get::<&str, _>("description").unwrap_or("").to_string(),
                unit_price:  row.get::<f64, _>("unit_price").unwrap_or(0.0),
            }).collect();
            Ok(warp::reply::json(&items))
        }
        Err(e) => Ok(warp::reply::json(&serde_json::json!({ "error": e.to_string() }))),
    }
}

/// GET /api/nav/inventory
/// Returns on-hand qty per SKU at TAC (US domestic warehouse).
pub async fn handle_nav_inventory(app_env: EnvConfig) -> Result<impl Reply, Rejection> {
    let env = NavEnv::from_config(&app_env);
    let db  = &app_env.nav_sql_db_nav18;

    let mut client = match make_client(&env, db).await {
        Ok(c) => c,
        Err(e) => return Ok(warp::reply::json(&serde_json::json!({ "error": e.to_string() }))),
    };

    let sql = "
        SELECT TOP 20
            i.[No_]                                         AS sku,
            CAST(ISNULL(SUM(ile.[Quantity]), 0) AS bigint)  AS qty_on_hand
        FROM [GRUS$Item] i WITH (NOLOCK)
        LEFT JOIN [GRUS$Item Ledger Entry] ile WITH (NOLOCK)
            ON  ile.[Item No_]      = i.[No_]
            AND ile.[Location Code] = 'TAC'
        WHERE i.[Blocked] = 0
        GROUP BY i.[No_]
        ORDER BY i.[No_]
    ";

    match client.simple_query(sql).await {
        Ok(stream) => {
            let rows = stream.into_first_result().await.unwrap_or_default();
            let inventory: Vec<NavInventoryRow> = rows.iter().map(|row| NavInventoryRow {
                sku:         row.get::<&str, _>("sku").unwrap_or("").to_string(),
                qty_on_hand: row.get::<i64, _>("qty_on_hand").unwrap_or(0),
            }).collect();
            Ok(warp::reply::json(&inventory))
        }
        Err(e) => Ok(warp::reply::json(&serde_json::json!({ "error": e.to_string() }))),
    }
}
