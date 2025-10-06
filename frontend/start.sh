#!/bin/bash

# 水龙头服务前端启动脚本

echo "🚀 启动水龙头服务前端..."

# 检查 Node.js 是否安装
if ! command -v node &> /dev/null; then
    echo "❌ 错误: 未找到 Node.js，请先安装 Node.js"
    exit 1
fi

# 检查 pnpm 是否安装
if ! command -v pnpm &> /dev/null; then
    echo "❌ 错误: 未找到 pnpm，请先安装 pnpm"
    echo "💡 安装命令: npm install -g pnpm"
    exit 1
fi

# 检查是否存在 .env.local 文件
if [ ! -f ".env.local" ]; then
    echo "⚠️  警告: 未找到 .env.local 文件"
    echo "📝 正在从 env.example 创建 .env.local..."
    cp env.example .env.local
    echo "✅ 已创建 .env.local 文件，请编辑其中的配置"
    echo "🔧 需要设置 VITE_GOOGLE_CLIENT_ID 环境变量"
fi

# 检查是否已安装依赖
if [ ! -d "node_modules" ]; then
    echo "📦 安装依赖..."
    pnpm install
    if [ $? -ne 0 ]; then
        echo "❌ 依赖安装失败"
        exit 1
    fi
    echo "✅ 依赖安装完成"
fi

# 启动开发服务器
echo "🌐 启动开发服务器..."
echo "📍 前端地址: http://localhost:3000"
echo "🔗 后端 API: http://localhost:8080"
echo ""
echo "按 Ctrl+C 停止服务器"
echo ""

pnpm run dev
