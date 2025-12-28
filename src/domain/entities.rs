use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{Money, PaymentMethod, PaymentState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 支付订单实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentOrder {
    /// 订单ID（内部）
    pub id: Uuid,

    /// 商户订单号
    pub out_order_no: String,

    /// 微信支付订单号（支付后返回）
    pub transaction_id: Option<String>,

    /// 支付金额
    pub amount: Money,

    /// 支付方式
    pub payment_method: PaymentMethod,

    /// 支付状态
    pub state: PaymentState,

    /// 商品描述
    pub description: String,

    /// 用户OpenID（小程序支付时需要）
    pub openid: Option<String>,

    /// 客户端IP
    pub client_ip: String,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 支付完成时间
    pub paid_at: Option<DateTime<Utc>>,

    /// 附加数据
    pub attach: Option<String>,

    /// 微信支付预下单ID
    pub prepay_id: Option<String>,
}

impl PaymentOrder {
    /// 创建新的支付订单
    pub fn new(
        out_order_no: String,
        amount: Money,
        payment_method: PaymentMethod,
        description: String,
        client_ip: String,
        openid: Option<String>,
        attach: Option<String>,
    ) -> DomainResult<Self> {
        // 验证金额
        if amount.to_cents() <= 0 {
            return Err(DomainError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // 验证商户订单号
        if out_order_no.is_empty() || out_order_no.len() > 64 {
            return Err(DomainError::ValidationError(
                "Out order no must be 1-64 characters".to_string(),
            ));
        }

        // 验证描述
        if description.is_empty() || description.len() > 127 {
            return Err(DomainError::ValidationError(
                "Description must be 1-127 characters".to_string(),
            ));
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            out_order_no,
            transaction_id: None,
            amount,
            payment_method,
            state: PaymentState::Pending,
            description,
            openid,
            client_ip,
            created_at: now,
            updated_at: now,
            paid_at: None,
            attach,
            prepay_id: None,
        })
    }

    /// 更新为处理中状态
    pub fn mark_as_processing(&mut self) -> DomainResult<()> {
        if self.state != PaymentState::Pending {
            return Err(DomainError::InvalidState {
                expected: PaymentState::Pending.to_string(),
                actual: self.state.to_string(),
            });
        }

        self.state = PaymentState::Processing;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 标记为支付成功
    pub fn mark_as_succeeded(&mut self, transaction_id: String) -> DomainResult<()> {
        if self.state != PaymentState::Processing && self.state != PaymentState::Pending {
            return Err(DomainError::InvalidState {
                expected: "processing or pending".to_string(),
                actual: self.state.to_string(),
            });
        }

        self.state = PaymentState::Succeeded;
        self.transaction_id = Some(transaction_id);
        self.paid_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 标记为支付失败
    pub fn mark_as_failed(&mut self) -> DomainResult<()> {
        if self.state != PaymentState::Processing && self.state != PaymentState::Pending {
            return Err(DomainError::InvalidState {
                expected: "processing or pending".to_string(),
                actual: self.state.to_string(),
            });
        }

        self.state = PaymentState::Failed;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 标记为已关闭
    pub fn mark_as_closed(&mut self) -> DomainResult<()> {
        if self.state == PaymentState::Succeeded || self.state == PaymentState::Refunded {
            return Err(DomainError::InvalidState {
                expected: "pending or processing or failed".to_string(),
                actual: self.state.to_string(),
            });
        }

        self.state = PaymentState::Closed;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 设置预下单ID
    pub fn set_prepay_id(&mut self, prepay_id: String) -> DomainResult<()> {
        self.prepay_id = Some(prepay_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 检查是否可以支付
    pub fn can_pay(&self) -> bool {
        self.state == PaymentState::Pending
    }

    /// 检查是否已完成（成功或失败）
    pub fn is_finished(&self) -> bool {
        matches!(
            self.state,
            PaymentState::Succeeded | PaymentState::Failed | PaymentState::Closed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_payment_order() {
        let order = PaymentOrder::new(
            "ORDER123".to_string(),
            Money::from_yuan(10),
            PaymentMethod::MiniProgram,
            "测试商品".to_string(),
            "127.0.0.1".to_string(),
            Some("openid123".to_string()),
            None,
        )
        .unwrap();

        assert_eq!(order.state, PaymentState::Pending);
        assert_eq!(order.amount.to_cents(), 1000);
        assert!(order.can_pay());
        assert!(!order.is_finished());
    }

    #[test]
    fn test_mark_as_succeeded() {
        let mut order = PaymentOrder::new(
            "ORDER123".to_string(),
            Money::from_yuan(10),
            PaymentMethod::MiniProgram,
            "测试商品".to_string(),
            "127.0.0.1".to_string(),
            Some("openid123".to_string()),
            None,
        )
        .unwrap();

        order.mark_as_succeeded("TX123".to_string()).unwrap();

        assert_eq!(order.state, PaymentState::Succeeded);
        assert_eq!(order.transaction_id, Some("TX123".to_string()));
        assert!(order.paid_at.is_some());
        assert!(order.is_finished());
    }

    #[test]
    fn test_invalid_amount() {
        let result = PaymentOrder::new(
            "ORDER123".to_string(),
            Money::from_cents(0),
            PaymentMethod::MiniProgram,
            "测试商品".to_string(),
            "127.0.0.1".to_string(),
            Some("openid123".to_string()),
            None,
        );

        assert!(result.is_err());
    }
}
