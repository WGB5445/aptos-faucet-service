pub mod config;
pub mod db;
pub mod logging;
pub mod models;
pub mod queue;
pub mod rate_limit;
pub mod repository;
pub mod service;

pub use db::DatabaseStore;
pub use service::{FaucetService, Identity};

use anyhow::Result;
use tokio::task::JoinHandle;

pub trait Service: Send + Sync + 'static {
    fn spawn(self) -> JoinHandle<Result<()>>;
}
