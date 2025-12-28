use crate::domain::errors::DomainResult;
use crate::domain::PaymentOrder;
use async_trait::async_trait;

/// 支付订单仓储端口接口
#[async_trait]
pub trait PaymentRepositoryPort: Send + Sync {
    /// 保存支付订单
    async fn save(&self, order: &PaymentOrder) -> DomainResult<()>;

    /// 根据ID查找订单
    async fn find_by_id(&self, id: uuid::Uuid) -> DomainResult<Option<PaymentOrder>>;

    /// 根据商户订单号查找
    async fn find_by_out_order_no(&self, out_order_no: &str) -> DomainResult<Option<PaymentOrder>>;

    /// 根据微信交易号查找
    async fn find_by_transaction_id(&self, transaction_id: &str)
        -> DomainResult<Option<PaymentOrder>>;

    /// 更新订单
    async fn update(&self, order: &PaymentOrder) -> DomainResult<()>;

    /// 删除订单（软删除）
    async fn delete(&self, id: uuid::Uuid) -> DomainResult<()>;
}
