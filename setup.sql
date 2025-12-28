-- 创建数据库
CREATE DATABASE IF NOT EXISTS payment_db CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- 使用数据库
USE payment_db;

-- 创建支付订单表
CREATE TABLE IF NOT EXISTS payment_orders (
    id CHAR(36) PRIMARY KEY COMMENT '订单ID (UUID)',
    out_order_no VARCHAR(64) NOT NULL UNIQUE COMMENT '商户订单号',
    transaction_id VARCHAR(64) NULL COMMENT '微信支付交易号',
    amount_cents BIGINT NOT NULL COMMENT '支付金额（分）',
    payment_method VARCHAR(50) NOT NULL COMMENT '支付方式: mini_program, jsapi, native, h5',
    state VARCHAR(50) NOT NULL COMMENT '支付状态: pending, processing, succeeded, failed, refunded, closed',
    description VARCHAR(127) NOT NULL COMMENT '商品描述',
    openid VARCHAR(128) NULL COMMENT '用户OpenID',
    client_ip VARCHAR(45) NOT NULL COMMENT '客户端IP',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    paid_at TIMESTAMP NULL COMMENT '支付完成时间',
    attach TEXT NULL COMMENT '附加数据',
    prepay_id VARCHAR(64) NULL COMMENT '微信预下单ID',

    INDEX idx_out_order_no (out_order_no),
    INDEX idx_transaction_id (transaction_id),
    INDEX idx_state (state),
    INDEX idx_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='支付订单表';

-- 显示创建的表
SHOW TABLES;
