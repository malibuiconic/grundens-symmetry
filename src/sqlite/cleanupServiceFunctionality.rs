use crate::Database;
use anyhow::Result;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{interval, Duration as TokioDuration};

pub struct CleanupService {
    db: Arc<Database>,
    config: CleanupConfig,
}

#[derive(Clone)]
pub struct CleanupConfig {
    /// How often to run cleanup (in seconds)
    pub cleanup_interval_seconds: u64,
    /// Maximum age for expired tokens (in minutes)
    pub max_token_age_minutes: i64,
    /// How long to keep completed redemptions (in days)
    pub keep_redemptions_days: i64,
    /// Maximum number of tokens per customer (prevent spam)
    pub max_tokens_per_customer: usize,
    /// How often to log cleanup stats
    pub log_stats_every_n_runs: u32,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            cleanup_interval_seconds: 60, // Run every minute
            max_token_age_minutes: 60,    // Keep tokens for 1 hour max
            keep_redemptions_days: 30,    // Keep redemption history for 30 days
            max_tokens_per_customer: 3,   // Max 3 active tokens per customer
            log_stats_every_n_runs: 10,   // Log stats every 10 runs (10 minutes)
        }
    }
}

#[derive(Debug, Default)]
pub struct CleanupStats {
    pub expired_tokens_removed: u64,
    pub duplicate_tokens_removed: u64,
    pub old_redemptions_removed: u64,
    pub expired_escrows_updated: u64,
    pub total_runs: u32,
    pub last_run: Option<chrono::DateTime<Utc>>,
}

