use std::str::FromStr;

use anyhow::Context;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Web,
    Telegram,
    Discord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Privileged,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub channel: Channel,
    pub handle: String,
    pub role: Role,
    pub domain: Option<String>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel: Channel,
    pub amount: u64,
    pub status: MintStatus,
    pub tx_hash: Option<String>,
    pub error: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub attempt: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MintStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quota {
    pub id: Uuid,
    pub user_id: Uuid,
    pub day: NaiveDate,
    pub minted_total: u64,
    pub success_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintOutcome {
    pub request: MintRequest,
    pub tx_hash: Option<String>,
}

impl Channel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Channel::Web => "web",
            Channel::Telegram => "telegram",
            Channel::Discord => "discord",
        }
    }
}

impl FromStr for Channel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "web" => Ok(Channel::Web),
            "telegram" => Ok(Channel::Telegram),
            "discord" => Ok(Channel::Discord),
            other => anyhow::bail!("unknown channel: {other}"),
        }
    }
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Privileged => "privileged",
            Role::Admin => "admin",
        }
    }
}

impl FromStr for Role {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "user" => Ok(Role::User),
            "privileged" => Ok(Role::Privileged),
            "admin" => Ok(Role::Admin),
            other => anyhow::bail!("unknown role: {other}"),
        }
    }
}

impl MintStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MintStatus::Pending => "pending",
            MintStatus::Processing => "processing",
            MintStatus::Completed => "completed",
            MintStatus::Failed => "failed",
        }
    }
}

impl FromStr for MintStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "pending" => Ok(MintStatus::Pending),
            "processing" => Ok(MintStatus::Processing),
            "completed" => Ok(MintStatus::Completed),
            "failed" => Ok(MintStatus::Failed),
            other => anyhow::bail!("unknown status: {other}"),
        }
    }
}

pub fn channel_from_db(value: &str) -> anyhow::Result<Channel> {
    Channel::from_str(value).with_context(|| format!("invalid channel value: {value}"))
}

pub fn role_from_db(value: &str) -> anyhow::Result<Role> {
    Role::from_str(value).with_context(|| format!("invalid role value: {value}"))
}

pub fn status_from_db(value: &str) -> anyhow::Result<MintStatus> {
    MintStatus::from_str(value).with_context(|| format!("invalid status value: {value}"))
}
