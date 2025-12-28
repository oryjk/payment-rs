use thiserror::Error;

/// 领域层错误类型
#[derive(Error, Debug)]
pub enum DomainError {
    /// 验证错误
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// 订单未找到
    #[error("Payment order not found: {0}")]
    OrderNotFound(String),

    /// 订单状态错误
    #[error("Invalid payment state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },

    /// 金额无效
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    /// 签名验证失败
    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    /// 微信支付API错误
    #[error("WeChat Pay API error: {0}")]
    WeChatPayError(String),

    /// 数据库错误
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    /// 序列化错误
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// HTTP请求错误
    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// 加密错误
    #[error("Cryptography error: {0}")]
    CryptoError(String),

    /// 配置错误
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// 内部错误
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// 领域结果类型
pub type DomainResult<T> = Result<T, DomainError>;
