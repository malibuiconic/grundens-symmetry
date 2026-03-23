use config::ConfigError;
use dotenv::dotenv;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct EnvConfig {
    pub server_port: String,
    // Shopify ENVS
    pub shopify_app_client_id: String,
    pub shopify_app_client_password: String,
    pub shopify_scopes: String,
    pub shopify_api_version: String,
    pub shopify_shop_name: String,
    // Grundens NAV SQL Server
    pub nav_sql_host: String,
    pub nav_sql_port: String,
    pub nav_sql_user: String,
    pub nav_sql_password: String,
    pub nav_sql_db_nav18: String,
    pub nav_sql_db_integration: String,
}

impl EnvConfig {
    pub fn env_variables() -> Result<Self, ConfigError> {
        dotenv().ok();
        let cf = config::Config::builder()
            .add_source(config::Environment::default())
            .build()?;
        cf.try_deserialize()
    }
}
