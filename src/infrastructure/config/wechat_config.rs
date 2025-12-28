use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 微信支付配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatPayConfig {
    /// 商户号
    pub mchid: String,

    /// 微信支付公钥ID
    pub serial_no: String,

    /// 商户API私钥路径（PKCS#8格式）
    pub private_key_path: String,

    /// 商户API私钥内容
    pub private_key: String,

    /// 商户API v3密钥（用于回调通知解密）
    pub api_v3_key: String,

    /// APPID
    pub appid: String,

    /// API基础URL
    pub base_url: String,
}

impl WeChatPayConfig {
    pub fn from_env() -> Arc<Self> {
        Arc::new(Self {
            mchid: std::env::var("WECHAT_MCHID")
                .expect("WECHAT_MCHID must be set"),
            serial_no: std::env::var("WECHAT_SERIAL_NO")
                .expect("WECHAT_SERIAL_NO must be set"),
            private_key_path: std::env::var("WECHAT_PRIVATE_KEY_PATH")
                .unwrap_or_else(|_| String::new()),
            private_key: std::env::var("WECHAT_PRIVATE_KEY")
                .expect("WECHAT_PRIVATE_KEY must be set"),
            api_v3_key: std::env::var("WECHAT_API_V3_KEY")
                .expect("WECHAT_API_V3_KEY must be set"),
            appid: std::env::var("WECHAT_APPID")
                .expect("WECHAT_APPID must be set"),
            base_url: std::env::var("WECHAT_BASE_URL")
                .unwrap_or_else(|_| "https://api.mch.weixin.qq.com".to_string()),
        })
    }
}
