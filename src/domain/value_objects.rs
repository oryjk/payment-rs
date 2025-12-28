use serde::{Deserialize, Serialize};
use std::fmt;

/// 支付状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentState {
    /// 待支付
    Pending,
    /// 支付中
    Processing,
    /// 支付成功
    Succeeded,
    /// 支付失败
    Failed,
    /// 已退款
    Refunded,
    /// 已关闭
    Closed,
}

impl fmt::Display for PaymentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentState::Pending => write!(f, "pending"),
            PaymentState::Processing => write!(f, "processing"),
            PaymentState::Succeeded => write!(f, "succeeded"),
            PaymentState::Failed => write!(f, "failed"),
            PaymentState::Refunded => write!(f, "refunded"),
            PaymentState::Closed => write!(f, "closed"),
        }
    }
}

/// 支付方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    /// 小程序支付
    MiniProgram,
    /// JSAPI支付（微信公众号/H5）
    Jsapi,
    /// Native支付（扫码）
    Native,
    /// H5支付（外部浏览器）
    H5,
}

impl fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentMethod::MiniProgram => write!(f, "mini_program"),
            PaymentMethod::Jsapi => write!(f, "jsapi"),
            PaymentMethod::Native => write!(f, "native"),
            PaymentMethod::H5 => write!(f, "h5"),
        }
    }
}

/// 货币金额（分为单位，避免浮点数精度问题）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    /// 金额（分）
    pub amount_cents: i64,
}

impl Money {
    /// 创建新的金额对象（单位：元）
    pub fn from_yuan(amount: i64) -> Self {
        Self {
            amount_cents: amount * 100,
        }
    }

    /// 创建新的金额对象（单位：分）
    pub fn from_cents(cents: i64) -> Self {
        Self { amount_cents: cents }
    }

    /// 转换为元
    pub fn to_yuan(&self) -> f64 {
        self.amount_cents as f64 / 100.0
    }

    /// 转换为分
    pub fn to_cents(&self) -> i64 {
        self.amount_cents
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "¥{:.2}", self.to_yuan())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_from_yuan() {
        let money = Money::from_yuan(10);
        assert_eq!(money.to_cents(), 1000);
        assert_eq!(money.to_yuan(), 10.0);
    }

    #[test]
    fn test_money_display() {
        let money = Money::from_yuan(10);
        assert_eq!(format!("{}", money), "¥10.00");
    }
}
