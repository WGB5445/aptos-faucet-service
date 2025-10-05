use anyhow::Result;
use chrono::{Duration, Utc};
use faucet_core::{
    config::AppConfig,
    logging,
    repository::{DailyReportRow, ReportingRepository},
    DatabaseStore,
};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;
    logging::init_telemetry(&config.telemetry);

    let skip_db = should_skip_db();
    let store = if skip_db {
        warn!("数据库连接已跳过，报表任务使用内存存储，数据不会持久化");
        DatabaseStore::memory()
    } else {
        DatabaseStore::connect(&config.database).await?
    };

    let today = Utc::now().date_naive();
    let yesterday = today - Duration::days(1);

    info!(day = %today, "开始生成日报");
    let today_rows = store.daily_summary(today).await?;
    render_report("今日", &today_rows);

    let yesterday_rows = store.daily_summary(yesterday).await?;
    render_report("昨日", &yesterday_rows);

    Ok(())
}

fn should_skip_db() -> bool {
    if std::env::args().any(|arg| arg == "--no-db") {
        return true;
    }

    if let Ok(value) = std::env::var("FAUCET_NO_DB") {
        let value = value.to_ascii_lowercase();
        return matches!(value.as_str(), "1" | "true" | "yes");
    }

    false
}

fn render_report(title: &str, rows: &[DailyReportRow]) {
    info!(title, entries = rows.len(), "汇总统计");
    for row in rows {
        info!(
            channel = %row.channel,
            total_amount = row.total_amount,
            success = row.success_count,
            failure = row.failure_count,
            "channel_summary"
        );
    }
}
