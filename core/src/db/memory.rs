use std::{collections::VecDeque, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use dashmap::DashMap;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::{MintOutcome, MintRequest, MintStatus, Quota, Role, User, SystemConfig, LimitConfigUpdate};
use crate::repository::{
    DailyReportRow, MintRepository, QuotaRepository, ReportingRepository, UserRepository, ConfigRepository,
};

#[derive(Clone, Default)]
pub struct MemoryStore {
    users: Arc<DashMap<(String, String), User>>, // (channel, handle)
    mints: Arc<DashMap<Uuid, MintRequest>>,
    queue: Arc<Mutex<VecDeque<Uuid>>>,
    quotas: Arc<DashMap<(Uuid, NaiveDate), Quota>>,
    configs: Arc<DashMap<String, SystemConfig>>,
    failures: Arc<Mutex<Vec<(Uuid, DateTime<Utc>, String)>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn key(channel: &str, handle: &str) -> (String, String) {
        (channel.to_ascii_lowercase(), handle.to_ascii_lowercase())
    }
}

#[async_trait]
impl UserRepository for MemoryStore {
    async fn upsert_user(&self, user: &User) -> Result<()> {
        let key = Self::key(user.channel.as_str(), &user.handle);
        self.users.insert(key, user.clone());
        Ok(())
    }

    async fn find_user(&self, channel: &str, handle: &str) -> Result<Option<User>> {
        let key = Self::key(channel, handle);
        Ok(self.users.get(&key).map(|entry| entry.clone()))
    }

    async fn set_role(&self, user_id: Uuid, role: Role) -> Result<()> {
        for mut entry in self.users.iter_mut() {
            let user = entry.value_mut();
            if user.id == user_id {
                user.role = role.clone();
                break;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl MintRepository for MemoryStore {
    async fn enqueue(&self, request: &MintRequest) -> Result<()> {
        let mut cloned = request.clone();
        cloned.status = MintStatus::Pending;
        self.mints.insert(cloned.id, cloned.clone());
        let mut queue = self.queue.lock().await;
        queue.push_back(cloned.id);
        Ok(())
    }

    async fn next_pending(&self) -> Result<Option<MintRequest>> {
        let mut queue = self.queue.lock().await;
        while let Some(id) = queue.pop_front() {
            if let Some(mut entry) = self.mints.get_mut(&id) {
                if matches!(entry.status, MintStatus::Pending | MintStatus::Processing) {
                    entry.status = MintStatus::Processing;
                    entry.processed_at = Some(Utc::now());
                    entry.attempt = entry.attempt.saturating_add(1);
                    return Ok(Some(entry.clone()));
                }
            }
        }
        Ok(None)
    }

    async fn update_status(&self, request_id: Uuid, status: MintStatus) -> Result<()> {
        if let Some(mut entry) = self.mints.get_mut(&request_id) {
            entry.status = status.clone();
            if matches!(status, MintStatus::Completed | MintStatus::Failed) {
                entry.processed_at = Some(Utc::now());
            }
        }
        Ok(())
    }

    async fn record_outcome(&self, outcome: &MintOutcome) -> Result<()> {
        if let Some(mut entry) = self.mints.get_mut(&outcome.request.id) {
            *entry = outcome.request.clone();
            entry.tx_hash = outcome.tx_hash.clone();
        }

        if outcome.request.status == MintStatus::Completed {
            let key = (
                outcome.request.user_id,
                outcome.request.requested_at.date_naive(),
            );
            self.quotas
                .entry(key)
                .and_modify(|quota| {
                    quota.success_count += 1;
                })
                .or_insert_with(|| Quota {
                    id: Uuid::new_v4(),
                    user_id: outcome.request.user_id,
                    day: outcome.request.requested_at.date_naive(),
                    minted_total: 0,
                    success_count: 1,
                });
        }

        Ok(())
    }
}

#[async_trait]
impl QuotaRepository for MemoryStore {
    async fn record_mint(&self, user_id: Uuid, day: NaiveDate, amount: u64) -> Result<()> {
        self.quotas
            .entry((user_id, day))
            .and_modify(|quota| {
                quota.minted_total += amount;
            })
            .or_insert_with(|| Quota {
                id: Uuid::new_v4(),
                user_id,
                day,
                minted_total: amount,
                success_count: 0,
            });
        Ok(())
    }

    async fn fetch_quota(&self, user_id: Uuid, day: NaiveDate) -> Result<Option<Quota>> {
        Ok(self.quotas.get(&(user_id, day)).map(|quota| quota.clone()))
    }
}

#[async_trait]
impl ReportingRepository for MemoryStore {
    async fn daily_summary(&self, day: NaiveDate) -> Result<Vec<DailyReportRow>> {
        use std::collections::HashMap;

        let mut totals: HashMap<String, (u64, u64, u64)> = HashMap::new();
        for mint_ref in self.mints.iter() {
            let mint = mint_ref.value();
            if mint.requested_at.date_naive() != day {
                continue;
            }
            let key = mint.channel.as_str().to_string();
            let entry = totals.entry(key).or_insert((0, 0, 0));
            entry.0 += mint.amount;
            if matches!(mint.status, MintStatus::Completed) {
                entry.1 += 1;
            } else if matches!(mint.status, MintStatus::Failed) {
                entry.2 += 1;
            }
        }

        Ok(totals
            .into_iter()
            .map(|kv| DailyReportRow {
                channel: kv.0,
                total_amount: kv.1 .0,
                success_count: kv.1 .1,
                failure_count: kv.1 .2,
            })
            .collect())
    }

    async fn log_failure(&self, request_id: Uuid, when: DateTime<Utc>, reason: &str) -> Result<()> {
        let mut failures = self.failures.lock().await;
        failures.push((request_id, when, reason.to_string()));
        Ok(())
    }
}

#[async_trait]
impl ConfigRepository for MemoryStore {
    async fn get_config(&self, key: &str) -> Result<Option<SystemConfig>> {
        Ok(self.configs.get(key).map(|entry| entry.value().clone()))
    }

    async fn set_config(&self, key: &str, value: &str, description: Option<&str>) -> Result<()> {
        let now = Utc::now();
        let config = SystemConfig {
            id: Uuid::new_v4(),
            key: key.to_string(),
            value: value.to_string(),
            description: description.map(|s| s.to_string()),
            created_at: now,
            updated_at: now,
        };
        self.configs.insert(key.to_string(), config);
        Ok(())
    }

    async fn get_all_configs(&self) -> Result<Vec<SystemConfig>> {
        Ok(self.configs.iter().map(|entry| entry.value().clone()).collect())
    }

    async fn update_limit_config(&self, config: &LimitConfigUpdate) -> Result<()> {
        if let Some(amount) = config.default_amount {
            self.set_config("limits.default_amount", &amount.to_string(), Some("Default user amount")).await?;
        }
        if let Some(cap) = config.default_daily_cap {
            self.set_config("limits.default_daily_cap", &cap.to_string(), Some("Default user daily cap")).await?;
        }
        if let Some(amount) = config.privileged_amount {
            self.set_config("limits.privileged_amount", &amount.to_string(), Some("Privileged user amount")).await?;
        }
        if let Some(cap) = config.privileged_daily_cap {
            self.set_config("limits.privileged_daily_cap", &cap.to_string(), Some("Privileged user daily cap")).await?;
        }
        Ok(())
    }

    async fn get_limit_config(&self) -> Result<Option<LimitConfigUpdate>> {
        let default_amount = self.get_config("limits.default_amount").await?.map(|c| c.value.parse::<u64>().ok()).flatten();
        let default_daily_cap = self.get_config("limits.default_daily_cap").await?.map(|c| c.value.parse::<u64>().ok()).flatten();
        let privileged_amount = self.get_config("limits.privileged_amount").await?.map(|c| c.value.parse::<u64>().ok()).flatten();
        let privileged_daily_cap = self.get_config("limits.privileged_daily_cap").await?.map(|c| c.value.parse::<u64>().ok()).flatten();

        if default_amount.is_some() || default_daily_cap.is_some() || privileged_amount.is_some() || privileged_daily_cap.is_some() {
            Ok(Some(LimitConfigUpdate {
                default_amount,
                default_daily_cap,
                privileged_amount,
                privileged_daily_cap,
            }))
        } else {
            Ok(None)
        }
    }
}
