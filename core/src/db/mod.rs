use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::{
    config::DatabaseConfig,
    models::{MintOutcome, MintRequest, MintStatus, Quota, Role, User},
    repository::{
        DailyReportRow, MintRepository, QuotaRepository, ReportingRepository, UserRepository, ConfigRepository,
    },
};

#[derive(Clone)]
pub enum DatabaseStore {
    #[cfg(feature = "postgres")]
    Postgres(crate::db::postgres::PostgresStore),
    #[cfg(feature = "mongodb")]
    Mongo(crate::db::mongodb::MongoStore),
    Memory(crate::db::memory::MemoryStore),
}

impl DatabaseStore {
    pub async fn connect(config: &DatabaseConfig) -> Result<Self> {
        match config {
            #[cfg(feature = "postgres")]
            DatabaseConfig::Postgres { url } => {
                let store = crate::db::postgres::PostgresStore::connect(url).await?;
                Ok(Self::Postgres(store))
            }
            #[cfg(feature = "mongodb")]
            DatabaseConfig::Mongodb { url, database } => {
                let store = crate::db::mongodb::MongoStore::connect(url, database).await?;
                Ok(Self::Mongo(store))
            }
            #[cfg(not(feature = "postgres"))]
            DatabaseConfig::Postgres { .. } => {
                anyhow::bail!("Postgres feature is disabled");
            }
            #[cfg(not(feature = "mongodb"))]
            DatabaseConfig::Mongodb { .. } => {
                anyhow::bail!("MongoDB feature is disabled");
            }
        }
    }

    pub fn memory() -> Self {
        Self::Memory(crate::db::memory::MemoryStore::new())
    }
}

#[async_trait]
impl UserRepository for DatabaseStore {
    async fn upsert_user(&self, user: &User) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.upsert_user(user).await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.upsert_user(user).await,
            DatabaseStore::Memory(store) => store.upsert_user(user).await,
        }
    }

    async fn find_user(&self, channel: &str, handle: &str) -> anyhow::Result<Option<User>> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.find_user(channel, handle).await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.find_user(channel, handle).await,
            DatabaseStore::Memory(store) => store.find_user(channel, handle).await,
        }
    }

    async fn set_role(&self, user_id: uuid::Uuid, role: Role) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.set_role(user_id, role).await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.set_role(user_id, role).await,
            DatabaseStore::Memory(store) => store.set_role(user_id, role).await,
        }
    }
}

#[async_trait]
impl MintRepository for DatabaseStore {
    async fn enqueue(&self, request: &MintRequest) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.enqueue(request).await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.enqueue(request).await,
            DatabaseStore::Memory(store) => store.enqueue(request).await,
        }
    }

    async fn next_pending(&self) -> anyhow::Result<Option<MintRequest>> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.next_pending().await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.next_pending().await,
            DatabaseStore::Memory(store) => store.next_pending().await,
        }
    }

    async fn update_status(
        &self,
        request_id: uuid::Uuid,
        status: MintStatus,
    ) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.update_status(request_id, status).await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.update_status(request_id, status).await,
            DatabaseStore::Memory(store) => store.update_status(request_id, status).await,
        }
    }

    async fn record_outcome(&self, outcome: &MintOutcome) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.record_outcome(outcome).await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.record_outcome(outcome).await,
            DatabaseStore::Memory(store) => store.record_outcome(outcome).await,
        }
    }
}

#[async_trait]
impl QuotaRepository for DatabaseStore {
    async fn record_mint(
        &self,
        user_id: uuid::Uuid,
        day: chrono::NaiveDate,
        amount: u64,
    ) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.record_mint(user_id, day, amount).await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.record_mint(user_id, day, amount).await,
            DatabaseStore::Memory(store) => store.record_mint(user_id, day, amount).await,
        }
    }

    async fn fetch_quota(
        &self,
        user_id: uuid::Uuid,
        day: chrono::NaiveDate,
    ) -> anyhow::Result<Option<Quota>> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.fetch_quota(user_id, day).await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.fetch_quota(user_id, day).await,
            DatabaseStore::Memory(store) => store.fetch_quota(user_id, day).await,
        }
    }
}

