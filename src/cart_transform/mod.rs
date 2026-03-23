// src/cart_transform/mod.rs - Cart Transform Management Module

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use warp::{Reply, Rejection};
use crate::sqlite::databaseFunctionality::Database;
use crate::configurations_manager::types::CartTransformSettings;
use log::{info, error};
use sqlx::Row;

// ===== STRUCTS =====

pub struct CartTransformManager {
    db: Arc<Database>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CartTransformFunction {
    pub id: String,
    pub api_type: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CartTransformRegistration {
    pub shop_domain: String,
    pub function_id: String,
    pub transform_id: Option<String>,
    pub enabled: bool,
    pub capability_status: String, // "supported", "unsupported", "unknown"
    pub error_reason: Option<String>,
}

#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Deserialize)]
struct ShopifyFunctionsResponse {
    #[serde(rename = "shopifyFunctions")]
    shopify_functions: FunctionConnection,
}

#[derive(Deserialize)]
struct FunctionConnection {
    edges: Vec<FunctionEdge>,
}

#[derive(Deserialize)]
struct FunctionEdge {
    node: FunctionNode,
}

#[derive(Deserialize)]
struct FunctionNode {
    id: String,
    #[serde(rename = "apiType")]
    api_type: String,
    title: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct CartTransformCreateResponse {
    #[serde(rename = "cartTransformCreate")]
    cart_transform_create: Option<CartTransformCreatePayload>,
}

#[derive(Deserialize)]
struct CartTransformCreatePayload {
    #[serde(rename = "cartTransform")]
    cart_transform: Option<CartTransformNode>,
    #[serde(rename = "userErrors")]
    user_errors: Vec<UserError>,
}

#[derive(Deserialize)]
struct CartTransformNode {
    id: String,
    #[serde(rename = "functionId")]
    function_id: String,
}

#[derive(Deserialize)]
struct UserError {
    message: String,
}

#[derive(Deserialize)]
struct ExistingCartTransformsResponse {
    #[serde(rename = "cartTransforms")]
    cart_transforms: CartTransformConnection,
}

#[derive(Deserialize)]
struct CartTransformConnection {
    edges: Vec<CartTransformEdge>,
}

#[derive(Deserialize)]
struct CartTransformEdge {
    node: ExistingCartTransformNode,
}

#[derive(Deserialize)]
struct ExistingCartTransformNode {
    id: String,
    #[serde(rename = "functionId")]
    function_id: String,
}

// ===== IMPLEMENTATION =====

impl CartTransformManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    // Initialize cart transform tables with migration support
    pub async fn init_cart_transform_tables(&self) -> anyhow::Result<()> {
        let pool = &self.db.pool;
        
        // Check if table exists and get its schema
        let table_info = sqlx::query("PRAGMA table_info(cart_transforms)")
            .fetch_all(pool)
            .await?;
        
        if table_info.is_empty() {
            // Table doesn't exist, create it with the correct schema
            info!("Creating cart_transforms table with correct schema");
            let create_cart_transforms_table = r#"
                CREATE TABLE cart_transforms (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    shop_domain TEXT NOT NULL UNIQUE,
                    function_id TEXT,
                    transform_id TEXT,
                    enabled BOOLEAN DEFAULT true,
                    capability_status TEXT NOT NULL DEFAULT 'unknown',
                    error_reason TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT
                )
            "#;
            sqlx::query(create_cart_transforms_table).execute(pool).await?;
        } else {
            // Table exists, check if function_id has NOT NULL constraint
            let function_id_info = table_info.iter().find(|row| {
                let column_name: String = row.get("name");
                column_name == "function_id"
            });
            
            if let Some(column_info) = function_id_info {
                let not_null: i32 = column_info.get("notnull");
                if not_null == 1 {
                    // function_id has NOT NULL constraint, need to migrate
                    info!("Migrating cart_transforms table to allow NULL function_id");
                    
                    // Drop the temporary table if it exists from a previous failed migration
                    sqlx::query("DROP TABLE IF EXISTS cart_transforms_temp").execute(pool).await?;
                    
                    // Create new table with correct schema
                    sqlx::query(r#"
                        CREATE TABLE cart_transforms_temp (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            shop_domain TEXT NOT NULL UNIQUE,
                            function_id TEXT,
                            transform_id TEXT,
                            enabled BOOLEAN DEFAULT true,
                            capability_status TEXT NOT NULL DEFAULT 'unknown',
                            error_reason TEXT,
                            created_at TEXT NOT NULL,
                            updated_at TEXT
                        )
                    "#).execute(pool).await?;
                    
                    // Copy existing data
                    sqlx::query(r#"
                        INSERT INTO cart_transforms_temp 
                        (id, shop_domain, function_id, transform_id, enabled, capability_status, error_reason, created_at, updated_at)
                        SELECT id, shop_domain, function_id, transform_id, enabled, 
                               COALESCE(capability_status, 'unknown'), error_reason, created_at, updated_at
                        FROM cart_transforms
                    "#).execute(pool).await?;
                    
                    // Drop old table and rename new one
                    sqlx::query("DROP TABLE cart_transforms").execute(pool).await?;
                    sqlx::query("ALTER TABLE cart_transforms_temp RENAME TO cart_transforms").execute(pool).await?;
                    
                    info!("✅ Cart Transform table migration completed");
                } else {
                    // Check if new columns exist and add them if they don't
                    let has_capability_status = table_info.iter().any(|row| {
                        let column_name: String = row.get("name");
                        column_name == "capability_status"
                    });
                    
                    let has_error_reason = table_info.iter().any(|row| {
                        let column_name: String = row.get("name");
                        column_name == "error_reason"
                    });
                    
                    // Add missing columns
                    if !has_capability_status {
                        info!("Adding capability_status column to cart_transforms table");
                        sqlx::query("ALTER TABLE cart_transforms ADD COLUMN capability_status TEXT NOT NULL DEFAULT 'unknown'")
                            .execute(pool)
                            .await?;
                    }
                    
                    if !has_error_reason {
                        info!("Adding error_reason column to cart_transforms table");
                        sqlx::query("ALTER TABLE cart_transforms ADD COLUMN error_reason TEXT")
                            .execute(pool)
                            .await?;
                    }
                }
            }
        }
        
        info!("✅ Cart Transform tables initialized with migrations");
        Ok(())
    }

    // Find Cart Transform function ID by querying Shopify Functions
    pub async fn find_cart_transform_function(&self, access_token: &str, shop_domain: &str) -> anyhow::Result<Option<String>> {
        let client = reqwest::Client::new();
        
        let query = r#"
            query {
                shopifyFunctions(first: 20) {
                    edges {
                        node {
                            id
                            apiType
                            title
                            description
                        }
                    }
                }
            }
        "#;

        let graphql_request = json!({
            "query": query
        });

        info!("🔍 Querying for Cart Transform functions on {}", shop_domain);
        
        let response = client
            .post(&format!("https://{}/admin/api/2024-01/graphql.json", shop_domain))
            .header("X-Shopify-Access-Token", access_token)
            .header("Content-Type", "application/json")
            .json(&graphql_request)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;
        
        if !status.is_success() {
            error!("GraphQL request failed with status {}: {}", status, response_text);
            return Err(anyhow::anyhow!("GraphQL request failed: {}", status));
        }

        let graphql_response: GraphQLResponse<ShopifyFunctionsResponse> = 
            serde_json::from_str(&response_text)?;

        if let Some(errors) = graphql_response.errors {
            let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
            error!("GraphQL errors: {:?}", error_messages);
            return Err(anyhow::anyhow!("GraphQL errors: {}", error_messages.join(", ")));
        }

        let data = graphql_response.data
            .ok_or_else(|| anyhow::anyhow!("No data in GraphQL response"))?;

        // Look for our loyalty cart transformer function
        for edge in data.shopify_functions.edges {
            let function = edge.node;
            if function.api_type == "cart_transform" && 
               (function.title.contains("loyalty") || function.title.contains("Loyalty")) {
                info!("✅ Found Cart Transform function: {} ({})", function.title, function.id);
                return Ok(Some(function.id));
            }
        }

        info!("⚠️ No Cart Transform function found for loyalty");
        Ok(None)
    }

    // Check for existing Cart Transforms
    pub async fn check_existing_cart_transforms(&self, access_token: &str, shop_domain: &str) -> anyhow::Result<Option<String>> {
        let client = reqwest::Client::new();
        
        let query = r#"
            query {
                cartTransforms(first: 10) {
                    edges {
                        node {
                            id
                            functionId
                        }
                    }
                }
            }
        "#;

        let graphql_request = json!({
            "query": query
        });

        info!("🔍 Checking for existing Cart Transforms on {}", shop_domain);
        
        let response = client
            .post(&format!("https://{}/admin/api/2024-01/graphql.json", shop_domain))
            .header("X-Shopify-Access-Token", access_token)
            .header("Content-Type", "application/json")
            .json(&graphql_request)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;
        
        if !status.is_success() {
            error!("GraphQL request failed with status {}: {}", status, response_text);
            return Err(anyhow::anyhow!("GraphQL request failed: {}", status));
        }

        let graphql_response: GraphQLResponse<ExistingCartTransformsResponse> = 
            serde_json::from_str(&response_text)?;

        if let Some(errors) = graphql_response.errors {
            let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
            error!("GraphQL errors: {:?}", error_messages);
            return Err(anyhow::anyhow!("GraphQL errors: {}", error_messages.join(", ")));
        }

        let data = graphql_response.data
            .ok_or_else(|| anyhow::anyhow!("No data in GraphQL response"))?;

        // Check if any existing Cart Transform exists
        for edge in data.cart_transforms.edges {
            let transform = edge.node;
            info!("✅ Found existing Cart Transform: {} (Function: {})", transform.id, transform.function_id);
            return Ok(Some(transform.id));
        }

        info!("ℹ️ No existing Cart Transforms found");
        Ok(None)
    }

    // Register Cart Transform with Shopify
    pub async fn register_cart_transform(&self, access_token: &str, shop_domain: &str, function_id: &str) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        
        let mutation = r#"
            mutation cartTransformCreate($functionId: String!) {
                cartTransformCreate(functionId: $functionId) {
                    cartTransform {
                        id
                        functionId
                    }
                    userErrors {
                        field
                        message
                    }
                }
            }
        "#;

        let variables = json!({
            "functionId": function_id
        });

        let graphql_request = json!({
            "query": mutation,
            "variables": variables
        });

        info!("🎫 Creating Cart Transform registration for {}", shop_domain);
        
        let response = client
            .post(&format!("https://{}/admin/api/2024-01/graphql.json", shop_domain))
            .header("X-Shopify-Access-Token", access_token)
            .header("Content-Type", "application/json")
            .json(&graphql_request)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;
        
        if !status.is_success() {
            error!("GraphQL request failed with status {}: {}", status, response_text);
            return Err(anyhow::anyhow!("GraphQL request failed: {}", status));
        }

        let graphql_response: GraphQLResponse<CartTransformCreateResponse> = 
            serde_json::from_str(&response_text)?;

        if let Some(errors) = graphql_response.errors {
            let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
            error!("GraphQL errors: {:?}", error_messages);
            return Err(anyhow::anyhow!("GraphQL errors: {}", error_messages.join(", ")));
        }

        let data = graphql_response.data
            .ok_or_else(|| anyhow::anyhow!("No data in GraphQL response"))?;

        let create_payload = data.cart_transform_create
            .ok_or_else(|| anyhow::anyhow!("No cartTransformCreate in response"))?;

        if !create_payload.user_errors.is_empty() {
            let error_messages: Vec<String> = create_payload.user_errors
                .iter()
                .map(|e| e.message.clone())
                .collect();
            error!("User errors creating Cart Transform: {:?}", error_messages);
            return Err(anyhow::anyhow!("Cart Transform creation errors: {}", error_messages.join(", ")));
        }

        let cart_transform = create_payload.cart_transform
            .ok_or_else(|| anyhow::anyhow!("No cart transform returned"))?;

        info!("✅ Successfully created Cart Transform: {}", cart_transform.id);
        Ok(cart_transform.id)
    }

    // Store Cart Transform registration in database
    pub async fn store_cart_transform(&self, shop_domain: &str, function_id: &str, transform_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let query = r#"
            INSERT OR REPLACE INTO cart_transforms 
            (shop_domain, function_id, transform_id, enabled, created_at, updated_at)
            VALUES ($1, $2, $3, true, $4, $4)
        "#;

        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(query)
            .bind(shop_domain)
            .bind(function_id)
            .bind(transform_id)
            .bind(&now)
            .execute(&self.db.pool)
            .await?;

        info!("📝 Stored Cart Transform registration for {}", shop_domain);
        Ok(())
    }

    // Get Cart Transform status for a shop
    pub async fn get_transform_status(&self, shop_domain: &str) -> Result<Option<CartTransformRegistration>, Box<dyn std::error::Error>> {
        let query = r#"
            SELECT shop_domain, function_id, transform_id, enabled, capability_status, error_reason
            FROM cart_transforms 
            WHERE shop_domain = $1
        "#;

        let row = sqlx::query(query)
            .bind(shop_domain)
            .fetch_optional(&self.db.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(CartTransformRegistration {
                shop_domain: row.get("shop_domain"),
                function_id: row.get::<Option<String>, _>("function_id").unwrap_or_default(),
                transform_id: row.get("transform_id"),
                enabled: row.get("enabled"),
                capability_status: row.get("capability_status"),
                error_reason: row.get("error_reason"),
            }))
        } else {
            Ok(None)
        }
    }

    // Enable/disable Cart Transform
    pub async fn toggle_cart_transform(&self, shop_domain: &str, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
        let query = r#"
            UPDATE cart_transforms 
            SET enabled = $1, updated_at = $2
            WHERE shop_domain = $3
        "#;

        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(query)
            .bind(enabled)
            .bind(&now)
            .bind(shop_domain)
            .execute(&self.db.pool)
            .await?;

        info!("🔄 Cart Transform {} for {}", if enabled { "enabled" } else { "disabled" }, shop_domain);
        Ok(())
    }

    // Store Cart Transform capability status
    pub async fn store_capability_status(&self, shop_domain: &str, function_id: Option<&str>, transform_id: Option<&str>, status: &str, error_reason: Option<&str>) -> anyhow::Result<()> {
        let query = r#"
            INSERT OR REPLACE INTO cart_transforms 
            (shop_domain, function_id, transform_id, enabled, capability_status, error_reason, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
        "#;

        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(query)
            .bind(shop_domain)
            .bind(function_id)
            .bind(transform_id)
            .bind(status == "supported")
            .bind(status)
            .bind(error_reason)
            .bind(&now)
            .execute(&self.db.pool)
            .await?;

        info!("📝 Stored Cart Transform capability status '{}' for {}", status, shop_domain);
        Ok(())
    }

    // Apply cart transform settings from configuration
    pub async fn apply_cart_transform_config(&self, shop_domain: &str, access_token: &str, config_manager: Arc<crate::configurations_manager::ConfigManager>) -> anyhow::Result<()> {
        info!("🔧 Applying Cart Transform configuration for {}", shop_domain);
        
        // Get current configuration
        let config = match config_manager.get_global_config(access_token, shop_domain).await {
            Ok(Some(config)) => config,
            Ok(None) => {
                info!("⚠️ No configuration found for {}, using defaults", shop_domain);
                return Ok(());
            }
            Err(e) => {
                error!("❌ Failed to get configuration for {}: {}", shop_domain, e);
                return Err(anyhow::anyhow!("Failed to get configuration: {}", e));
            }
        };

        // Update cart transform enabled status based on configuration
        let cart_settings = &config.cart_transform_settings;
        self.toggle_cart_transform(shop_domain, cart_settings.enabled).await
            .map_err(|e| anyhow::anyhow!("Failed to update cart transform status: {}", e))?;

        info!("✅ Cart Transform configuration applied for {} (enabled: {})", shop_domain, cart_settings.enabled);
        Ok(())
    }

    // Complete setup process for OAuth flow with enhanced error handling
    pub async fn setup_cart_transform_for_shop(&self, access_token: &str, shop_domain: &str) -> anyhow::Result<()> {
        info!("🚀 Starting Cart Transform setup for {}", shop_domain);

        // Step 1: Find the Cart Transform function
        let function_id = match self.find_cart_transform_function(access_token, shop_domain).await {
            Ok(Some(id)) => {
                info!("✅ Found Cart Transform function for {}", shop_domain);
                id
            }
            Ok(None) => {
                info!("⚠️ No Cart Transform function found for {}, marking as unsupported", shop_domain);
                self.store_capability_status(shop_domain, None, None, "unsupported", Some("No Cart Transform function deployed")).await?;
                return Ok(());
            }
            Err(e) => {
                let error_msg = format!("Function lookup failed: {}", e);
                error!("❌ Error finding Cart Transform function for {}: {}", shop_domain, e);
                self.store_capability_status(shop_domain, None, None, "unknown", Some(&error_msg)).await?;
                return Ok(());
            }
        };

        // Step 2: Check for existing Cart Transforms first
        match self.check_existing_cart_transforms(access_token, shop_domain).await {
            Ok(Some(existing_transform_id)) => {
                // Cart Transform already exists - mark as supported
                info!("✅ Cart Transform already exists for {}: {}", shop_domain, existing_transform_id);
                self.store_capability_status(shop_domain, Some(&function_id), Some(&existing_transform_id), "supported", None).await?;
                info!("✅ Cart Transform setup complete for {} (using existing)", shop_domain);
                return Ok(());
            }
            Ok(None) => {
                info!("ℹ️ No existing Cart Transform found, proceeding with registration");
            }
            Err(e) => {
                let error_msg = format!("Failed to check existing Cart Transforms: {}", e);
                error!("❌ Error checking existing Cart Transforms for {}: {}", shop_domain, e);
                self.store_capability_status(shop_domain, Some(&function_id), None, "unknown", Some(&error_msg)).await?;
                return Ok(());
            }
        }

        // Step 3: Attempt to register new Cart Transform
        match self.register_cart_transform(access_token, shop_domain, &function_id).await {
            Ok(transform_id) => {
                // Step 4: Store successful registration
                self.store_capability_status(shop_domain, Some(&function_id), Some(&transform_id), "supported", None).await?;
                info!("✅ Cart Transform setup complete for {}", shop_domain);
            }
            Err(e) => {
                let error_msg = e.to_string();
                
                // Handle "already registered" case (shouldn't happen due to Step 2, but just in case)
                if error_msg.contains("already registered") {
                    info!("ℹ️ Cart Transform already registered for {}, checking existing transforms", shop_domain);
                    // Try to find the existing transform ID
                    match self.check_existing_cart_transforms(access_token, shop_domain).await {
                        Ok(Some(existing_id)) => {
                            self.store_capability_status(shop_domain, Some(&function_id), Some(&existing_id), "supported", None).await?;
                            info!("✅ Cart Transform setup complete for {} (found existing after registration attempt)", shop_domain);
                            return Ok(());
                        }
                        _ => {
                            // Fallback: mark as supported without transform ID
                            self.store_capability_status(shop_domain, Some(&function_id), None, "supported", Some("Cart Transform exists but ID not found")).await?;
                            info!("✅ Cart Transform setup complete for {} (exists but ID unknown)", shop_domain);
                            return Ok(());
                        }
                    }
                }
                
                // Determine capability status based on error type
                let (status, reason) = if error_msg.contains("write_cart_transforms") {
                    ("unsupported", "Missing write_cart_transforms scope".to_string())
                } else if error_msg.contains("Checkout Extensibility") {
                    ("unsupported", "Store not upgraded to Checkout Extensibility".to_string())
                } else if error_msg.contains("products and preferences permission") {
                    ("unsupported", "User lacks products and preferences permission".to_string())
                } else if error_msg.contains("Access denied") {
                    ("unsupported", "Access denied for Cart Transform creation".to_string())
                } else {
                    ("unknown", format!("Registration failed: {}", error_msg))
                };

                self.store_capability_status(shop_domain, Some(&function_id), None, status, Some(&reason)).await?;
                
                if status == "unsupported" {
                    info!("⚠️ Cart Transform unsupported for {}: {}", shop_domain, reason);
                } else {
                    error!("❌ Cart Transform registration failed for {}: {}", shop_domain, reason);
                }
            }
        }

        Ok(())
    }
}

// ===== HANDLERS =====

pub async fn get_cart_transform_status_endpoint(
    shop_domain: String,
    cart_transform_manager: Arc<CartTransformManager>,
) -> Result<impl Reply, Rejection> {
    match cart_transform_manager.get_transform_status(&shop_domain).await {
        Ok(Some(status)) => {
            Ok(warp::reply::json(&json!({
                "success": true,
                "status": status
            })))
        }
        Ok(None) => {
            Ok(warp::reply::json(&json!({
                "success": true,
                "status": null,
                "message": "No Cart Transform registered"
            })))
        }
        Err(e) => {
            error!("Failed to get Cart Transform status: {}", e);
            Ok(warp::reply::json(&json!({
                "success": false,
                "error": "Failed to get Cart Transform status"
            })))
        }
    }
}

pub async fn toggle_cart_transform_endpoint(
    shop_domain: String,
    enabled: bool,
    cart_transform_manager: Arc<CartTransformManager>,
) -> Result<impl Reply, Rejection> {
    match cart_transform_manager.toggle_cart_transform(&shop_domain, enabled).await {
        Ok(_) => {
            Ok(warp::reply::json(&json!({
                "success": true,
                "message": format!("Cart Transform {} for {}", 
                    if enabled { "enabled" } else { "disabled" }, 
                    shop_domain)
            })))
        }
        Err(e) => {
            error!("Failed to toggle Cart Transform: {}", e);
            Ok(warp::reply::json(&json!({
                "success": false,
                "error": "Failed to toggle Cart Transform"
            })))
        }
    }
}

// ===== ERROR HANDLING =====

#[derive(Debug)]
pub enum CartTransformError {
    SetupFailed,
    ToggleFailed,
    StatusFailed,
}

impl warp::reject::Reject for CartTransformError {}

impl std::fmt::Display for CartTransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CartTransformError::SetupFailed => write!(f, "Failed to setup Cart Transform"),
            CartTransformError::ToggleFailed => write!(f, "Failed to toggle Cart Transform"),
            CartTransformError::StatusFailed => write!(f, "Failed to get Cart Transform status"),
        }
    }
}

impl std::error::Error for CartTransformError {}