impl CleanupService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            config: CleanupConfig::default(),
        }
    }

    pub fn with_config(db: Arc<Database>, config: CleanupConfig) -> Self {
        Self { db, config }
    }

    /// Start the cleanup service as a background task
    pub async fn start(&self) -> Result<()> {
        let mut cleanup_interval = interval(TokioDuration::from_secs(
            self.config.cleanup_interval_seconds,
        ));
        let mut stats = CleanupStats::default();

        info!(
            "🧹 Cleanup service started - running every {} seconds",
            self.config.cleanup_interval_seconds
        );

        loop {
            cleanup_interval.tick().await;

            match self.run_cleanup_cycle(&mut stats).await {
                Ok(_) => {
                    stats.total_runs += 1;
                    stats.last_run = Some(Utc::now());

                    // Log stats periodically
                    if stats.total_runs % self.config.log_stats_every_n_runs == 0 {
                        self.log_cleanup_stats(&stats);
                    }
                }
                Err(e) => {
                    error!("❌ Cleanup cycle failed: {}", e);
                }
            }
        }
    }

    /// Run a single cleanup cycle
    async fn run_cleanup_cycle(&self, stats: &mut CleanupStats) -> Result<()> {
        let start_time = Utc::now();

        // 1. Remove expired tokens
        let expired_removed = self.cleanup_expired_tokens().await?;
        stats.expired_tokens_removed += expired_removed;

        // 2. Remove duplicate tokens per customer (keep newest)
        let duplicates_removed = self.cleanup_duplicate_tokens().await?;
        stats.duplicate_tokens_removed += duplicates_removed;

        // 3. Update expired escrows
        let escrows_updated = self.db.cleanup_expired_escrows().await?;
        stats.expired_escrows_updated += escrows_updated;

        // 4. Remove old redemption history
        let redemptions_removed = self.cleanup_old_redemptions().await?;
        stats.old_redemptions_removed += redemptions_removed;

        // 5. Vacuum database periodically (every 100 runs)
        if stats.total_runs % 100 == 0 {
            self.vacuum_database().await?;
        }

        let duration = Utc::now().signed_duration_since(start_time);
        debug!(
            "🧹 Cleanup cycle completed in {}ms",
            duration.num_milliseconds()
        );

        Ok(())
    }

    /// Remove tokens that have expired
    async fn cleanup_expired_tokens(&self) -> Result<u64> {
        let cutoff_time = Utc::now().timestamp();

        let result = sqlx::query("DELETE FROM loyalty_cache WHERE expires_at < ?1")
            .bind(cutoff_time)
            .execute(&self.db.pool)
            .await?;

        let removed = result.rows_affected();
        if removed > 0 {
            debug!("🗑️ Removed {} expired tokens", removed);
        }

        Ok(removed)
    }

    /// Remove duplicate tokens per customer, keeping only the newest ones
    async fn cleanup_duplicate_tokens(&self) -> Result<u64> {
        // Get customer IDs with multiple active tokens
        let customers_with_multiple_tokens: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT customer_id 
            FROM loyalty_cache 
            WHERE expires_at > ?1
            GROUP BY customer_id 
            HAVING COUNT(*) > ?2
            "#,
        )
        .bind(Utc::now().timestamp())
        .bind(self.config.max_tokens_per_customer as i32)
        .fetch_all(&self.db.pool)
        .await?;

        let mut total_removed = 0u64;

        for customer_id in customers_with_multiple_tokens {
            // Get all tokens for this customer, ordered by creation time (newest first)
            let tokens: Vec<String> = sqlx::query_scalar(
                r#"
                SELECT token 
                FROM loyalty_cache 
                WHERE customer_id = ?1 AND expires_at > ?2
                ORDER BY created_at DESC
                "#,
            )
            .bind(&customer_id)
            .bind(Utc::now().timestamp())
            .fetch_all(&self.db.pool)
            .await?;

            // Keep only the newest N tokens, remove the rest
            if tokens.len() > self.config.max_tokens_per_customer {
                let tokens_to_remove = &tokens[self.config.max_tokens_per_customer..];

                for token in tokens_to_remove {
                    let result = sqlx::query("DELETE FROM loyalty_cache WHERE token = ?1")
                        .bind(token)
                        .execute(&self.db.pool)
                        .await?;

                    total_removed += result.rows_affected();
                }

                debug!(
                    "🗑️ Removed {} duplicate tokens for customer {}",
                    tokens_to_remove.len(),
                    customer_id
                );
            }
        }

        Ok(total_removed)
    }

    /// Remove old redemption history
    async fn cleanup_old_redemptions(&self) -> Result<u64> {
        let cutoff_time =
            (Utc::now() - Duration::days(self.config.keep_redemptions_days)).timestamp();

        let result = sqlx::query(
            "DELETE FROM loyalty_redemptions WHERE created_at < ?1 AND order_id IS NOT NULL",
        )
        .bind(cutoff_time)
        .execute(&self.db.pool)
        .await?;

        let removed = result.rows_affected();
        if removed > 0 {
            debug!("🗑️ Removed {} old redemption records", removed);
        }

        Ok(removed)
    }

    /// Vacuum database to reclaim space
    async fn vacuum_database(&self) -> Result<()> {
        info!("🧽 Running database VACUUM...");
        let start_time = Utc::now();

        sqlx::query("VACUUM").execute(&self.db.pool).await?;

        let duration = Utc::now().signed_duration_since(start_time);
        info!(
            "✅ Database VACUUM completed in {}ms",
            duration.num_milliseconds()
        );

        Ok(())
    }

    /// Get current cleanup statistics
    pub async fn get_stats(&self) -> Result<CleanupStats> {
        // Get current token count
        let active_tokens: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM loyalty_cache WHERE expires_at > ?1")
                .bind(Utc::now().timestamp())
                .fetch_one(&self.db.pool)
                .await?;

        // Get pending escrows
        let pending_escrows: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM points_escrow WHERE status = 'pending' AND expires_at > ?1",
        )
        .bind(Utc::now().timestamp())
        .fetch_one(&self.db.pool)
        .await?;

        // Get recent redemptions
        let recent_redemptions: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM loyalty_redemptions WHERE created_at > ?1")
                .bind((Utc::now() - Duration::days(7)).timestamp())
                .fetch_one(&self.db.pool)
                .await?;

        info!(
            "📊 Current state: {} active tokens, {} pending escrows, {} recent redemptions",
            active_tokens, pending_escrows, recent_redemptions
        );

        // Return basic stats - in a real implementation you'd want to persist these
        Ok(CleanupStats::default())
    }

    /// Log cleanup statistics
    fn log_cleanup_stats(&self, stats: &CleanupStats) {
        info!("📈 Cleanup Stats Summary:");
        info!(
            "  🗑️ Expired tokens removed: {}",
            stats.expired_tokens_removed
        );
        info!(
            "  🔄 Duplicate tokens removed: {}",
            stats.duplicate_tokens_removed
        );
        info!(
            "  ⏰ Expired escrows updated: {}",
            stats.expired_escrows_updated
        );
        info!(
            "  📝 Old redemptions removed: {}",
            stats.old_redemptions_removed
        );
        info!("  🏃 Total cleanup runs: {}", stats.total_runs);
        if let Some(last_run) = stats.last_run {
            info!(
                "  ⏱️ Last run: {}",
                last_run.format("%Y-%m-%d %H:%M:%S UTC")
            );
        }
    }

    /// Force cleanup now (useful for testing or manual triggers)
    pub async fn force_cleanup(&self) -> Result<CleanupStats> {
        let mut stats = CleanupStats::default();
        self.run_cleanup_cycle(&mut stats).await?;
        self.log_cleanup_stats(&stats);
        Ok(stats)
    }

    /// Get token deduplication suggestions for a customer
    pub async fn get_customer_token_info(
        &self,
        customer_id: &str,
    ) -> Result<Vec<CustomerTokenInfo>> {
        let tokens: Vec<CustomerTokenInfo> = sqlx::query_as(
            r#"
            SELECT 
                token,
                customer_id,
                email,
                netsuite_id,
                points,
                created_at,
                updated_at,
                expires_at,
                CASE 
                    WHEN points IS NOT NULL THEN 'HAS_POINTS'
                    ELSE 'PENDING'
                END as status
            FROM loyalty_cache 
            WHERE customer_id = ?1 AND expires_at > ?2
            ORDER BY created_at DESC
            "#,
        )
        .bind(customer_id)
        .bind(Utc::now().timestamp())
        .fetch_all(&self.db.pool)
        .await?;

        Ok(tokens)
    }

    /// Session-based token deduplication - get or create a single token for customer session
    pub async fn get_or_create_session_token(
        &self,
        customer_id: &str,
        email: &str,
        netsuite_id: &str,
        session_ttl_minutes: i64,
    ) -> Result<String> {
        // First, try to find an existing valid token for this customer
        let existing_token: Option<String> = sqlx::query_scalar(
            "SELECT token FROM loyalty_cache 
             WHERE customer_id = ?1 AND expires_at > ?2 
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(customer_id)
        .bind(Utc::now().timestamp())
        .fetch_optional(&self.db.pool)
        .await?;

        if let Some(token) = existing_token {
            debug!("♻️ Reusing existing token for customer: {}", customer_id);
            return Ok(token);
        }

        // No existing token, create a new one
        let token = uuid::Uuid::new_v4().to_string();
        self.db
            .create_token_entry(&token, customer_id, email, netsuite_id)
            .await?;

        debug!("🆕 Created new session token for customer: {}", customer_id);
        Ok(token)
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct CustomerTokenInfo {
    pub token: String,
    pub customer_id: String,
    pub email: String,
    pub netsuite_id: String,
    pub points: Option<f64>,
    pub created_at: i64,
    pub updated_at: Option<i64>,
    pub expires_at: i64,
    pub status: String,
}

/// Helper function to start cleanup service in main.rs
pub async fn start_cleanup_service(db: Arc<Database>) -> Result<()> {
    let cleanup_service = CleanupService::new(db);

    // Start cleanup service in background
    tokio::spawn(async move {
        if let Err(e) = cleanup_service.start().await {
            error!("💥 Cleanup service crashed: {}", e);
        }
    });

    Ok(())
}
