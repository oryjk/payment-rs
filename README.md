# Payment-RS - 微信支付集成服务

基于 Rust 和端口适配器架构的微信支付服务。

## 架构设计

本项目采用**端口适配器模式（六边形架构）**，具有以下特点：

- **清晰的分层**：领域层完全独立，不依赖外部框架
- **依赖注入**：通过 trait 实现松耦合
- **易于测试**：每层都可以独立测试
- **易于扩展**：可以轻松添加新的支付渠道

### 架构层次

```
├── domain/          # 领域层（核心业务逻辑）
├── ports/           # 端口接口定义
├── infrastructure/  # 基础设施（适配器实现）
├── application/     # 应用服务层
└── api/             # Web API接口层
```

## 功能特性

- ✅ 小程序支付（JSAPI支付）
- ✅ 订单查询
- ✅ 签名生成和验证
- ✅ 回调通知处理
- ✅ MySQL数据持久化
- ✅ RESTful API

## 技术栈

- **Web框架**: Axum 0.7
- **数据库**: MySQL + SQLx
- **异步运行时**: Tokio
- **加密**: RSA, AES-256-GCM, HMAC-SHA256
- **日志**: tracing

## 快速开始

### 1. 准备工作

确保你已经在微信支付平台注册了商户号，并获取以下信息：

- 商户号 (mchid)
- 商户API私钥 (private_key)
- 商户API证书序列号 (serial_no)
- APPID
- API v3密钥 (api_v3_key)

### 2. 数据库准备

连接到你的MySQL服务器并创建数据库：

```bash
mysql -h 117.72.164.211 -u root -p
```

```sql
CREATE DATABASE payment_db CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
```

执行迁移脚本：

```bash
mysql -h 117.72.164.211 -u root -p payment_db < migrations/001_create_payment_orders.sql
```

### 3. 配置环境变量

复制配置模板：

```bash
cp .env.example .env
```

编辑 `.env` 文件，填入你的配置信息：

```env
DATABASE_URL=mysql://root:beifa888@117.72.164.211:3306/payment_db

WECHAT_APPID=your_appid
WECHAT_MCHID=your_mchid
WECHAT_SERIAL_NO=your_serial_no
WECHAT_PRIVATE_KEY=-----BEGIN PRIVATE KEY-----
your_private_key_content
-----END PRIVATE KEY-----
WECHAT_API_V3_key=your_api_v3_key

SERVER_HOST=0.0.0.0
SERVER_PORT=3000
BASE_URL=http://your-domain.com
```

### 4. 运行服务

```bash
cargo run
```

服务将在 `http://localhost:3000` 启动。

## API 接口

### 创建支付订单

```http
POST /api/payments
Content-Type: application/json

{
  "out_order_no": "ORDER20231227001",
  "amount": {
    "amount_cents": 1000
  },
  "payment_method": "mini_program",
  "description": "测试商品",
  "openid": "user_openid",
  "client_ip": "127.0.0.1"
}
```

响应：

```json
{
  "order_id": "uuid",
  "out_order_no": "ORDER20231227001",
  "amount": 1000,
  "prepay_id": "wx...",
  "pay_params": {
    "time_stamp": "1703637600",
    "nonce_str": "...",
    "package": "prepay_id=wx...",
    "sign_type": "RSA",
    "pay_sign": "..."
  },
  "state": "pending"
}
```

### 查询订单

```http
GET /api/payments/ORDER20231227001
```

### 微信支付回调

```http
POST /api/webhooks/wechat
```

## 项目结构

```
payment-rs/
├── src/
│   ├── domain/              # 领域层
│   │   ├── entities.rs      # 实体
│   │   ├── value_objects.rs # 值对象
│   │   ├── errors.rs        # 错误类型
│   │   └── events.rs        # 领域事件
│   ├── ports/               # 端口接口
│   │   ├── wechat_pay_port.rs
│   │   └── payment_repository_port.rs
│   ├── infrastructure/      # 基础设施
│   │   ├── adapters/
│   │   │   ├── wechat_pay_adapter.rs
│   │   │   └── mysql_payment_repository.rs
│   │   └── config/
│   │       └── wechat_config.rs
│   ├── application/         # 应用层
│   │   ├── payment_service.rs
│   │   └── dto.rs
│   ├── api/                 # API层
│   │   ├── handlers.rs
│   │   └── routes.rs
│   └── main.rs
├── migrations/              # 数据库迁移
│   └── 001_create_payment_orders.sql
├── Cargo.toml
└── README.md
```

## 开发指南

### 添加新的支付方式

1. 在 `domain/value_objects.rs` 中添加新的支付方式
2. 在 `ports/wechat_pay_port.rs` 中添加新的接口方法
3. 在 `infrastructure/adapters/wechat_pay_adapter.rs` 中实现
4. 在 `application/payment_service.rs` 中添加业务逻辑

### 测试

```bash
# 运行单元测试
cargo test

# 运行集成测试
cargo test --test '*'

# 检查代码
cargo clippy

# 格式化代码
cargo fmt
```

## 安全建议

1. **永远不要**将 `.env` 文件提交到版本控制
2. 在生产环境中**必须**验证微信支付回调签名
3. 使用 HTTPS 保护所有API端点
4. 定期更新依赖包
5. 实施速率限制防止滥用

## 许可证

MIT License
