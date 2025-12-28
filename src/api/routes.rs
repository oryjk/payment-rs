use super::handlers::*;
use axum::{
    routing::{get, post},
    Router,
};

pub fn create_router<T: crate::ports::WeChatPayPort + Clone + 'static, R: crate::ports::PaymentRepositoryPort + Clone + 'static>(
    state: AppState<T, R>,
) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/payments", post(create_payment))
        .route("/api/payments/:out_order_no", get(query_payment))
        .route("/api/webhooks/wechat", post(wechat_webhook))
        .with_state(state)
}
