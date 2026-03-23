use crate::Database;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyData {
    pub customer_id: String,
    pub points: i64,
    pub tier: String,
    pub points_by_location: Option<HashMap<String, i64>>,
    pub last_sync: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowEntry {
    pub id: i64,
    pub checkout_token: String,
    pub customer_id: String,
    pub points_held: i64,
    pub discount_code: String,
    pub location: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: String,
}

impl Database {
    // Create loyalty-related tables
    pub async fn create_loyalty_tables(&self) -> Result<()> {
        // Loyalty cache table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS loyalty_cache (
                customer_id TEXT PRIMARY KEY,
                points INTEGER NOT NULL,
                tier TEXT NOT NULL,
                points_by_location TEXT,
                last_sync INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Points escrow table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS points_escrow (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                checkout_token TEXT NOT NULL UNIQUE,
                customer_id TEXT NOT NULL,
                points_held INTEGER NOT NULL,
                discount_code TEXT NOT NULL,
                location TEXT,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending'
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Redemption history table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS loyalty_redemptions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                customer_id TEXT NOT NULL,
                points_redeemed INTEGER NOT NULL,
                discount_code TEXT NOT NULL,
                location TEXT,
                created_at INTEGER NOT NULL,
                order_id TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_loyalty_expires ON loyalty_cache(expires_at)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_escrow_expires ON points_escrow(expires_at)")
            .execute(&self.pool)
            .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_escrow_customer ON points_escrow(customer_id, status)",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_redemptions_customer ON loyalty_redemptions(customer_id)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Get total points from cache (without escrow calculation)
    pub async fn get_total_points(&self, customer_id: &str) -> Result<i64> {
        let row = sqlx::query(
            "SELECT points FROM loyalty_cache WHERE customer_id = ?1 AND expires_at > ?2",
        )
        .bind(customer_id)
        .bind(Utc::now().timestamp())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(row.get("points"))
        } else {
            Ok(0)
        }
    }

    // Get available points (total minus escrow)
    pub async fn get_available_points(&self, customer_id: &str) -> Result<i64> {
        // Get total points from cache
        let total_points = self.get_total_points(customer_id).await?;

        // Subtract points in escrow
        let escrow_points: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(points_held), 0) FROM points_escrow 
             WHERE customer_id = ?1 AND status = 'pending' AND expires_at > ?2",
        )
        .bind(customer_id)
        .bind(Utc::now().timestamp())
        .fetch_one(&self.pool)
        .await?;

        Ok(total_points - escrow_points)
    }

    // Get cached loyalty data
    pub async fn get_loyalty_data(&self, customer_id: &str) -> Result<Option<LoyaltyData>> {
        let now = Utc::now().timestamp();

        let row =
            sqlx::query("SELECT * FROM loyalty_cache WHERE customer_id = ?1 AND expires_at > ?2")
                .bind(customer_id)
                .bind(now)
                .fetch_optional(&self.pool)
                .await?;

        if let Some(row) = row {
            let points_by_location: Option<HashMap<String, i64>> =
                if let Ok(json_str) = row.try_get::<String, _>("points_by_location") {
                    serde_json::from_str(&json_str).ok()
                } else {
                    None
                };

            Ok(Some(LoyaltyData {
                customer_id: row.get("customer_id"),
                points: row.get("points"),
                tier: row.get("tier"),
                points_by_location,
                last_sync: DateTime::from_timestamp(row.get("last_sync"), 0).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    // Store loyalty data with TTL
    pub async fn store_loyalty_data(&self, data: &LoyaltyData, ttl_seconds: i64) -> Result<()> {
        let now = Utc::now();
        let expires_at = now + Duration::seconds(ttl_seconds);

        let points_json = data
            .points_by_location
            .as_ref()
            .map(|p| serde_json::to_string(p).unwrap_or_default());

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO loyalty_cache 
            (customer_id, points, tier, points_by_location, last_sync, expires_at) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(&data.customer_id)
        .bind(data.points)
        .bind(&data.tier)
        .bind(points_json)
        .bind(data.last_sync.timestamp())
        .bind(expires_at.timestamp())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Create points escrow
    pub async fn create_escrow(
        &self,
        checkout_token: &str,
        customer_id: &str,
        points: i64,
        discount_code: &str,
        location: Option<String>,
    ) -> Result<()> {
        let now = Utc::now();
        let expires_at = now + Duration::minutes(30); // 30 min checkout timeout

        sqlx::query(
            r#"
            INSERT INTO points_escrow 
            (checkout_token, customer_id, points_held, discount_code, location, created_at, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#
        )
        .bind(checkout_token)
        .bind(customer_id)
        .bind(points)
        .bind(discount_code)
        .bind(location)
        .bind(now.timestamp())
        .bind(expires_at.timestamp())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Get escrow by checkout token
    pub async fn get_escrow_by_checkout(
        &self,
        checkout_token: &str,
    ) -> Result<Option<EscrowEntry>> {
        let row = sqlx::query("SELECT * FROM points_escrow WHERE checkout_token = ?1")
            .bind(checkout_token)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(EscrowEntry {
                id: row.get("id"),
                checkout_token: row.get("checkout_token"),
                customer_id: row.get("customer_id"),
                points_held: row.get("points_held"),
                discount_code: row.get("discount_code"),
                location: row.try_get("location").ok(),
                created_at: DateTime::from_timestamp(row.get("created_at"), 0).unwrap_or_default(),
                expires_at: DateTime::from_timestamp(row.get("expires_at"), 0).unwrap_or_default(),
                status: row.get("status"),
            }))
        } else {
            Ok(None)
        }
    }

    // Release escrow (order completed or cancelled)
    pub async fn release_escrow(&self, checkout_token: &str, completed: bool) -> Result<()> {
        let status = if completed { "completed" } else { "cancelled" };

        sqlx::query("UPDATE points_escrow SET status = ?1 WHERE checkout_token = ?2")
            .bind(status)
            .bind(checkout_token)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Cleanup expired escrows
    pub async fn cleanup_expired_escrows(&self) -> Result<u64> {
        let result = sqlx::query(
            "UPDATE points_escrow SET status = 'expired' 
             WHERE status = 'pending' AND expires_at < ?1",
        )
        .bind(Utc::now().timestamp())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    // Record redemption
    pub async fn record_redemption(
        &self,
        customer_id: &str,
        points: i64,
        discount_code: &str,
        location: Option<String>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO loyalty_redemptions 
            (customer_id, points_redeemed, discount_code, location, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(customer_id)
        .bind(points)
        .bind(discount_code)
        .bind(location)
        .bind(Utc::now().timestamp())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Update redemption with order ID
    pub async fn update_redemption_order_id(
        &self,
        discount_code: &str,
        order_id: &str,
    ) -> Result<()> {
        sqlx::query("UPDATE loyalty_redemptions SET order_id = ?1 WHERE discount_code = ?2")
            .bind(order_id)
            .bind(discount_code)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Update cached points (quick update after redemption)
    pub async fn update_cached_points(&self, customer_id: &str, new_points: i64) -> Result<()> {
        sqlx::query("UPDATE loyalty_cache SET points = ?1 WHERE customer_id = ?2")
            .bind(new_points)
            .bind(customer_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Cleanup old data
    pub async fn cleanup_old_data(&self, days_to_keep: i64) -> Result<()> {
        let cutoff = (Utc::now() - Duration::days(days_to_keep)).timestamp();

        // Clean old cache entries
        sqlx::query("DELETE FROM loyalty_cache WHERE expires_at < ?1")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;

        // Clean old completed redemptions
        sqlx::query(
            "DELETE FROM loyalty_redemptions 
             WHERE created_at < ?1 AND order_id IS NOT NULL",
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        // Clean old completed/cancelled escrows
        sqlx::query(
            "DELETE FROM points_escrow 
             WHERE created_at < ?1 AND status != 'pending'",
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Get the first active shop (for API calls)
    pub async fn get_active_shop(
        &self,
    ) -> Result<crate::sqlite::shopifyFunctionality::ShopifyStore> {
        let row = sqlx::query("SELECT * FROM shopify_stores WHERE is_active = 1 LIMIT 1")
            .fetch_one(&self.pool)
            .await?;

        Ok(crate::sqlite::shopifyFunctionality::ShopifyStore {
            id: Some(row.get("id")),
            shop_domain: row.get("shop_domain"),
            access_token: row.get("access_token"),
            scope: row.get("scope"),
            installed_at: DateTime::from_timestamp(row.get("installed_at"), 0).unwrap_or_default(),
            updated_at: DateTime::from_timestamp(row.get("updated_at"), 0).unwrap_or_default(),
            is_active: row.get("is_active"),
        })
    }
}
