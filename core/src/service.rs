use std::{collections::HashSet, sync::Arc};

use anyhow::Result;
use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    config::{AuthConfig, LimitConfig},
    models::{Channel, MintOutcome, MintStatus, Role, User},
    queue::{new_request, AptosClient},
    rate_limit::RateLimiter,
    repository::{MintRepository, QuotaRepository, ReportingRepository, UserRepository},
};

#[derive(Debug, Clone)]
pub struct Identity<'a> {
    pub channel: Channel,
    pub handle: &'a str,
    pub domain: Option<&'a str>,
}

pub struct FaucetService<S, C>
where
    S: UserRepository
        + MintRepository
        + QuotaRepository
        + ReportingRepository
        + Send
        + Sync
        + 'static,
    C: AptosClient,
{
    store: Arc<S>,
    client: Arc<C>,
    limits: LimitConfig,
    privileged_domains: HashSet<String>,
    rate_limiter: RateLimiter<Arc<S>>,
}

impl<S, C> FaucetService<S, C>
where
    S: UserRepository
        + MintRepository
        + QuotaRepository
        + ReportingRepository
        + Send
        + Sync
        + 'static,
    C: AptosClient,
{
    pub fn new(store: Arc<S>, client: Arc<C>, limits: LimitConfig, auth: &AuthConfig) -> Self {
        let privileged_domains = auth
            .privileged_domains
            .iter()
            .map(|d| d.to_ascii_lowercase())
            .collect::<HashSet<_>>();

        let rate_limiter = RateLimiter::new(store.clone(), limits.clone());

        Self {
            store,
            client,
            limits,
            privileged_domains,
            rate_limiter,
        }
    }

    pub fn limits(&self) -> &LimitConfig {
        &self.limits
    }

    pub fn max_amount_for_role(&self, role: &Role) -> u64 {
        self.rate_limiter.max_amount(role)
    }

    fn determine_role(&self, existing: Option<&Role>, domain: Option<&str>) -> Role {
        if matches!(existing, Some(Role::Admin)) {
            return Role::Admin;
        }

        if let Some(domain) = domain {
            let clean = domain.to_ascii_lowercase();
            if self.privileged_domains.contains(&clean) {
                return Role::Privileged;
            }
        }

        existing.cloned().unwrap_or(Role::User)
    }

    pub async fn touch_user(&self, identity: Identity<'_>) -> Result<User> {
        if let Some(mut user) = self
            .store
            .find_user(identity.channel.as_str(), identity.handle)
            .await?
        {
            let mut changed = false;
            let determined_role = self.determine_role(Some(&user.role), identity.domain);
            if determined_role != user.role {
                user.role = determined_role;
                changed = true;
            }

            if identity.domain.map(|s| s.to_string()) != user.domain {
                user.domain = identity.domain.map(|s| s.to_string());
                changed = true;
            }

            user.last_seen_at = Utc::now();
            if changed {
                self.store.upsert_user(&user).await?;
            }
            // persist heartbeat update
            self.store.upsert_user(&user).await?;
            Ok(user)
        } else {
            let mut user = User {
                id: Uuid::new_v4(),
                channel: identity.channel.clone(),
                handle: identity.handle.to_string(),
                role: Role::User,
                domain: identity.domain.map(|s| s.to_string()),
                last_seen_at: Utc::now(),
            };
            user.role = self.determine_role(None, identity.domain);
            self.store.upsert_user(&user).await?;
            Ok(user)
        }
    }

    pub async fn set_role(
        &self,
        actor: &User,
        target_channel: Channel,
        target_handle: &str,
        role: Role,
    ) -> Result<User> {
        if !matches!(actor.role, Role::Admin) {
            anyhow::bail!("only admins may change roles");
        }

        let mut user = self
            .store
            .find_user(target_channel.as_str(), target_handle)
            .await?
            .unwrap_or(User {
                id: Uuid::new_v4(),
                channel: target_channel.clone(),
                handle: target_handle.to_string(),
                role: Role::User,
                domain: None,
                last_seen_at: Utc::now(),
            });
        user.role = role;
        user.last_seen_at = Utc::now();
        self.store.upsert_user(&user).await?;
        Ok(user)
    }

    pub async fn mint(&self, user: &User, amount: u64) -> Result<MintOutcome> {
        if amount == 0 {
            anyhow::bail!("amount must be greater than zero");
        }

        self.rate_limiter.check_and_record(user, amount).await?;

        let mut request = new_request(user.id, user.channel.clone(), amount);
        self.store.enqueue(&request).await?;
        self.store
            .update_status(request.id, MintStatus::Processing)
            .await?;
        request.status = MintStatus::Processing;
        request.attempt = request.attempt.saturating_add(1);

        match self.client.submit_transfer(&request).await {
            Ok(hash) => {
                request.status = MintStatus::Completed;
                request.tx_hash = Some(hash.clone());
                request.processed_at = Some(Utc::now());

                let outcome = MintOutcome {
                    request: request.clone(),
                    tx_hash: Some(hash.clone()),
                };
                self.store.record_outcome(&outcome).await?;
                info!(user = %user.handle, ?hash, "mint_success");
                Ok(outcome)
            }
            Err(err) => {
                let error_message = err.to_string();
                warn!(user = %user.handle, error = %error_message, "mint_failed");
                request.status = MintStatus::Failed;
                request.error = Some(error_message.clone());
                request.processed_at = Some(Utc::now());

                let outcome = MintOutcome {
                    request: request.clone(),
                    tx_hash: None,
                };

                self.store.record_outcome(&outcome).await?;
                self.store
                    .log_failure(request.id, Utc::now(), &error_message)
                    .await?;

                Err(err)
            }
        }
    }

    pub fn default_amount(&self, role: &Role) -> u64 {
        match role {
            Role::Admin | Role::Privileged => self.limits.privileged_amount,
            Role::User => self.limits.default_amount,
        }
    }

    pub fn max_daily_cap(&self, role: &Role) -> Option<u64> {
        match role {
            Role::Admin | Role::Privileged => self.limits.privileged_daily_cap,
            Role::User => Some(self.limits.default_daily_cap),
        }
    }

    pub async fn quota_snapshot(&self, user: &User) -> Result<QuotaSnapshot> {
        let today = Utc::now().date_naive();
        let minted = self
            .store
            .fetch_quota(user.id, today)
            .await?
            .map(|quota| quota.minted_total)
            .unwrap_or(0);

        Ok(QuotaSnapshot {
            minted,
            cap: self.max_daily_cap(&user.role),
        })
    }

    pub async fn find_user(&self, channel: Channel, handle: &str) -> Result<Option<User>> {
        self.store.find_user(channel.as_str(), handle).await
    }
}

#[derive(Debug, Clone)]
pub struct QuotaSnapshot {
    pub minted: u64,
    pub cap: Option<u64>,
}

impl QuotaSnapshot {
    pub fn remaining(&self) -> Option<u64> {
        self.cap.map(|cap| cap.saturating_sub(self.minted))
    }
}
