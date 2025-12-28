use crate::domain::errors::DomainResult;
use crate::domain::PaymentOrder;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 微信支付请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatPayRequest {
    pub out_order_no: String,
    pub description: String,
    pub amount_cents: i64,
    pub openid: Option<String>,
    pub client_ip: String,
    pub attach: Option<String>,
}

/// 微信支付响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatPayResponse {
    pub prepay_id: String,
}

/// 小程序支付参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniProgramPayParams {
    pub time_stamp: String,
    pub nonce_str: String,
    pub package: String,
    pub sign_type: String,
    pub pay_sign: String,
}

/// 查询订单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderQueryResponse {
    pub trade_state: String,
    pub transaction_id: Option<String>,
    pub trade_state_desc: Option<String>,
}

/// 回调通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentNotification {
    pub id: String,
    pub event_type: String,
    pub resource: NotificationResource,
    pub create_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResource {
    pub algorithm: String,
    pub ciphertext: String,
    pub nonce: String,
    pub associated_data: String,
}

/// 微信支付端口接口
#[async_trait]
pub trait WeChatPayPort: Send + Sync + Clone {
    /// 创建支付订单（小程序支付）
    async fn create_mini_program_order(
        &self,
        request: WeChatPayRequest,
    ) -> DomainResult<WeChatPayResponse>;

    /// 生成小程序支付参数
    async fn generate_mini_pay_params(
        &self,
        prepay_id: &str,
    ) -> DomainResult<MiniProgramPayParams>;

    /// 查询订单
    async fn query_order(&self, out_order_no: &str) -> DomainResult<OrderQueryResponse>;

    /// 关闭订单
    async fn close_order(&self, out_order_no: &str) -> DomainResult<()>;

    /// 验证回调通知签名
    async fn verify_notification(
        &self,
        timestamp: &str,
        nonce: &str,
        body: &str,
        signature: &str,
    ) -> DomainResult<bool>;

    /// 解密回调通知
    async fn decrypt_notification(
        &self,
        ciphertext: &str,
        associated_data: &str,
        nonce: &str,
    ) -> DomainResult<String>;
}
