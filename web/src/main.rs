mod auth;
mod error;
mod session;

use std::sync::Arc;

use anyhow::Result;
use auth::GoogleVerifier;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use error::ApiError;
use faucet_core::{
    config::AppConfig,
    logging,
    models::{Channel, MintStatus, Role, User},
    queue::LoggingAptosClient,
    DatabaseStore, FaucetService, Identity,
};
use serde::{Deserialize, Serialize};
use session::SessionManager;
use tokio::signal;
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    faucet: Arc<FaucetService<DatabaseStore, LoggingAptosClient>>,
    sessions: SessionManager,
    verifier: GoogleVerifier,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;
    logging::init_telemetry(&config.telemetry);

    let skip_db = should_skip_db();
    let store = if skip_db {
        warn!("数据库连接已跳过，使用内存存储，所有数据将在进程结束后丢失");
        Arc::new(DatabaseStore::memory())
    } else {
        Arc::new(DatabaseStore::connect(&config.database).await?)
    };
    let aptos_client = Arc::new(LoggingAptosClient);
    let faucet = Arc::new(FaucetService::new(
        store.clone(),
        aptos_client,
        config.limits.clone(),
        &config.auth,
    ));

    let verifier = GoogleVerifier::new(&config.auth.google_client_id)?;

    let state = AppState {
        faucet,
        sessions: SessionManager::default(),
        verifier,
    };

    info!(addr = %config.server.http_addr, "Web 服务启动");

    let router = build_router(state);

    let listener = tokio::net::TcpListener::bind(&config.server.http_addr).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

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

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/session", post(create_session))
        .route("/api/me", get(current_user))
        .route("/api/mint", post(mint_tokens))
        .route("/api/admin/role", post(update_role))
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    StatusCode::OK
}

#[derive(Debug, Deserialize)]
struct SessionRequest {
    id_token: String,
}

#[derive(Debug, Serialize)]
struct SessionResponse {
    token: String,
    user: UserView,
}

#[derive(Debug, Serialize)]
struct UserView {
    handle: String,
    role: Role,
    max_amount: u64,
    max_daily_cap: Option<u64>,
    minted_today: u64,
    remaining_today: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct MintRequestPayload {
    amount: Option<u64>,
}

#[derive(Debug, Serialize)]
struct MintResponse {
    status: MintStatus,
    amount: u64,
    tx_hash: Option<String>,
    minted_today: u64,
    remaining_today: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RoleUpdateRequest {
    handle: String,
    channel: Channel,
    role: Role,
}

async fn create_session(
    State(state): State<AppState>,
    Json(payload): Json<SessionRequest>,
) -> Result<Json<SessionResponse>, ApiError> {
    let profile = state
        .verifier
        .verify(&payload.id_token)
        .await
        .map_err(|err| {
            tracing::warn!(error = %err, "google_token_invalid");
            ApiError::Unauthorized
        })?;

    let user = state
        .faucet
        .touch_user(Identity {
            channel: Channel::Web,
            handle: &profile.email,
            domain: profile.domain.as_deref(),
        })
        .await?;

    let token = state.sessions.create(&user);
    let view = build_user_view(&state, &user).await?;
    Ok(Json(SessionResponse { token, user: view }))
}

async fn current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserView>, ApiError> {
    let token = extract_bearer(&headers)?;
    let user = resolve_user(&state, token).await?;
    Ok(Json(build_user_view(&state, &user).await?))
}

async fn mint_tokens(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MintRequestPayload>,
) -> Result<Json<MintResponse>, ApiError> {
    let token = extract_bearer(&headers)?;
    let user = resolve_user(&state, token).await?;
    let amount = payload
        .amount
        .unwrap_or_else(|| state.faucet.default_amount(&user.role));

    let outcome = state.faucet.mint(&user, amount).await?;
    let snapshot = state.faucet.quota_snapshot(&user).await?;

    Ok(Json(MintResponse {
        status: outcome.request.status,
        amount: outcome.request.amount,
        tx_hash: outcome.tx_hash,
        minted_today: snapshot.minted,
        remaining_today: snapshot.remaining(),
    }))
}

async fn update_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RoleUpdateRequest>,
) -> Result<Json<UserView>, ApiError> {
    let token = extract_bearer(&headers)?;
    let actor = resolve_user(&state, token).await?;
    if !matches!(actor.role, Role::Admin) {
        return Err(ApiError::Forbidden);
    }

    let updated = state
        .faucet
        .set_role(
            &actor,
            payload.channel.clone(),
            &payload.handle,
            payload.role.clone(),
        )
        .await?;

    Ok(Json(build_user_view(&state, &updated).await?))
}

async fn build_user_view(state: &AppState, user: &User) -> Result<UserView, ApiError> {
    let snapshot = state.faucet.quota_snapshot(user).await?;
    Ok(UserView {
        handle: user.handle.clone(),
        role: user.role.clone(),
        max_amount: state.faucet.max_amount_for_role(&user.role),
        max_daily_cap: state.faucet.max_daily_cap(&user.role),
        minted_today: snapshot.minted,
        remaining_today: snapshot.remaining(),
    })
}

async fn resolve_user(state: &AppState, token: &str) -> Result<User, ApiError> {
    let session = state.sessions.get(token).ok_or(ApiError::Unauthorized)?;

    let identity = Identity {
        channel: session.channel.clone(),
        handle: &session.handle,
        domain: session.domain.as_deref(),
    };

    let user = state.faucet.touch_user(identity).await?;

    Ok(user)
}

fn extract_bearer(headers: &HeaderMap) -> Result<&str, ApiError> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
        .ok_or(ApiError::Unauthorized)?
        .trim();

    if token.is_empty() {
        Err(ApiError::Unauthorized)
    } else {
        Ok(token)
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};

        signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("收到关闭信号");
}
