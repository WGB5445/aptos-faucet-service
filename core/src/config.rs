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

        let config: AppConfig = builder.build()?.try_deserialize()?;
        
        // 验证必需的配置
        config.validate()?;
        
        Ok(config)
    }
    
    fn validate(&self) -> Result<(), config::ConfigError> {
        // 检查是否跳过数据库验证
        let skip_db = std::env::args().any(|arg| arg == "--no-db") ||
            std::env::var("FAUCET_NO_DB")
                .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
                .unwrap_or(false);
        
        // 验证数据库配置（除非跳过数据库）
        if !skip_db {
            match &self.database {
                DatabaseConfig::Postgres { url } if url.is_empty() => {
                    return Err(config::ConfigError::Message(
                        "数据库 URL 不能为空，请设置 FAUCET__DATABASE__URL 环境变量".to_string()
                    ));
                }
                DatabaseConfig::Mongodb { url, .. } if url.is_empty() => {
                    return Err(config::ConfigError::Message(
                        "MongoDB URL 不能为空，请设置 FAUCET__DATABASE__URL 环境变量".to_string()
                    ));
                }
                _ => {}
            }
        }
        
        // 验证 OAuth 配置
        if self.auth.google_client_id.is_empty() {
            return Err(config::ConfigError::Message(
                "Google Client ID 不能为空，请设置 FAUCET__AUTH__GOOGLE_CLIENT_ID 环境变量".to_string()
            ));
        }
        
        // 注意：google_client_secret 不是必需的，因为后端只验证ID token
        // 如果需要服务器端OAuth流程，可以取消下面的注释
        // if self.auth.google_client_secret.is_empty() {
        //     return Err(config::ConfigError::Message(
        //         "Google Client Secret 不能为空，请设置 FAUCET__AUTH__GOOGLE_CLIENT_SECRET 环境变量".to_string()
        //     ));
        // }
        
        Ok(())
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
