use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::models::{MintOutcome, MintRequest, MintStatus, Quota, Role, User};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn upsert_user(&self, user: &User) -> anyhow::Result<()>;
    async fn find_user(&self, channel: &str, handle: &str) -> anyhow::Result<Option<User>>;
    async fn set_role(&self, user_id: Uuid, role: Role) -> anyhow::Result<()>;
}

#[async_trait]
pub trait MintRepository: Send + Sync {
    async fn enqueue(&self, request: &MintRequest) -> anyhow::Result<()>;
    async fn next_pending(&self) -> anyhow::Result<Option<MintRequest>>;
    async fn update_status(&self, request_id: Uuid, status: MintStatus) -> anyhow::Result<()>;
    async fn record_outcome(&self, outcome: &MintOutcome) -> anyhow::Result<()>;
}

#[async_trait]
pub trait QuotaRepository: Send + Sync {
    async fn record_mint(&self, user_id: Uuid, day: NaiveDate, amount: u64) -> anyhow::Result<()>;
    async fn fetch_quota(&self, user_id: Uuid, day: NaiveDate) -> anyhow::Result<Option<Quota>>;
}

#[async_trait]
pub trait ReportingRepository: Send + Sync {
    async fn daily_summary(&self, day: NaiveDate) -> anyhow::Result<Vec<DailyReportRow>>;
    async fn log_failure(
        &self,
        request_id: Uuid,
        when: DateTime<Utc>,
        reason: &str,
    ) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct DailyReportRow {
    pub channel: String,
    pub total_amount: u64,
    pub success_count: u64,
    pub failure_count: u64,
}
