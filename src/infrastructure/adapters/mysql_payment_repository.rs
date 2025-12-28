use crate::domain::errors::DomainResult;
use crate::domain::PaymentOrder;
use crate::ports::payment_repository_port::PaymentRepositoryPort;
use async_trait::async_trait;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use tracing::{debug, error};

/// MySQL支付订单仓储实现
#[derive(Clone)]
pub struct MySqlPaymentRepository {
    pool: Arc<Pool<MySql>>,
}

impl MySqlPaymentRepository {
    pub fn new(pool: Arc<Pool<MySql>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentRepositoryPort for MySqlPaymentRepository {
    /// 保存支付订单
    async fn save(&self, order: &PaymentOrder) -> DomainResult<()> {
        let query = r#"
            INSERT INTO payment_orders (
                id, out_order_no, transaction_id, amount_cents,
                payment_method, state, description, openid,
                client_ip, created_at, updated_at, paid_at,
                attach, prepay_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(order.id)
            .bind(&order.out_order_no)
            .bind(&order.transaction_id)
            .bind(order.amount.to_cents())
            .bind(order.payment_method.to_string())
            .bind(order.state.to_string())
            .bind(&order.description)
            .bind(&order.openid)
            .bind(&order.client_ip)
            .bind(order.created_at)
            .bind(order.updated_at)
            .bind(order.paid_at)
            .bind(&order.attach)
            .bind(&order.prepay_id)
            .execute(self.pool.as_ref())
            .await?;

        debug!("Payment order saved: {}", order.id);
        Ok(())
    }

    /// 根据ID查找订单
    async fn find_by_id(&self, id: uuid::Uuid) -> DomainResult<Option<PaymentOrder>> {
        let query = r#"
            SELECT id, out_order_no, transaction_id, amount_cents,
                   payment_method, state, description, openid,
                   client_ip, created_at, updated_at, paid_at,
                   attach, prepay_id
            FROM payment_orders
            WHERE id = ?
        "#;

        let result = sqlx::query_as::<_, PaymentOrderRow>(query)
            .bind(id)
            .fetch_optional(self.pool.as_ref())
            .await?;

        Ok(result.map(|row| row.into_order()))
    }

    /// 根据商户订单号查找
    async fn find_by_out_order_no(&self, out_order_no: &str) -> DomainResult<Option<PaymentOrder>> {
        let query = r#"
            SELECT id, out_order_no, transaction_id, amount_cents,
                   payment_method, state, description, openid,
                   client_ip, created_at, updated_at, paid_at,
                   attach, prepay_id
            FROM payment_orders
            WHERE out_order_no = ?
        "#;

        let result = sqlx::query_as::<_, PaymentOrderRow>(query)
            .bind(out_order_no)
            .fetch_optional(self.pool.as_ref())
            .await?;

        Ok(result.map(|row| row.into_order()))
    }

    /// 根据微信交易号查找
    async fn find_by_transaction_id(
        &self,
        transaction_id: &str,
    ) -> DomainResult<Option<PaymentOrder>> {
        let query = r#"
            SELECT id, out_order_no, transaction_id, amount_cents,
                   payment_method, state, description, openid,
                   client_ip, created_at, updated_at, paid_at,
                   attach, prepay_id
            FROM payment_orders
            WHERE transaction_id = ?
        "#;

        let result = sqlx::query_as::<_, PaymentOrderRow>(query)
            .bind(transaction_id)
            .fetch_optional(self.pool.as_ref())
            .await?;

        Ok(result.map(|row| row.into_order()))
    }

    /// 更新订单
    async fn update(&self, order: &PaymentOrder) -> DomainResult<()> {
        let query = r#"
            UPDATE payment_orders
            SET transaction_id = ?, state = ?, updated_at = ?, paid_at = ?, prepay_id = ?
            WHERE id = ?
        "#;

        let rows_affected = sqlx::query(query)
            .bind(&order.transaction_id)
            .bind(order.state.to_string())
            .bind(order.updated_at)
            .bind(order.paid_at)
            .bind(&order.prepay_id)
            .bind(order.id)
            .execute(self.pool.as_ref())
            .await?
            .rows_affected();

        if rows_affected == 0 {
            error!("No order found to update: {}", order.id);
            return Err(crate::domain::errors::DomainError::OrderNotFound(
                order.id.to_string(),
            ));
        }

        debug!("Payment order updated: {}", order.id);
        Ok(())
    }

    /// 删除订单（软删除）
    async fn delete(&self, id: uuid::Uuid) -> DomainResult<()> {
        let query = "DELETE FROM payment_orders WHERE id = ?";

        let rows_affected = sqlx::query(query)
            .bind(id)
            .execute(self.pool.as_ref())
            .await?
            .rows_affected();

        if rows_affected == 0 {
            return Err(crate::domain::errors::DomainError::OrderNotFound(
                id.to_string(),
            ));
        }

        debug!("Payment order deleted: {}", id);
        Ok(())
    }
}

/// 数据库行结构体
#[derive(Debug, sqlx::FromRow)]
struct PaymentOrderRow {
    id: uuid::Uuid,
    out_order_no: String,
    transaction_id: Option<String>,
    amount_cents: i64,
    payment_method: String,
    state: String,
    description: String,
    openid: Option<String>,
    client_ip: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    paid_at: Option<chrono::DateTime<chrono::Utc>>,
    attach: Option<String>,
    prepay_id: Option<String>,
}

impl PaymentOrderRow {
    fn into_order(self) -> PaymentOrder {
        use crate::domain::value_objects::{Money, PaymentMethod, PaymentState};

        let payment_method = match self.payment_method.as_str() {
            "mini_program" => PaymentMethod::MiniProgram,
            "jsapi" => PaymentMethod::Jsapi,
            "native" => PaymentMethod::Native,
            "h5" => PaymentMethod::H5,
            _ => panic!("Invalid payment method: {}", self.payment_method),
        };

        let state = match self.state.as_str() {
            "pending" => PaymentState::Pending,
            "processing" => PaymentState::Processing,
            "succeeded" => PaymentState::Succeeded,
            "failed" => PaymentState::Failed,
            "refunded" => PaymentState::Refunded,
            "closed" => PaymentState::Closed,
            _ => panic!("Invalid payment state: {}", self.state),
        };

        PaymentOrder {
            id: self.id,
            out_order_no: self.out_order_no,
            transaction_id: self.transaction_id,
            amount: Money::from_cents(self.amount_cents),
            payment_method,
            state,
            description: self.description,
            openid: self.openid,
            client_ip: self.client_ip,
            created_at: self.created_at,
            updated_at: self.updated_at,
            paid_at: self.paid_at,
            attach: self.attach,
            prepay_id: self.prepay_id,
        }
    }
}
