use std::{str::FromStr, sync::Arc};

use anyhow::{Context, Result};
use faucet_core::{
    config::AppConfig,
    logging,
    models::{Channel, Role, User},
    queue::LoggingAptosClient,
    DatabaseStore, FaucetService, Identity,
};
use teloxide::{
    dispatching::UpdateFilterExt, dptree, error_handlers::ErrorHandler, prelude::*,
    update_listeners::Polling,
};
use tracing::{error, info, warn};

#[derive(Clone)]
struct BotState {
    faucet: Arc<FaucetService<DatabaseStore, LoggingAptosClient>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;
    logging::init_telemetry(&config.telemetry);

    let skip_db = should_skip_db();
    let store = if skip_db {
        warn!("数据库连接已跳过，Telegram 机器人使用内存存储，数据不会持久化");
        Arc::new(DatabaseStore::memory())
    } else {
        Arc::new(DatabaseStore::connect(&config.database).await?)
    };
    let faucet = Arc::new(FaucetService::new(
        store.clone(),
        Arc::new(LoggingAptosClient),
        config.limits.clone(),
        &config.auth,
    ));

    let bot = Bot::from_env();
    let state = Arc::new(BotState { faucet });

    info!("Telegram bot 启动");

    Dispatcher::builder(
        bot.clone(),
        Update::filter_message().endpoint(handle_message),
    )
    .dependencies(dptree::deps![bot, state.clone()])
    .enable_ctrlc_handler()
    .build()
    .dispatch_with_listener(
        Polling::builder(Bot::from_env()).build(),
        Arc::new(LoggingErrorHandler),
    )
    .await;

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

async fn handle_message(bot: Bot, msg: Message, state: Arc<BotState>) -> Result<()> {
    let text = msg.text().unwrap_or("").trim();
    if text.is_empty() {
        return Ok(());
    }

    let user = msg.from.as_ref().context("消息缺少发送者信息")?;
    let handle = user
        .username
        .clone()
        .unwrap_or_else(|| user.id.0.to_string());

    let profile = state
        .faucet
        .touch_user(Identity {
            channel: Channel::Telegram,
            handle: &handle,
            domain: None,
        })
        .await?;

    if text.starts_with("/start") || text.starts_with("/help") {
        send_welcome(&bot, &msg, &state, &profile, &handle).await?;
    } else if text.starts_with("/mint") {
        let amount = text
            .split_whitespace()
            .nth(1)
            .map(|value| value.parse::<u64>())
            .transpose()?;
        let amount = amount.unwrap_or_else(|| state.faucet.default_amount(&profile.role));
        handle_mint(&bot, &msg, &state, &profile, amount).await?;
    } else if text.starts_with("/setrole") {
        let mut parts = text.split_whitespace();
        parts.next();
        let target = parts.next().context("缺少用户参数")?;
        let role_str = parts.next().context("缺少角色参数")?;
        let role = Role::from_str(role_str)?;
        set_role(&bot, &msg, &state, &profile, target.to_string(), role).await?;
    }

    Ok(())
}

async fn send_welcome(
    bot: &Bot,
    msg: &Message,
    state: &Arc<BotState>,
    profile: &User,
    handle: &str,
) -> Result<()> {
    let snapshot = state.faucet.quota_snapshot(profile).await?;
    let cap_text = snapshot
        .cap
        .map(|cap| cap.to_string())
        .unwrap_or_else(|| "无限制".to_string());
    let remaining_text = snapshot
        .remaining()
        .map(|left| left.to_string())
        .unwrap_or_else(|| "无限制".to_string());
    let message = format!(
        "欢迎回来, {}!\n角色: {:?}\n单次额度: {}\n日上限: {}\n今日已用: {}\n今日剩余: {}",
        handle,
        profile.role,
        state.faucet.max_amount_for_role(&profile.role),
        cap_text,
        snapshot.minted,
        remaining_text,
    );
    bot.send_message(msg.chat.id, message).await?;
    Ok(())
}

async fn handle_mint(
    bot: &Bot,
    msg: &Message,
    state: &Arc<BotState>,
    profile: &User,
    amount: u64,
) -> Result<()> {
    match state.faucet.mint(profile, amount).await {
        Ok(outcome) => {
            let snapshot = state.faucet.quota_snapshot(profile).await?;
            let hash = outcome.tx_hash.as_deref().unwrap_or("<pending>");
            let remaining_text = snapshot
                .remaining()
                .map(|left| left.to_string())
                .unwrap_or_else(|| "无限制".to_string());
            let message = format!(
                "✅ 铸币成功!\n数量: {}\n交易: {}\n今日已用: {}\n今日剩余: {}",
                outcome.request.amount, hash, snapshot.minted, remaining_text,
            );
            bot.send_message(msg.chat.id, message).await?;
        }
        Err(err) => {
            bot.send_message(msg.chat.id, format!("❌ 失败: {}", err))
                .await?;
        }
    }
    Ok(())
}

async fn set_role(
    bot: &Bot,
    msg: &Message,
    state: &Arc<BotState>,
    actor: &User,
    handle: String,
    role: Role,
) -> Result<()> {
    if !matches!(actor.role, Role::Admin) {
        bot.send_message(msg.chat.id, "只有管理员可以设置角色")
            .await?;
        return Ok(());
    }

    let target_handle = handle.trim_start_matches('@').to_string();
    match state
        .faucet
        .set_role(actor, Channel::Telegram, &target_handle, role.clone())
        .await
    {
        Ok(updated) => {
            bot.send_message(
                msg.chat.id,
                format!("已将 {} 的角色更新为 {:?}", updated.handle, updated.role),
            )
            .await?;
        }
        Err(err) => {
            bot.send_message(msg.chat.id, format!("更新失败: {}", err))
                .await?;
        }
    }

    Ok(())
}

struct LoggingErrorHandler;

impl<E: std::fmt::Display + Send + 'static> ErrorHandler<E> for LoggingErrorHandler {
    fn handle_error(
        self: Arc<Self>,
        error: E,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>> {
        Box::pin(async move {
            error!(%error, "tg_dispatch_error");
        })
    }
}
