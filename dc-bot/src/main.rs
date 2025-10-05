use std::{str::FromStr, sync::Arc};

use anyhow::{Context as AnyhowContext, Result};
use faucet_core::{
    config::AppConfig,
    logging,
    models::{Channel, Role},
    queue::LoggingAptosClient,
    DatabaseStore, FaucetService, Identity,
};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use tracing::{error, info, warn};

struct BotState {
    faucet: Arc<FaucetService<DatabaseStore, LoggingAptosClient>>,
}

struct Handler {
    state: Arc<BotState>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if let Err(err) = self.handle_message(&ctx, &msg).await {
            error!(%err, "dc_message_error");
            if let Err(reply_err) = msg
                .channel_id
                .say(&ctx.http, format!("❌ 出错了: {}", err))
                .await
            {
                error!(%reply_err, "dc_reply_error");
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!(user = %ready.user.name, "Discord bot 已联机");
    }
}

impl Handler {
    async fn handle_message(&self, ctx: &Context, msg: &Message) -> Result<()> {
        let content = msg.content.trim();
        if content.starts_with("!mint") {
            self.handle_mint(ctx, msg, content).await
        } else if content.starts_with("!setrole") {
            self.handle_set_role(ctx, msg, content).await
        } else if content.starts_with("!help") {
            msg.channel_id
                .say(
                    &ctx.http,
                    "命令列表:\n!mint [amount] - 按默认或指定数量发放\n!setrole <@user> <user|privileged|admin> - 管理员设定角色",
                )
                .await?;
            Ok(())
        } else {
            Ok(())
        }
    }

    async fn handle_mint(&self, ctx: &Context, msg: &Message, content: &str) -> Result<()> {
        let handle = msg.author.id.to_string();
        let profile = self
            .state
            .faucet
            .touch_user(Identity {
                channel: Channel::Discord,
                handle: &handle,
                domain: None,
            })
            .await?;

        let mut parts = content.split_whitespace();
        parts.next();
        let amount = parts.next().map(|value| value.parse::<u64>()).transpose()?;
        let amount = amount.unwrap_or_else(|| self.state.faucet.default_amount(&profile.role));

        match self.state.faucet.mint(&profile, amount).await {
            Ok(outcome) => {
                let snapshot = self.state.faucet.quota_snapshot(&profile).await?;
                let hash = outcome.tx_hash.unwrap_or_else(|| "<pending>".to_string());
                msg.channel_id
                    .say(
                        &ctx.http,
                        format!(
                            "✅ 已分发 {} 枚代币\nTx: {}\n今日已用: {}\n今日剩余: {}",
                            outcome.request.amount,
                            hash,
                            snapshot.minted,
                            snapshot
                                .remaining()
                                .map(|left| left.to_string())
                                .unwrap_or_else(|| "无限制".to_string()),
                        ),
                    )
                    .await?;
            }
            Err(err) => {
                msg.channel_id
                    .say(&ctx.http, format!("❌ 失败: {}", err))
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_set_role(&self, ctx: &Context, msg: &Message, content: &str) -> Result<()> {
        let actor_handle = msg.author.id.to_string();
        let actor = self
            .state
            .faucet
            .touch_user(Identity {
                channel: Channel::Discord,
                handle: &actor_handle,
                domain: None,
            })
            .await?;

        if !matches!(actor.role, Role::Admin) {
            msg.channel_id
                .say(&ctx.http, "⚠️ 只有管理员可以设置角色")
                .await?;
            return Ok(());
        }

        let mut parts = content.split_whitespace();
        parts.next();
        let user_part = parts.next().context("缺少用户参数")?;
        let role_part = parts.next().context("缺少角色参数")?;
        let role = Role::from_str(role_part)?;

        let user_id = user_part
            .trim_matches(|c: char| c == '<' || c == '>' || c == '@' || c == '!')
            .to_string();

        let updated = self
            .state
            .faucet
            .set_role(&actor, Channel::Discord, &user_id, role.clone())
            .await?;

        msg.channel_id
            .say(
                &ctx.http,
                format!("✅ 已将用户 {} 设为 {:?}", user_id, updated.role),
            )
            .await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;
    logging::init_telemetry(&config.telemetry);

    let token = std::env::var("DISCORD_TOKEN")?;

    let skip_db = should_skip_db();
    let store = if skip_db {
        warn!("数据库连接已跳过，Discord 机器人使用内存存储，数据不会持久化");
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

    let handler = Handler {
        state: Arc::new(BotState { faucet }),
    };

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES;
    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .await?;

    client.start().await?;

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
