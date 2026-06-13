//! # lsys-kms
//!
//! 为 lsys-core 的 KMS 解密器提供阿里云和腾讯云的实现。
//!
//! ## 功能特性
//!
//! - `aliyun-kms` — 支持阿里云 KMS 解密
//! - `tencent-kms` — 支持腾讯云 KMS 解密
//!
//! ## 使用示例
//!
//! ```ignore
//! use lsys_kms::aliyun::AliyunKmsDecryptor;
//! use lsys_core::secret::SecretManager;
//! use std::sync::Arc;
//!
//! let kms = AliyunKmsDecryptor::new(
//!     "your-access-key-id",
//!     "your-access-key-secret",
//!     "cn-beijing", // region
//! );
//!
//! let manager = SecretManager::builder(&config)
//!     .kms_provider("aliyun", Arc::new(kms))
//!     .build()
//!     .await?;
//! ```

#[cfg(feature = "aliyun-kms")]
pub mod aliyun;

#[cfg(feature = "tencent-kms")]
pub mod tencent;
