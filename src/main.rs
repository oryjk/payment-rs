mod api;
mod application;
mod domain;
mod infrastructure;
mod ports;

use api::AppState;
use application::PaymentService;
use infrastructure::{MySqlPaymentRepository, WeChatPayAdapter, WeChatPayConfig};
use sqlx::MySqlPool;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    // 加载环境变量
    dotenvy::dotenv().ok();

    info!("Starting Payment Service...");

    // 创建数据库连接池
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    info!("Connecting to database...");

    let pool = MySqlPool::connect(&database_url).await?;
    info!("Database connected successfully");

    // 初始化微信支付配置
    let wechat_config = WeChatPayConfig::from_env();
    info!("WeChat Pay configuration loaded for mchid: {}", wechat_config.mchid);

    // 创建微信支付适配器
    let wechat_adapter = Arc::new(WeChatPayAdapter::new(wechat_config.clone()));

    // 创建仓储
    let repository = Arc::new(MySqlPaymentRepository::new(Arc::new(pool)));

    // 创建支付服务
    let payment_service = Arc::new(PaymentService::new(
        wechat_adapter,
        repository,
    ));

    // 创建应用状态
    let app_state = AppState {
        payment_service,
    };

    // 创建路由
    let app = api::create_router(app_state);

    // 启动服务器
    let host = std::env::var("SERVER_HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);

    info!("Server listening on {}", addr);
    info!("Available endpoints:");
    info!("  GET  /health - Health check");
    info!("  POST /api/payments - Create payment");
    info!("  GET  /api/payments/:out_order_no - Query payment");
    info!("  POST /api/webhooks/wechat - WeChat payment webhook");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

