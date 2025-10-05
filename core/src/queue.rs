use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::{MintOutcome, MintRequest, MintStatus};
use crate::repository::{MintRepository, UserRepository};

#[async_trait]
pub trait AptosClient: Send + Sync {
    async fn submit_transfer(&self, request: &MintRequest) -> Result<String>;
}

#[derive(Clone)]
pub struct MintQueue<R, U, C> {
    repo: Arc<R>,
    users: Arc<U>,
    client: Arc<C>,
    tx: mpsc::Sender<MintRequest>,
}

impl<R, U, C> MintQueue<R, U, C>
where
    R: MintRepository + 'static,
    U: UserRepository + 'static,
    C: AptosClient + 'static,
{
    pub fn new(
        repo: Arc<R>,
        users: Arc<U>,
        client: Arc<C>,
        depth: usize,
    ) -> (Self, mpsc::Receiver<MintRequest>) {
        let (tx, rx) = mpsc::channel(depth);
        (
            Self {
                repo,
                users,
                client,
                tx,
            },
            rx,
        )
    }

    pub async fn enqueue(&self, mut request: MintRequest) -> Result<()> {
        request.status = MintStatus::Pending;
        self.repo.enqueue(&request).await?;
        self.tx
            .send(request)
            .await
            .map_err(|_| anyhow::anyhow!("queue closed"))
    }
}

pub async fn worker_loop<R, U, C>(
    mut rx: mpsc::Receiver<MintRequest>,
    repo: Arc<R>,
    client: Arc<C>,
) -> Result<()>
where
    R: MintRepository + 'static,
    U: UserRepository + 'static,
    C: AptosClient + 'static,
{
    while let Some(mut request) = rx.recv().await {
        repo.update_status(request.id, MintStatus::Processing)
            .await?;
        match client.submit_transfer(&request).await {
            Ok(hash) => {
                request.status = MintStatus::Completed;
                request.tx_hash = Some(hash.clone());
                repo.record_outcome(&MintOutcome {
                    request,
                    tx_hash: Some(hash),
                })
                .await?;
            }
            Err(err) => {
                warn!(request_id = %request.id, error = %err, "mint_failed");
                repo.update_status(request.id, MintStatus::Failed).await?;
            }
        }
    }

    info!("Mint worker 已停止");
    Ok(())
}

pub fn new_request(user_id: Uuid, channel: crate::models::Channel, amount: u64) -> MintRequest {
    let now = chrono::Utc::now();
    MintRequest {
        id: Uuid::new_v4(),
        user_id,
        channel,
        amount,
        status: MintStatus::Pending,
        tx_hash: None,
        error: None,
        requested_at: now,
        processed_at: None,
        attempt: 0,
    }
}

pub struct LoggingAptosClient;

#[async_trait]
impl AptosClient for LoggingAptosClient {
    async fn submit_transfer(&self, request: &MintRequest) -> Result<String> {
        info!(user_id = %request.user_id, amount = request.amount, "mock_aptos_transfer");
        Ok(format!("mock-tx-{}", Uuid::new_v4()))
    }
}
