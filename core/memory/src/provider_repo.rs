use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Provider plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProviderPlugin {
    pub id: String,
    pub provider_type: String,
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub default_model: Option<String>,
    pub config: Option<String>,
    pub enabled: i32,
    pub priority: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Session fast mode state
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SessionFastMode {
    pub id: String,
    pub session_id: String,
    pub fast_enabled: i32,
    pub fast_model: Option<String>,
    pub fast_config: Option<String>,
    pub toggles: Option<String>,
    pub updated_at: String,
}

/// Model fallback chain
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ModelFallback {
    pub id: String,
    pub primary_model: String,
    pub fallback_model: String,
    pub provider: Option<String>,
    pub priority: i32,
    pub created_at: String,
}

/// Provider model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProviderModel {
    pub id: String,
    pub provider_id: String,
    pub model_id: String,
    pub model_name: String,
    pub context_length: Option<i32>,
    pub supports_vision: i32,
    pub supports_tools: i32,
    pub pricing_input: Option<f64>,
    pub pricing_output: Option<f64>,
    pub last_verified: String,
}

/// Provider health status
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProviderHealth {
    pub id: String,
    pub provider_id: String,
    pub status: String,
    pub latency_ms: Option<i32>,
    pub last_check: String,
    pub error_message: Option<String>,
    pub consecutive_failures: i32,
}

