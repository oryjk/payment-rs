# 安装和配置指南

## 步骤1: 创建数据库

使用MySQL客户端连接到数据库并执行设置脚本：

### 方法A: 使用命令行

```bash
mysql -h 117.72.164.211 -u root -p < setup.sql
```

输入密码：`beifa888`

### 方法B: 使用图形化工具

1. 使用 Navicat、MySQL Workbench 或其他工具连接到：
   - 主机：117.72.164.211
   - 用户：root
   - 密码：beifa888

2. 执行 `setup.sql` 文件中的SQL语句

## 步骤2: 配置微信支付参数

编辑 `.env` 文件，填入你的微信支付配置：

### 获取微信支付配置

1. **APPID**: 微信公众平台获取
2. **商户号 (MCHID)**: 微信商户平台获取
3. **商户API私钥**:
   - 登录微信商户平台
   - 账户中心 > API安全 > API证书
   - 下载证书，打开 apiclient_key.pem 文件
   - 将内容（包括 BEGIN/END PRIVATE KEY）复制到 WECHAT_PRIVATE_KEY

4. **证书序列号 (SERIAL_NO)**:
   - 在商户平台API证书页面查看
   - 或使用命令：`openssl x509 -in apiclient_cert.pem -noout -serial`

5. **API v3密钥**:
   - 商户平台 > 账户中心 > API安全 > API v3密钥
   - 设置一个32位的字符串

### .env 配置示例

```env
WECHAT_APPID=wx1234567890abcdef
WECHAT_MCHID=1234567890
WECHAT_SERIAL_NO=1234567890ABCDEF
WECHAT_PRIVATE_KEY=-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC...
-----END PRIVATE KEY-----
WECHAT_API_V3_KEY=your32characterapikeyv3key123456789
```

## 步骤3: 编译和运行

### 编译项目

```bash
# 检查代码
cargo check

# 编译项目
cargo build
```

### 运行服务

```bash
cargo run
```

服务启动后，你会看到：

```
Starting Payment Service...
Connecting to database...
Database connected successfully
WeChat Pay configuration loaded for mchid: 1234567890
Server listening on 0.0.0.0:3000
Available endpoints:
  GET  /health - Health check
  POST /api/payments - Create payment
  GET  /api/payments/:out_order_no - Query payment
  POST /api/webhooks/wechat - WeChat payment webhook
```

## 步骤4: 测试API

### 健康检查

```bash
curl http://localhost:3000/health
```

预期响应：
```json
{
  "status": "ok"
}
```

### 创建支付订单

```bash
curl -X POST http://localhost:3000/api/payments \
  -H "Content-Type: application/json" \
  -d '{
    "out_order_no": "TEST'$(date +%s)'",
    "amount": {
      "amount_cents": 100
    },
    "payment_method": "mini_program",
    "description": "测试商品",
    "openid": "test_openid",
    "client_ip": "127.0.0.1"
  }'
```

## 常见问题

### Q: 编译错误 "rsa package not found"
```bash
cargo clean
cargo build
```

### Q: 数据库连接失败
- 检查 `.env` 中的 DATABASE_URL 是否正确
- 确保数据库已经创建
- 检查防火墙设置

### Q: 微信支付API调用失败
- 确认私钥格式正确（包含 BEGIN/END 行）
- 检查商户号和APPID是否匹配
- 确认API v3密钥正确

### Q: 签名验证失败
- 确认私钥是正确的apiclient_key.pem内容
- 检查系统时间是否准确

## 安全建议

1. **生产环境必须**：
   - 使用HTTPS
   - 启用回调签名验证
   - 不要将.env提交到版本控制
   - 使用环境变量或密钥管理服务

2. **建议**：
   - 设置合理的超时时间
   - 实施速率限制
   - 记录所有交易日志
   - 定期备份数据库

## 下一步

- 查看生成的文档，了解如何集成到小程序
- 实现退款功能
- 添加支付通知的签名验证
- 添加单元测试和集成测试
