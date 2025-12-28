use super::handlers::*;
use axum::{
    routing::{get, post},
    Router,
};

pub fn create_router<T: crate::ports::WeChatPayPort, R: crate::ports::PaymentRepositoryPort>(
    state: AppState<T, R>,
) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/payments", post(create_payment))
        .route("/api/payments/:out_order_no", get(query_payment))
        .route("/api/webhooks/wechat", post(wechat_webhook))
        .with_state(state)
}