/// Provider plugins repository
pub struct ProviderRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> ProviderRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    // ============ Provider Plugins ============

    pub async fn list_providers(
        &self,
        provider_type: Option<&str>,
        enabled_only: bool,
    ) -> Result<Vec<ProviderPlugin>, sqlx::Error> {
        let sql = match (provider_type, enabled_only) {
            (Some(_pt), true) => {
                "SELECT * FROM provider_plugins WHERE provider_type = ? AND enabled = 1 ORDER BY priority DESC"
            }
            (Some(_pt), false) => {
                "SELECT * FROM provider_plugins WHERE provider_type = ? ORDER BY priority DESC"
            }
            (None, true) => {
                "SELECT * FROM provider_plugins WHERE enabled = 1 ORDER BY priority DESC"
            }
            (None, false) => {
                "SELECT * FROM provider_plugins ORDER BY priority DESC"
            }
        };

        match provider_type {
            Some(pt) => {
                sqlx::query_as::<_, ProviderPlugin>(sql)
                    .bind(pt)
                    .fetch_all(self.pool)
                    .await
            }
            None => {
                sqlx::query_as::<_, ProviderPlugin>(sql)
                    .fetch_all(self.pool)
                    .await
            }
        }
    }

    pub async fn get_provider(&self, id: &str) -> Result<ProviderPlugin, sqlx::Error> {
        sqlx::query_as::<_, ProviderPlugin>("SELECT * FROM provider_plugins WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn create_provider(
        &self,
        id: &str,
        provider_type: &str,
        name: &str,
        base_url: &str,
        api_key: Option<&str>,
        default_model: Option<&str>,
        config: Option<&str>,
    ) -> Result<ProviderPlugin, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO provider_plugins (id, provider_type, name, base_url, api_key, default_model, config, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(provider_type)
        .bind(name)
        .bind(base_url)
        .bind(api_key)
        .bind(default_model)
        .bind(config)
        .bind(&now)
        .bind(&now)
        .execute(self.pool)
        .await?;

        self.get_provider(id).await
    }

    pub async fn update_provider(
        &self,
        id: &str,
        name: Option<&str>,
        base_url: Option<&str>,
        api_key: Option<&str>,
        default_model: Option<&str>,
        config: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<ProviderPlugin, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE provider_plugins SET
                name = COALESCE(?, name),
                base_url = COALESCE(?, base_url),
                api_key = COALESCE(?, api_key),
                default_model = COALESCE(?, default_model),
                config = COALESCE(?, config),
                enabled = COALESCE(?, enabled),
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(name)
        .bind(base_url)
        .bind(api_key)
        .bind(default_model)
        .bind(config)
        .bind(enabled.map(|e| if e { 1 } else { 0 }))
        .bind(&now)
        .bind(id)
        .execute(self.pool)
        .await?;

        self.get_provider(id).await
    }

    pub async fn delete_provider(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM provider_plugins WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    // ============ Session Fast Mode ============

    pub async fn get_session_fast_mode(
        &self,
        session_id: &str,
    ) -> Result<Option<SessionFastMode>, sqlx::Error> {
        sqlx::query_as::<_, SessionFastMode>("SELECT * FROM session_fast_mode WHERE session_id = ?")
            .bind(session_id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn upsert_session_fast_mode(
        &self,
        id: &str,
        session_id: &str,
        fast_enabled: bool,
        fast_model: Option<&str>,
        fast_config: Option<&str>,
        toggles: Option<&str>,
    ) -> Result<SessionFastMode, sqlx::Error> {
        let now = Utc::now();
        let enabled = if fast_enabled { 1 } else { 0 };

        sqlx::query(
            r#"
            INSERT INTO session_fast_mode (id, session_id, fast_enabled, fast_model, fast_config, toggles, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(session_id) DO UPDATE SET
                fast_enabled = excluded.fast_enabled,
                fast_model = COALESCE(excluded.fast_model, session_fast_mode.fast_model),
                fast_config = COALESCE(excluded.fast_config, session_fast_mode.fast_config),
                toggles = COALESCE(excluded.toggles, session_fast_mode.toggles),
                updated_at = excluded.updated_at
            "#,
        )
        .bind(id)
        .bind(session_id)
        .bind(enabled)
        .bind(fast_model)
        .bind(fast_config)
        .bind(toggles)
        .bind(&now)
        .execute(self.pool)
        .await?;

        self.get_session_fast_mode(session_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    // ============ Model Fallbacks ============

    pub async fn list_fallbacks(
        &self,
        primary_model: Option<&str>,
    ) -> Result<Vec<ModelFallback>, sqlx::Error> {
        match primary_model {
            Some(model) => {
                sqlx::query_as::<_, ModelFallback>(
                    "SELECT * FROM model_fallbacks WHERE primary_model = ? ORDER BY priority ASC",
                )
                .bind(model)
                .fetch_all(self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, ModelFallback>(
                    "SELECT * FROM model_fallbacks ORDER BY primary_model, priority ASC",
                )
                .fetch_all(self.pool)
                .await
            }
        }
    }

    pub async fn add_fallback(
        &self,
        id: &str,
        primary_model: &str,
        fallback_model: &str,
        provider: Option<&str>,
        priority: i32,
    ) -> Result<ModelFallback, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO model_fallbacks (id, primary_model, fallback_model, provider, priority, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(primary_model)
        .bind(fallback_model)
        .bind(provider)
        .bind(priority)
        .bind(&now)
        .execute(self.pool)
        .await?;

        sqlx::query_as::<_, ModelFallback>("SELECT * FROM model_fallbacks WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
    }

    pub async fn delete_fallback(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM model_fallbacks WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    // ============ Provider Models ============

    pub async fn list_provider_models(
        &self,
        provider_id: &str,
    ) -> Result<Vec<ProviderModel>, sqlx::Error> {
        sqlx::query_as::<_, ProviderModel>(
            "SELECT * FROM provider_models WHERE provider_id = ? ORDER BY model_name",
        )
        .bind(provider_id)
        .fetch_all(self.pool)
        .await
    }

    pub async fn add_provider_model(
        &self,
        id: &str,
        provider_id: &str,
        model_id: &str,
        model_name: &str,
        context_length: Option<i32>,
    ) -> Result<ProviderModel, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO provider_models (id, provider_id, model_id, model_name, context_length, last_verified)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(provider_id)
        .bind(model_id)
        .bind(model_name)
        .bind(context_length)
        .bind(&now)
        .execute(self.pool)
        .await?;

        sqlx::query_as::<_, ProviderModel>("SELECT * FROM provider_models WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
    }

    // ============ Provider Health ============

    pub async fn get_provider_health(
        &self,
        provider_id: &str,
    ) -> Result<Option<ProviderHealth>, sqlx::Error> {
        sqlx::query_as::<_, ProviderHealth>("SELECT * FROM provider_health WHERE provider_id = ?")
            .bind(provider_id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn update_provider_health(
        &self,
        provider_id: &str,
        status: &str,
        latency_ms: Option<i32>,
        error_message: Option<&str>,
    ) -> Result<ProviderHealth, sqlx::Error> {
        let now = Utc::now();
        let id = format!("health_{}", provider_id);

        // Get current failures
        let current = self.get_provider_health(provider_id).await?;
        let failures = match current {
            Some(h) => {
                if status == "healthy" {
                    0
                } else {
                    h.consecutive_failures + 1
                }
            }
            None => 1,
        };

        sqlx::query(
            r#"
            INSERT INTO provider_health (id, provider_id, status, latency_ms, last_check, error_message, consecutive_failures)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(provider_id) DO UPDATE SET
                status = excluded.status,
                latency_ms = COALESCE(excluded.latency_ms, provider_health.latency_ms),
                last_check = excluded.last_check,
                error_message = excluded.error_message,
                consecutive_failures = excluded.consecutive_failures
            "#,
        )
        .bind(&id)
        .bind(provider_id)
        .bind(status)
        .bind(latency_ms)
        .bind(&now)
        .bind(error_message)
        .bind(failures)
        .execute(self.pool)
        .await?;

        self.get_provider_health(provider_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }
}
