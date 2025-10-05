use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::LimitConfig;
use crate::models::{Role, User};
use crate::repository::QuotaRepository;

pub struct RateLimiter<R> {
    repo: R,
    memory: Mutex<HashMap<(Uuid, NaiveDate), u64>>,
    limits: LimitConfig,
}

impl<R: QuotaRepository> RateLimiter<R> {
    pub fn new(repo: R, limits: LimitConfig) -> Self {
        Self {
            repo,
            memory: Mutex::new(HashMap::new()),
            limits,
        }
    }

    pub fn max_amount(&self, role: &Role) -> u64 {
        match role {
            Role::Admin | Role::Privileged => self.limits.privileged_amount,
            Role::User => self.limits.default_amount,
        }
    }

    fn max_daily_cap(&self, role: &Role) -> Option<u64> {
        match role {
            Role::Admin => self.limits.privileged_daily_cap,
            Role::Privileged => self.limits.privileged_daily_cap,
            Role::User => Some(self.limits.default_daily_cap),
        }
    }

    pub async fn check_and_record(&self, user: &User, amount: u64) -> Result<()> {
        let today = Utc::now().date_naive();
        if amount > self.max_amount(&user.role) {
            anyhow::bail!("amount exceeds role limit");
        }

        if let Some(cap) = self.max_daily_cap(&user.role) {
            let mut guard = self.memory.lock().await;
            let key = (user.id, today);
            let entry = guard.entry(key).or_insert(0);
            if *entry + amount > cap {
                anyhow::bail!("daily cap reached");
            }
            *entry += amount;
        }

        self.repo.record_mint(user.id, today, amount).await
    }
}

#[async_trait]
pub trait LimitRefresh {
    async fn refresh(&self, user_id: Uuid, when: DateTime<Utc>) -> Result<()>;
}
