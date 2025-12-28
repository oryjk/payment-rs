use crate::domain::errors::{DomainError, DomainResult};
use crate::infrastructure::config::wechat_config::WeChatPayConfig;
use crate::ports::wechat_pay_port::*;
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use reqwest::Client;
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::Signer;
use rsa::Pkcs1v15Sign;
use serde_json::json;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error};

type HmacSha256 = Hmac<Sha256>;

/// 微信支付适配器实现
pub struct WeChatPayAdapter {
    config: Arc<WeChatPayConfig>,
    client: Client,
}

impl WeChatPayAdapter {
    pub fn new(config: Arc<WeChatPayConfig>) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// 生成签名
    fn build_signature(
        &self,
        method: &str,
        url: &str,
        timestamp: &str,
        nonce: &str,
        body: &str,
    ) -> DomainResult<String> {
        let message = format!("{}\n{}\n{}\n{}\n{}", method, url, timestamp, nonce, body);

        // 加载私钥
        let private_key = rsa::RsaPrivateKey::from_pkcs8_pem(&self.config.private_key)
            .map_err(|e| DomainError::CryptoError(format!("Failed to load private key: {}", e)))?;

        // 创建签名器
        let mut signer =
            Pkcs1v15Sign::new::<Sha256>(private_key.as_ref());

        // 签名
        let signature = signer
            .sign(message.as_bytes())
            .to_vec();

        // Base64编码
        Ok(base64::encode(&signature))
    }

    /// 生成Authorization头
    fn build_authorization(
        &self,
        method: &str,
        url: &str,
        body: &str,
    ) -> DomainResult<String> {
        let timestamp = format!("{}", chrono::Utc::now().timestamp());
        let nonce = format!("{}", uuid::Uuid::new_v4());

        let signature = self.build_signature(method, url, &timestamp, &nonce, body)?;

        let auth = format!(
            "mchid=\"{}\",nonce_str=\"{}\",timestamp=\"{}\",serial_no=\"{}\",signature=\"{}\"",
            self.config.mchid, nonce, timestamp, self.config.serial_no, signature
        );

        let schema = "WECHATPAY2-SHA256-RSA2048";
        Ok(format!("{} {}", schema, auth))
    }

    /// 生成随机字符串
    fn generate_nonce_str() -> String {
        uuid::Uuid::new_v4().to_string().replace("-", "")
    }

