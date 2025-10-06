# 水龙头服务前端

现代化的水龙头服务前端界面，支持 Google OAuth 登录、代币领取和管理后台。

## 功能特性

- 🔐 **Google OAuth 登录** - 安全的身份验证
- 💧 **代币领取** - 简单易用的代币领取界面
- 👑 **管理后台** - 管理员专用的用户角色管理
- 📱 **响应式设计** - 支持桌面和移动设备
- 🎨 **现代 UI** - 基于 Tailwind CSS 的美观界面

## 技术栈

- **React 18** - 用户界面框架
- **TypeScript** - 类型安全
- **Vite** - 快速构建工具
- **Tailwind CSS** - 样式框架
- **React Router** - 路由管理
- **Axios** - HTTP 客户端
- **Lucide React** - 图标库

## 快速开始

### 1. 安装依赖

```bash
npm install
```

### 2. 环境配置

复制环境变量模板：

```bash
cp env.example .env.local
```

编辑 `.env.local` 文件，设置以下变量：

```env
VITE_GOOGLE_CLIENT_ID=your_google_client_id_here
VITE_API_BASE_URL=http://localhost:8080
```

### 3. 启动开发服务器

```bash
npm run dev
```

访问 [http://localhost:3000](http://localhost:3000) 查看应用。

### 4. 构建生产版本

```bash
npm run build
```

## Google OAuth 配置

1. 访问 [Google Cloud Console](https://console.cloud.google.com/)
2. 创建新项目或选择现有项目
3. 启用 Google+ API
4. 创建 OAuth 2.0 客户端 ID
5. 将客户端 ID 添加到环境变量中

## 项目结构

```
src/
├── components/          # 可复用组件
│   ├── Layout.tsx      # 主布局组件
│   └── LoadingSpinner.tsx
├── contexts/           # React Context
│   └── AuthContext.tsx # 认证上下文
├── lib/               # 工具库
│   ├── api.ts         # API 客户端
│   └── googleAuth.ts  # Google OAuth 集成
├── pages/             # 页面组件
│   ├── HomePage.tsx   # 首页（代币领取）
│   ├── LoginPage.tsx  # 登录页面
│   └── AdminPage.tsx  # 管理后台
├── types/             # TypeScript 类型定义
│   └── index.ts
├── App.tsx            # 主应用组件
├── main.tsx           # 应用入口
└── index.css          # 全局样式
```

## API 集成

前端与后端 API 的集成点：

- `POST /api/session` - Google OAuth 登录
- `GET /api/me` - 获取当前用户信息
- `POST /api/mint` - 领取代币
- `POST /api/admin/role` - 更新用户角色（仅管理员）

## 部署

### 使用 Docker

```bash
# 构建镜像
docker build -t faucet-frontend .

# 运行容器
docker run -p 3000:3000 faucet-frontend
```

### 使用 Nginx

1. 构建生产版本：`npm run build`
2. 将 `dist` 目录内容复制到 Nginx 服务器
3. 配置 Nginx 代理 API 请求到后端服务

## 开发指南

### 添加新功能

1. 在 `src/types/index.ts` 中定义相关类型
2. 在 `src/lib/api.ts` 中添加 API 调用
3. 创建相应的组件和页面
4. 更新路由配置

### 样式指南

- 使用 Tailwind CSS 类名
- 遵循现有的设计系统
- 保持响应式设计
- 使用语义化的颜色和间距

## 故障排除

### 常见问题

1. **Google OAuth 不工作**
   - 检查 `VITE_GOOGLE_CLIENT_ID` 是否正确设置
   - 确认 Google Cloud Console 中的重定向 URI 配置

2. **API 请求失败**
   - 检查后端服务是否运行
   - 确认 `VITE_API_BASE_URL` 配置正确

3. **构建失败**
   - 检查 TypeScript 类型错误
   - 确认所有依赖已正确安装

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License
