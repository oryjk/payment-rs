use crate::application::dto::{CreatePaymentRequest, PaymentResponse};
use crate::domain::errors::DomainResult;
use crate::domain::PaymentOrder;
use crate::ports::PaymentRepositoryPort;
use crate::ports::WeChatPayPort;
use std::sync::Arc;
use tracing::{debug, error, info};

/// 支付服务
pub struct PaymentService<T: WeChatPayPort, R: PaymentRepositoryPort> {
    wechat_pay: Arc<T>,
    repository: Arc<R>,
}

impl<T: WeChatPayPort, R: PaymentRepositoryPort> PaymentService<T, R> {
    pub fn new(wechat_pay: Arc<T>, repository: Arc<R>) -> Self {
        Self {
            wechat_pay,
            repository,
        }
    }

    /// 创建支付订单
    pub async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> DomainResult<PaymentResponse> {
        info!("Creating payment for order: {}", request.out_order_no);

        // 1. 创建领域对象
        let mut order = PaymentOrder::new(
            request.out_order_no.clone(),
            request.amount,
            request.payment_method,
            request.description,
            request.client_ip,
            request.openid,
            request.attach,
        )?;

        // 2. 保存到数据库
        self.repository.save(&order).await?;
        debug!("Order saved to database: {}", order.id);

        // 3. 调用微信支付API
        let wechat_request = crate::ports::wechat_pay_port::WeChatPayRequest {
            out_order_no: order.out_order_no.clone(),
            description: order.description.clone(),
            amount_cents: order.amount.to_cents(),
            openid: order.openid.clone(),
            client_ip: order.client_ip.clone(),
            attach: order.attach.clone(),
        };

        let wechat_response = self
            .wechat_pay
            .create_mini_program_order(wechat_request)
            .await?;

        // 4. 更新预下单ID
        order.set_prepay_id(wechat_response.prepay_id.clone())?;
        self.repository.update(&order).await?;

        // 5. 生成小程序支付参数
        let pay_params = self
            .wechat_pay
            .generate_mini_pay_params(&wechat_response.prepay_id)
            .await?;

        info!("Payment created successfully: {}", order.id);

        Ok(PaymentResponse {
            order_id: order.id,
            out_order_no: order.out_order_no,
            amount: order.amount.to_cents(),
            prepay_id: wechat_response.prepay_id,
            pay_params: Some(pay_params),
            state: order.state.to_string(),
        })
    }

    /// 查询订单
    pub async fn query_payment(&self, out_order_no: &str) -> DomainResult<PaymentResponse> {
        info!("Querying payment: {}", out_order_no);

        // 1. 从数据库查询
        let mut order = self
            .repository
            .find_by_out_order_no(out_order_no)
            .await?
            .ok_or_else(|| {
                crate::domain::errors::DomainError::OrderNotFound(out_order_no.to_string())
            })?;

        // 2. 如果订单未完成，向微信查询最新状态
        if !order.is_finished() {
            debug!("Order not finished, querying WeChat: {}", out_order_no);
            let query_response = self.wechat_pay.query_order(out_order_no).await?;

            match query_response.trade_state.as_str() {
                "SUCCESS" => {
                    if let Some(tx_id) = query_response.transaction_id {
                        order.mark_as_succeeded(tx_id)?;
                        self.repository.update(&order).await?;
                    }
                }
                "CLOSED" => {
                    order.mark_as_closed()?;
                    self.repository.update(&order).await?;
                }
                "PAYERROR" => {
                    order.mark_as_failed()?;
                    self.repository.update(&order).await?;
                }
                _ => {
                    debug!("Order state unchanged: {}", query_response.trade_state);
                }
            }
        }

        Ok(PaymentResponse {
            order_id: order.id,
            out_order_no: order.out_order_no,
            amount: order.amount.to_cents(),
            prepay_id: order.prepay_id.unwrap_or_default(),
            pay_params: None,
            state: order.state.to_string(),
        })
    }

    /// 处理支付回调
    pub async fn handle_payment_notification(
        &self,
        notification: crate::ports::wechat_pay_port::PaymentNotification,
    ) -> DomainResult<()> {
        info!(
            "Handling payment notification for order: {}",
            notification.id
        );

        // 解密通知数据
        let decrypted = self
            .wechat_pay
            .decrypt_notification(
                &notification.resource.ciphertext,
                &notification.resource.associated_data,
                &notification.resource.nonce,
            )
            .await?;

        debug!("Decrypted notification: {}", decrypted);

        // 解析JSON
        let data: serde_json::Value = serde_json::from_str(&decrypted)?;
        let out_order_no = data["out_trade_no"]
            .as_str()
            .ok_or_else(|| {
                crate::domain::errors::DomainError::ValidationError(
                    "Missing out_trade_no in notification".to_string(),
                )
            })?
            .to_string();

        // 查找订单
        let mut order = self
            .repository
            .find_by_out_order_no(&out_order_no)
            .await?
            .ok_or_else(|| {
                crate::domain::errors::DomainError::OrderNotFound(out_order_no.clone())
            })?;

        // 更新订单状态
        match notification.event_type.as_str() {
            "TRANSACTION.SUCCESS" => {
                let transaction_id = data["transaction_id"]
                    .as_str()
                    .ok_or_else(|| {
                        crate::domain::errors::DomainError::ValidationError(
                            "Missing transaction_id in notification".to_string(),
                        )
                    })?
                    .to_string();

                order.mark_as_succeeded(transaction_id)?;
                self.repository.update(&order).await?;

                info!("Payment succeeded via notification: {}", out_order_no);
            }
            _ => {
                debug!("Unhandled notification event type: {}", notification.event_type);
            }
        }

        Ok(())
    }
}
