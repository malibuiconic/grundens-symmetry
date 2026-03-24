#![allow(unused)]
#[macro_use]
extern crate log;

use anyhow::Result;
use envconfig::EnvConfig;
use std::convert::Infallible;
use std::sync::Arc;
use std::{net::SocketAddr, str::FromStr};
use warp::reply::with::headers;
use warp::{http::HeaderMap, http::StatusCode, Buf, Filter, Rejection, Reply};


// Shopify Oauth Flow
use crate::shopify::oauth_flow::shopify_callback_handler;
use crate::shopify::oauth_flow::shopify_install;

// Database
// use crate::sqlite::cleanupServiceFunctionality::start_cleanup_service;
use crate::sqlite::databaseFunctionality::Database;

// NAV18/17
use crate::nav::handle_nav_status;
use crate::nav::handle_nav_items;
use crate::nav::handle_nav_inventory;
// use crate::nav::check_nav_connections;

// Pricing Export
use crate::pricing::{
    handle_pricing_ui, handle_generate, handle_list_csvs,
    handle_download_csv, handle_delete_csv,
};

/*
// Loyalty Endpoints
use crate::loyalty::loyalty_handlers::{
    celigo_callback_endpoint, create_token_endpoint, get_points_endpoint,
    update_customer_metafields_endpoint, update_customer_loyalty_metafields_endpoint,
    escrow_operation_endpoint, simulate_customer_endpoint,
};

// Discount Endpoints
use crate::discounting::{
    create_discount_code_endpoint, mark_discount_applied_endpoint, DiscountManager,
    start_discount_cleanup_service,
};

// Cart Transform Endpoints
use crate::cart_transform::{
    get_cart_transform_status_endpoint, toggle_cart_transform_endpoint, CartTransformManager,
};

// Configuration Endpoints
use crate::configurations_manager::{
    get_global_config_endpoint, set_global_config_endpoint, validate_config_endpoint,
    deploy_config_endpoint, get_config_history_endpoint, get_deployment_history_endpoint,
    rollback_config_endpoint, get_config_presets_endpoint, ConfigManager, with_config_manager,
};
*/

// Custom Libs:
mod envconfig;     // environment variable(s) handler
// mod loyalty;    // loyalty module
mod shopify;       // shopify context
mod sqlite;        // database/session storage
mod nav;           // Grundens NAV18 direct SQL
mod pricing;       // pricing CSV export + management UI

// mod discounting;
// mod cart_transform; // cart transform management
// mod configurations_manager; // configuration management

