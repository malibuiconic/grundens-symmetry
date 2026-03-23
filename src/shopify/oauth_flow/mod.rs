use crate::sqlite::databaseFunctionality::Database;
// use crate::cart_transform::CartTransformManager;
use crate::EnvConfig;
use crate::WebResult;
use serde::{Deserialize, Serialize};

use reqwest::{Client, StatusCode};
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use textnonce::TextNonce;
use warp::{http::Uri, reject, reply::json, Rejection, Reply};
use log::{info, error};

#[derive(Deserialize, Debug, Clone)]
pub struct ShopifyInstall {
    hmac: String,
    host: String,
    shop: String,
    timestamp: i32,
}

// This endpoint handler is the whitelisted url in the partners dashboard of the application
// it is the starting point for the Oauth handshake!
pub async fn shopify_install(
    url_params: ShopifyInstall,
    app_env: EnvConfig,
    db: Arc<Database>,
    host: String,
) -> WebResult<impl Reply> {
    println!("Shopify Store Attempting Install: {}", url_params.shop);

    // Check if store is already installed and if scopes match
    match db.get_store(&url_params.shop).await {
        Ok(Some(store)) => {
            // Check if current scopes match the required scopes
            let current_scopes = &store.scope;
            let required_scopes = &app_env.shopify_scopes;
            
            info!(
                "🔍 Scope comparison for {}: Current='{}', Required='{}'",
                url_params.shop, current_scopes, required_scopes
            );
            
            // Simple scope comparison - if they don't match, allow re-auth
            if current_scopes == required_scopes {
                info!(
                    "✅ Store {} already installed with correct scopes, redirecting to dashboard",
                    url_params.shop
                );
                let app_dashboard_uri = "/malibu_dash".to_string();
                let uri = app_dashboard_uri.parse::<Uri>().unwrap();
                return Ok(warp::redirect(uri));
            } else {
                info!(
                    "🔄 Store {} exists but scopes have changed. Proceeding with re-authentication.",
                    url_params.shop
                );
            }
        }
        Ok(None) => {
            info!("❌ No store found in database for: {}", url_params.shop);
        }
        Err(e) => {
            error!("Database error checking store: {}", e);
            // Continue with OAuth flow even if DB check fails
        }
    }

    // Allow for Cache Access
    let random_64_state = TextNonce::new();
    let app_host = format!("https://{}", host);
    // build redirect back to shopify
    let oauthstring = &("https://".to_string()
        + &url_params.shop
        + &"/admin/oauth/authorize?client_id=".to_string()
        + &app_env.shopify_app_client_id
        + &"&scope=".to_string()
        + &app_env.shopify_scopes
        + &"&state=".to_string()
        + &random_64_state
        + &"&redirect_uri=".to_string()
        + &app_host
        + &"/shopify/callback".to_string());
    
    info!("🔗 Redirecting to OAuth URL: {}", oauthstring);
    println!("🔗 Redirecting to OAuth URL: {}", oauthstring);
    
    // create uri that shopify uses as a callback
    let uri = oauthstring.parse::<Uri>().unwrap();
    // Redirect back to shopify to get install page ;)
    Ok(warp::redirect(uri))
}

#[derive(Deserialize, Debug, Clone)]
pub struct ShopifyCode {
    code: String,
    state: String,
    shop: String,
}

#[derive(Serialize, Debug)]
pub struct ShopifyAuthValues {
    client_id: String,
    client_secret: String,
    code: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ShopifyReturnedPermToken {
    access_token: String,
    scope: String,
}

impl fmt::Display for ShopifyReturnedPermToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.access_token, self.scope)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShopifyStore {
    pub store_name: String,
    pub perm_token: String,
}

pub async fn shopify_callback_handler(
    url_params: ShopifyCode,
    app_env: EnvConfig,
    db: Arc<Database>,
    host: String,
) -> WebResult<impl Reply> {
    info!("Authenticating Shopify App for shop: {}", url_params.shop);

    let shop = url_params.shop.clone();
    let db_clone = db.clone();

    let body = ShopifyAuthValues {
        client_id: app_env.shopify_app_client_id.clone(),
        client_secret: app_env.shopify_app_client_password.clone(),
        code: url_params.code,
    };

    // Process OAuth token exchange in background with better error handling
    tokio::spawn(async move {
        info!("Starting OAuth token exchange for shop: {}", shop);

        match exchange_oauth_token(body, shop.clone(), db_clone.clone()).await {
            Ok(store) => {
                info!(
                    "✅ Successfully stored Shopify store: {} with ID: {:?}",
                    store.shop_domain, store.id
                );

                /*
                // Setup Cart Transform after successful OAuth
                let cart_transform_manager = Arc::new(CartTransformManager::new(db_clone));
                if let Err(e) = cart_transform_manager
                    .setup_cart_transform_for_shop(&store.access_token, &store.shop_domain)
                    .await
                {
                    error!("⚠️ Failed to setup Cart Transform for {}: {}", store.shop_domain, e);
                    // Don't fail the OAuth flow if Cart Transform setup fails
                } else {
                    info!("✅ Cart Transform setup completed for {}", store.shop_domain);
                }
                */
            }
            Err(e) => {
                error!("❌ Failed to process OAuth token for shop {}: {}", shop, e);
            }
        }
    });

    let app_dashboard_uri = "/malibu_dash".to_string();
    let uri = app_dashboard_uri.parse::<Uri>().unwrap();
    Ok(warp::redirect(uri))
}

// Separate function to handle OAuth token exchange and database storage
async fn exchange_oauth_token(
    body: ShopifyAuthValues,
    shop: String,
    db: Arc<Database>,
) -> anyhow::Result<crate::sqlite::shopifyFunctionality::ShopifyStore> {
    info!("🔄 Requesting access token from Shopify for shop: {}", shop);

    // Shopify Access Token Url
    let access_token_url = format!("https://{}/admin/oauth/access_token", shop);
    let client = Client::new();

    let request = client
        .post(access_token_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send request to Shopify: {}", e))?;

    match request.status() {
        StatusCode::OK => {
            info!("✅ Shopify App Authentication Success for: {}", shop);
        }
        err => {
            error!("❌ Received Auth Code Status Error for {}: {:?}", shop, err);
            return Err(anyhow::anyhow!(
                "OAuth authentication failed with status: {:?}",
                err
            ));
        }
    }

    let result_bytes = request
        .bytes()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read response bytes: {}", e))?;

    let result = std::str::from_utf8(&result_bytes[0..])
        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in response: {}", e))?;

    let shopify_perm_token: ShopifyReturnedPermToken = serde_json::from_str(result)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON response: {}", e))?;

    info!("🔑 Received Shopify token for shop: {}", shop);

    // Store in database
    info!("💾 Storing token in database for shop: {}", shop);
    let store = db
        .upsert_store(
            &shop,
            &shopify_perm_token.access_token,
            &shopify_perm_token.scope,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to store in database: {}", e))?;

    info!(
        "✅ Successfully stored shop {} in database with ID: {:?}",
        shop, store.id
    );

    Ok(store)
}
