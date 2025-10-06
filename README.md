# Rust Faucet Service

一个支持 Web、Telegram、Discord 多渠道访问的 Aptos 水龙头服务骨架，围绕共享核心库构建。当前仓库提供基础架构和模块接口，便于后续继续完善业务逻辑与数据库实现。

## 工作区结构

- `core`：配置加载、角色权限、队列与仓储接口。
- `web`：Axum Web 服务，负责页面、OAuth、管理后台（待实现）。
- `tg-bot`：Telegram Bot 入口，处理指令后调用核心服务。
- `dc-bot`：Discord Bot 入口，支持 Slash Commands。
- `reporting`：报表与计划任务。
- `frontend`：React + Vite 前端工程，可独立部署 faucet & 管理界面。

## 快速开始

1. 安装 Rust 1.75+ 与 Cargo。

2. 复制环境变量示例文件并配置：
   ```shell
   cp env.example .env
   # 编辑 .env 文件，填入真实的配置值
   ```

3. 或者直接设置环境变量：
   ```shell
   export FAUCET__DATABASE__URL="postgres://user:password@localhost:5432/faucet"
   export FAUCET__AUTH__GOOGLE_CLIENT_ID="your-client-id"
   export FAUCET__AUTH__GOOGLE_CLIENT_SECRET="your-client-secret"
   export TELOXIDE_TOKEN="your-telegram-token"
   export DISCORD_TOKEN="your-discord-token"
   ```

4. 运行服务：
   ```shell
   cargo run -p web
   cargo run -p tg-bot
   cargo run -p dc-bot
   cargo run -p reporting
   ```

### 前端开发

前端项目位于 `frontend/`，默认使用 Vite + React。可通过环境变量选择后端 API 地址，不与 Rust 服务同仓部署也没问题：

```shell
cd frontend
cp .env.example .env          # 配置 VITE_API_BASE_URL 指向后端
npm install                   # 或 pnpm / yarn
npm run dev                   # 本地开发 (默认 http://localhost:5173)

npm run build                 # 生成 dist/ 静态资源，可直接托管到 CDN
```

> - `VITE_API_BASE_URL`：构建期注入的后端地址。
> - 开发期 `VITE_DEV_PROXY=1` 时，Vite 会将 `/api` 请求代理至后端。
> - 线上如需运行时覆盖，可在部署时设置 `window.__API_BASE__`（例如通过自定义脚本注入）。

## 安全注意事项

⚠️ **重要**：配置文件 `config/default.toml` 中的敏感信息已清空，请通过环境变量设置：
- 数据库连接字符串
- OAuth 客户端密钥
- Bot 令牌

不要将包含真实凭据的 `.env` 文件提交到版本控制系统。

## 下一步建议

- 在 `core::repository` 中实现 PostgreSQL 与 MongoDB 的具体仓储。
- 将队列处理逻辑与 Aptos SDK 集成，提交真实交易。
- 完善 Web 前端页面、OAuth 回调及管理端。
- 为 Bot 增加角色、限额查询与管理员配置接口。
- 编写集成测试与 CI 流程，保障多渠道协同稳定。