pub type WebResult<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    // Initalize Database
    info!("Initializing SQLite database..");
    let db = Arc::new(Database::new().await?);

    /*
    // Initialize loyalty tables
    info!("Initializing loyalty tables...");
    if let Err(e) = db.init_loyalty_tables().await {
        error!("Failed to initialize loyalty tables: {}", e);
        return Err(e);
    }

    // Initialize discount manager and tables
    info!("Initializing discount manager...");
    let discount_manager = Arc::new(DiscountManager::new(db.clone()));
    if let Err(e) = discount_manager.init_discount_tables().await {
        error!("Failed to initialize discount tables: {}", e);
        return Err(anyhow::anyhow!("Failed to initialize discount tables: {}", e));
    }
    

    // Initialize cart transform manager and tables
    info!("Initializing cart transform manager...");
    let cart_transform_manager = Arc::new(CartTransformManager::new(db.clone()));
    if let Err(e) = cart_transform_manager.init_cart_transform_tables().await {
        error!("Failed to initialize cart transform tables: {}", e);
        return Err(anyhow::anyhow!("Failed to initialize cart transform tables: {}", e));
    }

    // Initialize configuration manager and tables
    info!("Initializing configuration manager...");
    let config_manager = Arc::new(ConfigManager::new(db.clone()));
    if let Err(e) = config_manager.init_config_tables().await {
        error!("Failed to initialize configuration tables: {}", e);
        return Err(anyhow::anyhow!("Failed to initialize configuration tables: {}", e));
    }
    */

    info!("Database initialized successfully!");

    // Start cleanup service
    // info!("Starting cleanup service...");
    // start_cleanup_service(db.clone()).await?;

    // Start discount cleanup service
    // info!("Starting discount cleanup service...");
    // start_discount_cleanup_service(discount_manager.clone()).await.map_err(|e| anyhow::anyhow!("Failed to start discount cleanup: {}", e))?;

    // Test Database connection
    match db.health_check().await {
        Ok(_) => info!("Database health check passed"),
        Err(e) => {
            error!("Database health check failed: {}", e);
            return Err(e);
        }
    }
    

    // setup environment variables
    let app_env = EnvConfig::env_variables().unwrap();

    // NAV DB connectivity is verified on-demand (pricing export, /api/nav/* routes).
    // Removed startup probe — malibudev's IP is not whitelisted in Azure SQL firewall
    // and the probe goes direct rather than through the SSH tunnel, causing noise in
    // Grundens' Azure audit logs.
    // init port setup for hosting environment (GCP/Linode/Docker or whatever)
    let port = match std::env::var("SERVER_PORT") {
        Ok(port) => port,
        _ => app_env.clone().server_port,
    };
    let address = format!("0.0.0.0:{}", port);
    let socket_address = SocketAddr::from_str(&address);

    //  Determine the Host (used in Shopify Oauth steps)
    let get_host = warp::header::<String>("host");

    // Shopify Application Install Routes
    // #1 - with install link provided by partners ;)
    let shopify_install = warp::path!("shopify_install")
        .and(warp::get())
        .and(warp::query())
        .and(with_config(app_env.clone()))
        .and(with_database(db.clone()))
        .and(get_host)
        .and_then(shopify_install);
    // #2 - callback from shopify - sets DB values and redircts to YEW Interface
    let shopify_callback = warp::path!("shopify" / "callback")
        .and(warp::get())
        .and(warp::query())
        .and(with_config(app_env.clone()))
        .and(with_database(db.clone()))
        .and(get_host)
        .and_then(shopify_callback_handler);

    let shopify_app_install_routes = shopify_install.or(shopify_callback);
    
    // #3 - Malibu Dash (WASM)
    let malibu_dash_files = warp::path("malibu_dash")
        .and(warp::fs::dir("src/malibu_dash"));
    let malibu_dash_fallback = warp::path("malibu_dash")
        .and(warp::fs::file("src/malibu_dash/index.html"));
    
    let malibu_dash = malibu_dash_files.or(malibu_dash_fallback);
    
    /*
    // #4 - Loyalty Endpoints
    // Create token endpoint (this is used for storage in SQLlite instances)
    let create_token = warp::path!("api" / "loyalty" / "token")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_database(db.clone()))
        .and(with_config(app_env.clone()))
        .and_then(create_token_endpoint);

    // Celigo callback webhook
    let celigo_callback = warp::path!("webhook" / "celigo-callback")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_database(db.clone()))
        .and_then(celigo_callback_endpoint);

    // Update customer app metafields (simulates Celigo customer sync)
    let update_metafields = warp::path!("api" / "loyalty" / "update-metafields")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_database(db.clone()))
        .and(with_config(app_env.clone()))
        .and_then(update_customer_metafields_endpoint);

    // Enhanced loyalty metafields endpoint with escrow support
    let update_loyalty_metafields = warp::path!("api" / "loyalty" / "update-loyalty-metafields")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_database(db.clone()))
        .and(with_config(app_env.clone()))
        .and_then(update_customer_loyalty_metafields_endpoint);

    // Escrow operations endpoint
    let escrow_operations = warp::path!("api" / "loyalty" / "escrow")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_database(db.clone()))
        .and(with_config(app_env.clone()))
        .and_then(escrow_operation_endpoint);

    // Simulation endpoint for testing
    let simulate_customer = warp::path!("api" / "loyalty" / "simulate")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_database(db.clone()))
        .and(with_config(app_env.clone()))
        .and_then(simulate_customer_endpoint);

    // Get points by token
    let get_points = warp::path!("api" / "loyalty" / "points")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_database(db.clone()))
        .and_then(get_points_endpoint);

    let loyalty_routes = create_token
        .or(celigo_callback)
        .or(update_metafields)
        .or(update_loyalty_metafields)
        .or(escrow_operations)
        .or(simulate_customer)
        .or(get_points);

    // #5 - Discount Endpoints
    // Create discount code endpoint
    let create_discount = warp::path!("api" / "loyalty" / "create-discount")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_discount_manager(discount_manager.clone()))
        .and(with_database(db.clone()))
        .and(with_config(app_env.clone()))
        .and_then(create_discount_code_endpoint);

    // Mark discount as applied endpoint
    let mark_discount_applied = warp::path!("api" / "loyalty" / "mark-discount-applied")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_discount_manager(discount_manager.clone()))
        .and_then(mark_discount_applied_endpoint);

    let discount_routes = create_discount.or(mark_discount_applied);
    
    // #6 - Cart Transform Endpoints
    // Get cart transform status endpoint
    let get_cart_transform_status = warp::path!("api" / "cart-transform" / "status" / String)
        .and(warp::get())
        .and(with_cart_transform_manager(cart_transform_manager.clone()))
        .and_then(get_cart_transform_status_endpoint);

    // Toggle cart transform endpoint
    let toggle_cart_transform = warp::path!("api" / "cart-transform" / "toggle" / String / bool)
        .and(warp::post())
        .and(with_cart_transform_manager(cart_transform_manager.clone()))
        .and_then(toggle_cart_transform_endpoint);

    let cart_transform_routes = get_cart_transform_status.or(toggle_cart_transform);
    
    // #7 - Configuration Endpoints
    // Get global configuration
    let get_global_config = warp::path!("api" / "config" / "global")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_config_manager(config_manager.clone()))
        .and(with_config(app_env.clone()))
        .and_then(get_global_config_endpoint);

    // Set global configuration
    let set_global_config = warp::path!("api" / "config" / "global")
        .and(warp::put())
        .and(warp::body::json())
        .and(with_config_manager(config_manager.clone()))
        .and(with_config(app_env.clone()))
        .and_then(set_global_config_endpoint);

    // Validate configuration
    let validate_config = warp::path!("api" / "config" / "validate")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(validate_config_endpoint);

    // Deploy configuration
    let deploy_config = warp::path!("api" / "config" / "deploy")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_config_manager(config_manager.clone()))
        .and(with_config(app_env.clone()))
        .and_then(deploy_config_endpoint);

    // Get configuration history
    let get_config_history = warp::path!("api" / "config" / "history")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_config_manager(config_manager.clone()))
        .and_then(get_config_history_endpoint);

    // Get deployment history
    let get_deployment_history = warp::path!("api" / "config" / "deployment-history")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_config_manager(config_manager.clone()))
        .and_then(get_deployment_history_endpoint);

    // Rollback configuration
    let rollback_config = warp::path!("api" / "config" / "rollback")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_config_manager(config_manager.clone()))
        .and(with_config(app_env.clone()))
        .and_then(rollback_config_endpoint);

    // Get configuration presets
    let get_config_presets = warp::path!("api" / "config" / "presets")
        .and(warp::get())
        .and_then(get_config_presets_endpoint);

    let config_routes = get_global_config
        .or(set_global_config)
        .or(validate_config)
        .or(deploy_config)
        .or(get_config_history)
        .or(get_deployment_history)
        .or(rollback_config)
        .or(get_config_presets);
    
    */
    // ---------------------------------------------------------------------------
    // Grundens NAV18 Routes  (/api/nav/*)
    // Read-only — requires Cisco Secure Connect VPN to reach sql-grus-prd-01
    // ---------------------------------------------------------------------------
    let nav_status = warp::path!("api" / "nav" / "status")
        .and(warp::get())
        .and(with_config(app_env.clone()))
        .and_then(handle_nav_status);

    let nav_items = warp::path!("api" / "nav" / "items")
        .and(warp::get())
        .and(with_config(app_env.clone()))
        .and_then(handle_nav_items);

    let nav_inventory = warp::path!("api" / "nav" / "inventory")
        .and(warp::get())
        .and(with_config(app_env.clone()))
        .and_then(handle_nav_inventory);

    let nav_routes = nav_status.or(nav_items).or(nav_inventory);

    // ---------------------------------------------------------------------------
    // Pricing Export Routes  (/pricing, /api/pricing/*)
    // Shopify + NAV18 pricing merge → Matrixify CSV management UI
    // ---------------------------------------------------------------------------
    let pricing_ui = warp::path("pricing")
        .and(warp::get())
        .and(warp::path::end())
        .and_then(handle_pricing_ui);

    let pricing_generate = warp::path!("api" / "pricing" / "generate")
        .and(warp::post())
        .and(with_config(app_env.clone()))
        .and(with_database(db.clone()))
        .and_then(handle_generate);

    let pricing_list = warp::path!("api" / "pricing" / "csvs")
        .and(warp::get())
        .and_then(handle_list_csvs);

    let pricing_download = warp::path!("api" / "pricing" / "csv" / String)
        .and(warp::get())
        .and_then(handle_download_csv);

    let pricing_delete = warp::path!("api" / "pricing" / "csv" / String)
        .and(warp::delete())
        .and_then(handle_delete_csv);

    let pricing_routes = pricing_ui
        .or(pricing_generate)
        .or(pricing_list)
        .or(pricing_download)
        .or(pricing_delete);

    // Health check route
    let health_route =
        warp::path::end().map(|| format!("@Malibu Health: Status - {}", StatusCode::OK));

    // Combine all routes
    let routes = shopify_app_install_routes
        .or(malibu_dash)
        //.or(loyalty_routes)
        //.or(discount_routes)
        //.or(cart_transform_routes)
        //.or(config_routes)
        .or(nav_routes)
        .or(pricing_routes)
        .or(health_route)
        .with(
            warp::cors()
                .allow_credentials(true)
                .allow_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
                .allow_headers(vec![
                    "Content-Type",
                    "Authorization",
                    "X-Shopify-Hmac-Sha256",
                    "Origin",
                    "X-Requested-With",
                    "Accept",
                    "Access-Control-Allow-Origin",
                    "Access-Control-Allow-Headers",
                    "Access-Control-Allow-Methods",
                    "Access-Control-Allow-Credentials",
                ])
                .allow_origin("https://extensions.shopifycdn.com")
                .allow_origin("https://grundens.myshopify.com")
                .allow_any_origin(), // Add this back - this is what makes it work
        )
        .with(warp::reply::with::header(
            "Content-Security-Policy",
            "frame-ancestors https://admin.shopify.com https://*.myshopify.com",
        ))
        .with(warp::reply::with::header(
            "X-Frame-Options",
            "ALLOW-FROM https://admin.shopify.com",
        ));

    info!("Starting server on {}", address);
    warp::serve(routes).run(socket_address.unwrap()).await;

    Ok(())
}

// Helper Functions/Filters
fn with_config(
    app_env: EnvConfig,
) -> impl Filter<Extract = (EnvConfig,), Error = Infallible> + Clone {
    warp::any().map(move || app_env.clone())
}

fn with_database(
    db: Arc<Database>,
) -> impl Filter<Extract = (Arc<Database>,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}
/*
fn with_discount_manager(
    discount_manager: Arc<DiscountManager>,
) -> impl Filter<Extract = (Arc<DiscountManager>,), Error = Infallible> + Clone {
    warp::any().map(move || discount_manager.clone())
}

fn with_cart_transform_manager(
    cart_transform_manager: Arc<CartTransformManager>,
) -> impl Filter<Extract = (Arc<CartTransformManager>,), Error = Infallible> + Clone {
    warp::any().map(move || cart_transform_manager.clone())
}
*/
