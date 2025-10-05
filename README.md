# Rust Faucet Service

一个支持 Web、Telegram、Discord 多渠道访问的 Aptos 水龙头服务骨架，围绕共享核心库构建。当前仓库提供基础架构和模块接口，便于后续继续完善业务逻辑与数据库实现。

## 工作区结构

- `core`：配置加载、角色权限、队列与仓储接口。
- `web`：Axum Web 服务，负责页面、OAuth、管理后台（待实现）。
- `tg-bot`：Telegram Bot 入口，处理指令后调用核心服务。
- `dc-bot`：Discord Bot 入口，支持 Slash Commands。
- `reporting`：报表与计划任务。

## 快速开始

1. 安装 Rust 1.75+ 与 Cargo。
2. 配置必要的环境变量：
   ```shell
   export FAUCET__AUTH__GOOGLE_CLIENT_ID="your-client-id"
   export FAUCET__AUTH__GOOGLE_CLIENT_SECRET="your-client-secret"
   export TELEGRAM_BOT_TOKEN="your-telegram-token"
   export DISCORD_TOKEN="your-discord-token"
   ```
3. 更新 `config/default.toml` 以匹配数据库与限额配置。
4. 在不同终端运行：
   ```shell
   cargo run -p web
   cargo run -p tg-bot
   cargo run -p dc-bot
   cargo run -p reporting
   ```

## 下一步建议

- 在 `core::repository` 中实现 PostgreSQL 与 MongoDB 的具体仓储。
- 将队列处理逻辑与 Aptos SDK 集成，提交真实交易。
- 完善 Web 前端页面、OAuth 回调及管理端。
- 为 Bot 增加角色、限额查询与管理员配置接口。
- 编写集成测试与 CI 流程，保障多渠道协同稳定。
