use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub limits: LimitConfig,
    pub auth: AuthConfig,
    pub queue: QueueConfig,
    pub database: DatabaseConfig,
    pub telemetry: TelemetryConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::Environment::with_prefix("FAUCET").separator("__"));

        builder.build()?.try_deserialize()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub http_addr: String,
    pub public_base_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LimitConfig {
    pub default_amount: u64,
    pub default_daily_cap: u64,
    pub privileged_amount: u64,
    pub privileged_daily_cap: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub privileged_domains: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QueueConfig {
    #[serde(with = "humantime_serde")]
    pub visibility_timeout: Duration,
    #[serde(with = "humantime_serde")]
    pub retry_backoff: Duration,
    pub max_retries: u16,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DatabaseConfig {
    Postgres { url: String },
    Mongodb { url: String, database: String },
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelemetryConfig {
    pub json: bool,
    pub otlp_endpoint: Option<String>,
}
