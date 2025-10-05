use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry,
};

use crate::config::TelemetryConfig;

pub fn init_telemetry(config: &TelemetryConfig) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,hyper=warn,sqlx=warn"));

    let fmt_layer = if config.json {
        fmt::layer().json().with_current_span(true).boxed()
    } else {
        fmt::layer().with_target(false).boxed()
    };

    let subscriber = Registry::default().with(filter).with(fmt_layer);

    subscriber.init();

    if let Some(_endpoint) = &config.otlp_endpoint {
        tracing::warn!("OTLP 导出尚未实现");
    }
}
