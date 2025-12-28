use crate::domain::value_objects::{Money, PaymentMethod};
use crate::ports::wechat_pay_port::MiniProgramPayParams;
use serde::{Deserialize, Serialize};

/// 创建支付请求
#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    /// 商户订单号
    pub out_order_no: String,

    /// 支付金额（分）
    pub amount: Money,

    /// 支付方式
    pub payment_method: PaymentMethod,

    /// 商品描述
    pub description: String,

    /// 用户OpenID（小程序支付时必填）
    pub openid: Option<String>,

    /// 客户端IP
    pub client_ip: String,

    /// 附加数据
    pub attach: Option<String>,
}

/// 支付响应
#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    /// 订单ID
    pub order_id: uuid::Uuid,

    /// 商户订单号
    pub out_order_no: String,

    /// 支付金额（分）
    pub amount: i64,

    /// 预下单ID
    pub prepay_id: String,

    /// 小程序支付参数（仅小程序支付时返回）
    pub pay_params: Option<MiniProgramPayParams>,

    /// 订单状态
    pub state: String,
}

/// 错误响应
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(error: String, message: String) -> Self {
        Self { error, message }
    }
}