#[async_trait]
impl ReportingRepository for DatabaseStore {
    async fn daily_summary(&self, day: chrono::NaiveDate) -> anyhow::Result<Vec<DailyReportRow>> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.daily_summary(day).await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.daily_summary(day).await,
            DatabaseStore::Memory(store) => store.daily_summary(day).await,
        }
    }

    async fn log_failure(
        &self,
        request_id: uuid::Uuid,
        when: chrono::DateTime<chrono::Utc>,
        reason: &str,
    ) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.log_failure(request_id, when, reason).await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.log_failure(request_id, when, reason).await,
            DatabaseStore::Memory(store) => store.log_failure(request_id, when, reason).await,
        }
    }
}

#[async_trait]
impl<T> UserRepository for Arc<T>
where
    T: UserRepository + ?Sized,
{
    async fn upsert_user(&self, user: &User) -> anyhow::Result<()> {
        (**self).upsert_user(user).await
    }

    async fn find_user(&self, channel: &str, handle: &str) -> anyhow::Result<Option<User>> {
        (**self).find_user(channel, handle).await
    }

    async fn set_role(&self, user_id: uuid::Uuid, role: Role) -> anyhow::Result<()> {
        (**self).set_role(user_id, role).await
    }
}

#[async_trait]
impl<T> MintRepository for Arc<T>
where
    T: MintRepository + ?Sized,
{
    async fn enqueue(&self, request: &MintRequest) -> anyhow::Result<()> {
        (**self).enqueue(request).await
    }

    async fn next_pending(&self) -> anyhow::Result<Option<MintRequest>> {
        (**self).next_pending().await
    }

    async fn update_status(
        &self,
        request_id: uuid::Uuid,
        status: MintStatus,
    ) -> anyhow::Result<()> {
        (**self).update_status(request_id, status).await
    }

    async fn record_outcome(&self, outcome: &MintOutcome) -> anyhow::Result<()> {
        (**self).record_outcome(outcome).await
    }
}

#[async_trait]
impl<T> QuotaRepository for Arc<T>
where
    T: QuotaRepository + ?Sized,
{
    async fn record_mint(
        &self,
        user_id: uuid::Uuid,
        day: chrono::NaiveDate,
        amount: u64,
    ) -> anyhow::Result<()> {
        (**self).record_mint(user_id, day, amount).await
    }

    async fn fetch_quota(
        &self,
        user_id: uuid::Uuid,
        day: chrono::NaiveDate,
    ) -> anyhow::Result<Option<Quota>> {
        (**self).fetch_quota(user_id, day).await
    }
}

#[async_trait]
impl<T> ReportingRepository for Arc<T>
where
    T: ReportingRepository + ?Sized,
{
    async fn daily_summary(&self, day: chrono::NaiveDate) -> anyhow::Result<Vec<DailyReportRow>> {
        (**self).daily_summary(day).await
    }

    async fn log_failure(
        &self,
        request_id: uuid::Uuid,
        when: chrono::DateTime<chrono::Utc>,
        reason: &str,
    ) -> anyhow::Result<()> {
        (**self).log_failure(request_id, when, reason).await
    }
}

#[async_trait]
impl ConfigRepository for DatabaseStore {
    async fn get_config(&self, key: &str) -> anyhow::Result<Option<crate::models::SystemConfig>> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.get_config(key).await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.get_config(key).await,
            DatabaseStore::Memory(store) => store.get_config(key).await,
        }
    }

    async fn set_config(&self, key: &str, value: &str, description: Option<&str>) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.set_config(key, value, description).await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.set_config(key, value, description).await,
            DatabaseStore::Memory(store) => store.set_config(key, value, description).await,
        }
    }

    async fn get_all_configs(&self) -> anyhow::Result<Vec<crate::models::SystemConfig>> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.get_all_configs().await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.get_all_configs().await,
            DatabaseStore::Memory(store) => store.get_all_configs().await,
        }
    }

    async fn update_limit_config(&self, config: &crate::models::LimitConfigUpdate) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.update_limit_config(config).await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.update_limit_config(config).await,
            DatabaseStore::Memory(store) => store.update_limit_config(config).await,
        }
    }

    async fn get_limit_config(&self) -> anyhow::Result<Option<crate::models::LimitConfigUpdate>> {
        match self {
            #[cfg(feature = "postgres")]
            DatabaseStore::Postgres(store) => store.get_limit_config().await,
            #[cfg(feature = "mongodb")]
            DatabaseStore::Mongo(store) => store.get_limit_config().await,
            DatabaseStore::Memory(store) => store.get_limit_config().await,
        }
    }
}

pub mod memory;
#[cfg(feature = "mongodb")]
pub mod mongodb;
#[cfg(feature = "postgres")]
pub mod postgres;
