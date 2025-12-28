use crate::domain::entities::PaymentOrder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 领域事件trait
pub trait DomainEvent {
    fn event_type(&self) -> &'static str;
    fn occurred_at(&self) -> DateTime<Utc>;
}

/// 支付订单创建事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentOrderCreated {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub order_id: Uuid,
    pub out_order_no: String,
    pub amount: i64,
}

impl DomainEvent for PaymentOrderCreated {
    fn event_type(&self) -> &'static str {
        "PaymentOrderCreated"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

impl PaymentOrderCreated {
    pub fn from_order(order: &PaymentOrder) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            order_id: order.id,
            out_order_no: order.out_order_no.clone(),
            amount: order.amount.to_cents(),
        }
    }
}

/// 支付成功事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSucceeded {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub order_id: Uuid,
    pub out_order_no: String,
    pub transaction_id: String,
    pub amount: i64,
}

impl DomainEvent for PaymentSucceeded {
    fn event_type(&self) -> &'static str {
        "PaymentSucceeded"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

impl PaymentSucceeded {
    pub fn from_order(order: &PaymentOrder) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            order_id: order.id,
            out_order_no: order.out_order_no.clone(),
            transaction_id: order
                .transaction_id
                .clone()
                .expect("Transaction ID must exist"),
            amount: order.amount.to_cents(),
        }
    }
}

/// 支付失败事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentFailed {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub order_id: Uuid,
    pub out_order_no: String,
    pub reason: String,
}

impl DomainEvent for PaymentFailed {
    fn event_type(&self) -> &'static str {
        "PaymentFailed"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

impl PaymentFailed {
    pub fn new(order: &PaymentOrder, reason: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            order_id: order.id,
            out_order_no: order.out_order_no.clone(),
            reason,
        }
    }
}