    /// 解密回调数据
    fn decrypt_callback_data(
        &self,
        ciphertext: &str,
        associated_data: &str,
        nonce: &str,
    ) -> DomainResult<String> {
        let key = &self.config.api_v3_key;

        // AES-256-GCM解密
        let key_bytes = key.as_bytes();
        let nonce_bytes = nonce.as_bytes();
        let ciphertext_bytes = &base64::decode(ciphertext)
            .map_err(|e| DomainError::CryptoError(format!("Base64 decode error: {}", e)))?;

        // 使用aes-gcm crate进行解密
        use aes_gcm::{
            aead::{Aead, AeadCore, KeyInit, OsRng},
            Aes256Gcm, Nonce,
        };

        let cipher_key = Aes256Gcm::new_from_slice(key_bytes)
            .map_err(|e| DomainError::CryptoError(format!("AES init error: {}", e)))?;

        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher_key
            .decrypt(nonce, ciphertext_bytes.as_ref())
            .map_err(|e| DomainError::CryptoError(format!("Decrypt error: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| DomainError::CryptoError(format!("UTF8 decode error: {}", e)))
    }
}

#[async_trait]
impl WeChatPayPort for WeChatPayAdapter {
    /// 创建小程序支付订单
    async fn create_mini_program_order(
        &self,
        request: WeChatPayRequest,
    ) -> DomainResult<WeChatPayResponse> {
        let url = format!("{}/v3/pay/transactions/jsapi", self.config.base_url);

        let body = json!({
            "appid": self.config.appid,
            "mchid": self.config.mchid,
            "description": request.description,
            "out_trade_no": request.out_order_no,
            "notify_url": format!("{}/api/webhooks/wechat", std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())),
            "amount": {
                "total": request.amount_cents,
                "currency": "CNY"
            },
            "payer": {
                "openid": request.openid.ok_or_else(|| DomainError::ValidationError("OpenID is required for mini program payment".to_string()))?
            },
            "scene_info": {
                "payer_client_ip": request.client_ip
            }
        });

        let body_str = body.to_string();
        debug!("WeChat pay request body: {}", body_str);

        let authorization = self.build_authorization("POST", "/v3/pay/transactions/jsapi", &body_str)?;

        let response = self
            .client
            .post(&url)
            .header("Authorization", authorization)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(body_str)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("WeChat pay API error: {} - {}", status, error_text);
            return Err(DomainError::WeChatPayError(format!(
                "API returned {}: {}",
                status, error_text
            )));
        }

        let resp_json: serde_json::Value = response.json().await?;
        debug!("WeChat pay response: {}", resp_json);

        let prepay_id = resp_json["prepay_id"]
            .as_str()
            .ok_or_else(|| DomainError::WeChatPayError("Missing prepay_id".to_string()))?;

        Ok(WeChatPayResponse {
            prepay_id: prepay_id.to_string(),
        })
    }

    /// 生成小程序支付参数
    async fn generate_mini_pay_params(
        &self,
        prepay_id: &str,
    ) -> DomainResult<MiniProgramPayParams> {
        let timestamp = format!("{}", chrono::Utc::now().timestamp());
        let nonce_str = Self::generate_nonce_str();
        let package = format!("prepay_id={}", prepay_id);

        let message = format!(
            "{}\n{}\n{}\n{}\n{}",
            self.config.appid, timestamp, nonce_str, package, ""
        );

        // 使用私钥签名
        let private_key = rsa::RsaPrivateKey::from_pkcs8_pem(&self.config.private_key)
            .map_err(|e| DomainError::CryptoError(format!("Failed to load private key: {}", e)))?;

        let mut signer = Pkcs1v15Sign::new::<Sha256>(private_key.as_ref());
        let signature = signer.sign(message.as_bytes()).to_vec();
        let pay_sign = base64::encode(&signature);

        Ok(MiniProgramPayParams {
            time_stamp: timestamp,
            nonce_str,
            package,
            sign_type: "RSA".to_string(),
            pay_sign,
        })
    }

    /// 查询订单
    async fn query_order(&self, out_order_no: &str) -> DomainResult<OrderQueryResponse> {
        let url = format!(
            "{}/v3/pay/transactions/out-trade-no/{}?mchid={}",
            self.config.base_url, out_order_no, self.config.mchid
        );

        let authorization =
            self.build_authorization("GET", &url, "")?;

        let response = self
            .client
            .get(&url)
            .header("Authorization", authorization)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(DomainError::WeChatPayError(format!(
                "Query order failed: {} - {}",
                status, error_text
            )));
        }

        let resp_json: serde_json::Value = response.json().await?;

        Ok(OrderQueryResponse {
            trade_state: resp_json["trade_state"]
                .as_str()
                .unwrap_or("UNKNOWN")
                .to_string(),
            transaction_id: resp_json["transaction_id"].as_str().map(String::from),
            trade_state_desc: resp_json["trade_state_desc"].as_str().map(String::from),
        })
    }

    /// 关闭订单
    async fn close_order(&self, out_order_no: &str) -> DomainResult<()> {
        let url = format!(
            "{}/v3/pay/transactions/out-trade-no/{}/close",
            self.config.base_url, out_order_no
        );

        let body = json!({ "mchid": self.config.mchid });
        let body_str = body.to_string();

        let authorization =
            self.build_authorization("POST", &url.replace(self.config.base_url, ""), &body_str)?;

        let response = self
            .client
            .post(&url)
            .header("Authorization", authorization)
            .header("Content-Type", "application/json")
            .body(body_str)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(DomainError::WeChatPayError(format!(
                "Close order failed: {} - {}",
                status, error_text
            )));
        }

        Ok(())
    }

    /// 验证回调通知签名
    async fn verify_notification(
        &self,
        timestamp: &str,
        nonce: &str,
        body: &str,
        signature: &str,
    ) -> DomainResult<bool> {
        let message = format!("{}\n{}\n{}\n{}", timestamp, nonce, body, "");

        // 使用微信支付平台证书公钥验证签名
        // 这里需要加载微信支付平台证书，暂时返回true
        // TODO: 实现完整的签名验证
        debug!("Signature verification for message: {}", message);
        Ok(true)
    }

    /// 解密回调通知
    async fn decrypt_notification(
        &self,
        ciphertext: &str,
        associated_data: &str,
        nonce: &str,
    ) -> DomainResult<String> {
        self.decrypt_callback_data(ciphertext, associated_data, nonce)
    }
}
