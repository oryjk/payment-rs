use crate::application::{ErrorResponse, PaymentService};
use crate::ports::wechat_pay_port::PaymentNotification;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tracing::{error, info};

/// 应用状态
#[derive(Clone)]
pub struct AppState<T: crate::ports::WeChatPayPort, R: crate::ports::PaymentRepositoryPort> {
    pub payment_service: std::sync::Arc<PaymentService<T, R>>,
}

impl<T: crate::ports::WeChatPayPort, R: crate::ports::PaymentRepositoryPort> Clone
    for AppState<T, R>
{
    fn clone(&self) -> Self {
        Self {
            payment_service: self.payment_service.clone(),
        }
    }
}

/// 创建支付订单
pub async fn create_payment<T: crate::ports::WeChatPayPort, R: crate::ports::PaymentRepositoryPort>(
    State(state): State<AppState<T, R>>,
    Json(request): Json<crate::application::CreatePaymentRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    info!("Received payment creation request: {}", request.out_order_no);

    state
        .payment_service
        .create_payment(request)
        .await
        .map(|response| (StatusCode::CREATED, Json(response)).into_response())
        .map_err(|e| {
            error!("Payment creation error: {}", e);
            let status = match e {
                crate::domain::errors::DomainError::ValidationError(_) => StatusCode::BAD_REQUEST,
                crate::domain::errors::InvalidAmount(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (
                status,
                Json(ErrorResponse::new(
                    "PAYMENT_ERROR".to_string(),
                    e.to_string(),
                )),
            )
        })
}

/// 查询订单
pub async fn query_payment<T: crate::ports::WeChatPayPort, R: crate::ports::PaymentRepositoryPort>(
    State(state): State<AppState<T, R>>,
    Path(out_order_no): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    info!("Received payment query request: {}", out_order_no);

    state
        .payment_service
        .query_payment(&out_order_no)
        .await
        .map(|response| (StatusCode::OK, Json(response)).into_response())
        .map_err(|e| {
            error!("Payment query error: {}", e);
            let status = match e {
                crate::domain::errors::DomainError::OrderNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (
                status,
                Json(ErrorResponse::new(
                    "QUERY_ERROR".to_string(),
                    e.to_string(),
                )),
            )
        })
}

/// 微信支付回调
pub async fn wechat_webhook<
    T: crate::ports::WeChatPayPort,
    R: crate::ports::PaymentRepositoryPort,
>(
    State(state): State<AppState<T, R>>,
    headers: axum::http::HeaderMap,
    body: String,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    info!("Received WeChat payment webhook");

    // 提取签名头
    let timestamp = headers
        .get("Wechatpay-Timestamp")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "INVALID_SIGNATURE".to_string(),
                    "Missing Wechatpay-Timestamp".to_string(),
                )),
            )
        })?;

    let nonce = headers
        .get("Wechatpay-Nonce")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "INVALID_SIGNATURE".to_string(),
                    "Missing Wechatpay-Nonce".to_string(),
                )),
            )
        })?;

    let signature = headers
        .get("Wechatpay-Signature")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "INVALID_SIGNATURE".to_string(),
                    "Missing Wechatpay-Signature".to_string(),
                )),
            )
        })?;

    // TODO: 实现签名验证
    // 实际应用中必须验证签名以防止伪造请求
    debug!("Webhook signature verification skipped (TODO: implement)");

    // 解析通知
    let notification: PaymentNotification = serde_json::from_str(&body).map_err(|e| {
        error!("Failed to parse notification: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_REQUEST".to_string(),
                format!("Failed to parse notification: {}", e),
            )),
        )
    })?;

    // 处理通知
    state
        .payment_service
        .handle_payment_notification(notification)
        .await
        .map(|_| {
            // 返回微信要求的响应格式
            let response = serde_json::json!({
                "code": "SUCCESS",
                "message": "成功"
            });
            (StatusCode::OK, axum::Json(response)).into_response()
        })
        .map_err(|e| {
            error!("Webhook handling error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "WEBHOOK_ERROR".to_string(),
                    e.to_string(),
                )),
            )
        })
}

/// 健康检查
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}
