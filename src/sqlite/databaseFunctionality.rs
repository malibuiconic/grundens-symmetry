use crate::sqlite::shopifyFunctionality::ShopifyStore;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        // Create data directory if it doesn't exist
        std::fs::create_dir_all("data").unwrap_or_else(|_| {
            eprintln!("Warning: Could not create data directory");
        });

        // Create SQLite connection with optimizations
        let pool = SqlitePool::connect("sqlite:data/loyalty_app.db?mode=rwc").await?;

        // Configure SQLite for performance
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA cache_size = -64000")
            .execute(&pool)
            .await?; // 64MB cache
        sqlx::query("PRAGMA temp_store = memory")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA mmap_size = 268435456")
            .execute(&pool)
            .await?; // 256MB mmap

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS shopify_stores (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                shop_domain TEXT NOT NULL UNIQUE,
                access_token TEXT NOT NULL,
                scope TEXT NOT NULL,
                installed_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Create indexes for fast lookups
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_shop_domain ON shopify_stores(shop_domain)")
            .execute(&pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_active_stores ON shopify_stores(is_active)")
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    // Store or update Shopify store token
    pub async fn upsert_store(
        &self,
        shop_domain: &str,
        access_token: &str,
        scope: &str,
    ) -> Result<ShopifyStore> {
        let now = Utc::now();

        // Use runtime query instead of macro
        let row = sqlx::query(
            r#"
            INSERT INTO shopify_stores (shop_domain, access_token, scope, installed_at, updated_at, is_active)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(shop_domain) DO UPDATE SET
                access_token = excluded.access_token,
                scope = excluded.scope,
                updated_at = excluded.updated_at,
                is_active = 1
            RETURNING 
                id,
                shop_domain,
                access_token,
                scope,
                installed_at,
                updated_at,
                is_active
            "#
        )
        .bind(shop_domain)
        .bind(access_token)
        .bind(scope)
        .bind(now.timestamp())
        .bind(now.timestamp())
        .bind(true)
        .fetch_one(&self.pool)
        .await?;

        // Manually map the row to ShopifyStore
        let store = ShopifyStore {
            id: Some(row.get("id")),
            shop_domain: row.get("shop_domain"),
            access_token: row.get("access_token"),
            scope: row.get("scope"),
            installed_at: DateTime::from_timestamp(row.get("installed_at"), 0).unwrap_or_default(),
            updated_at: DateTime::from_timestamp(row.get("updated_at"), 0).unwrap_or_default(),
            is_active: row.get("is_active"),
        };

        Ok(store)
    }

    // Get store by shop domain
    pub async fn get_store(&self, shop_domain: &str) -> Result<Option<ShopifyStore>> {
        println!("🔍 Database lookup for shop: {}", shop_domain);
        info!("Database lookup for shop: {}", shop_domain);

        let row = sqlx::query(
            r#"
            SELECT 
                id,
                shop_domain,
                access_token,
                scope,
                installed_at,
                updated_at,
                is_active
            FROM shopify_stores 
            WHERE shop_domain = ?1 AND is_active = 1
            "#,
        )
        .bind(shop_domain)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            println!("✅ Found existing store in database: {}", shop_domain);
            info!("Found existing store in database: {}", shop_domain);

            let store = ShopifyStore {
                id: Some(row.get("id")),
                shop_domain: row.get("shop_domain"),
                access_token: row.get("access_token"),
                scope: row.get("scope"),
                installed_at: DateTime::from_timestamp(row.get("installed_at"), 0)
                    .unwrap_or_default(),
                updated_at: DateTime::from_timestamp(row.get("updated_at"), 0).unwrap_or_default(),
                is_active: row.get("is_active"),
            };
            Ok(Some(store))
        } else {
            println!("❌ No store found in database for: {}", shop_domain);
            info!("No store found in database for: {}", shop_domain);
            Ok(None)
        }
    }

    // Health check
    pub async fn health_check(&self) -> Result<bool> {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(true)
    }
}
