use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine;

use crate::config::Config;

use super::error::SecretError;
use super::kms::KmsDecryptor;

/// 启动后只读的密钥存储。
///
/// 通过 [`SecretManager::builder`] 创建，`build()` 时从配置文件加载全部密钥并缓存在内存中。
/// 之后各模块持有 `Arc<SecretManager>`，通过 [`require`][Self::require] 按 key_id 获取密钥字节。
pub struct SecretManager {
    pub keys: HashMap<String, Vec<u8>>,
}

impl SecretManager {
    /// 创建 Builder，传入 lsys-core [`Config`] 引用。
    pub fn builder(config: &Config) -> SecretManagerBuilder<'_> {
        SecretManagerBuilder {
            config,
            kms_providers: HashMap::new(),
        }
    }
}

impl Default for SecretManager {
    fn default() -> Self {
        SecretManager {
            keys: HashMap::new(),
        }
    }
}

impl SecretManager {
    /// 获取密钥字节，key_id 不存在时返回 `None`。
    pub fn get(&self, key_id: &str) -> Option<&[u8]> {
        self.keys.get(key_id).map(|v| v.as_slice())
    }

    /// 获取密钥字节，key_id 不存在时返回 `Err(SecretError::KeyNotFound)`。
    pub fn require(&self, key_id: &str) -> Result<&[u8], SecretError> {
        self.keys
            .get(key_id)
            .map(|v| v.as_slice())
            .ok_or_else(|| SecretError::KeyNotFound(key_id.to_string()))
    }

    /// 获取密钥并解析为 UTF-8 字符串引用。
    /// key_id 不存在或内容不是合法 UTF-8 时返回错误。
    pub fn require_str(&self, key_id: &str) -> Result<&str, SecretError> {
        let bytes = self.require(key_id)?;
        std::str::from_utf8(bytes)
            .map_err(|e| SecretError::Decode(format!("secret.{}: not valid UTF-8: {}", key_id, e)))
    }
}

/// [`SecretManager`] 构建器。
///
/// # 配置格式（TOML）
///
/// ```toml
/// # 适配器①：明文密钥（value 支持 hex: / base64: / raw: 前缀，无前缀视为 raw）
/// [secret.file_aes_key]
/// source = "plain"
/// value  = "hex:6368616e67652d746869732d746f2d796f757273656372657421"
///
/// # 适配器②：KMS 解密（ciphertext 编码方式同 value）
/// [secret.login_signing_key]
/// source     = "kms"
/// kms        = "my-aws"
/// ciphertext = "base64:AQICAHg..."
/// ```
///
/// # 示例
///
/// ```rust,ignore
/// let manager = SecretManager::builder(&app_core.config)
///     .kms_provider("my-aws", Arc::new(MyAwsDecryptor::new(...)))
///     .build()
///     .await?;
/// let manager = Arc::new(manager);
/// ```
pub struct SecretManagerBuilder<'a> {
    config: &'a Config,
    kms_providers: HashMap<String, Arc<dyn KmsDecryptor>>,
}

impl<'a> SecretManagerBuilder<'a> {
    /// 注册一个命名的 KMS 解密器。
    ///
    /// `name` 对应配置中 `kms = "<name>"` 字段。
    pub fn kms_provider(
        mut self,
        name: impl Into<String>,
        provider: Arc<dyn KmsDecryptor>,
    ) -> Self {
        self.kms_providers.insert(name.into(), provider);
        self
    }

    /// 从配置加载全部密钥。
    ///
    /// - `source = "plain"` — 同步解码 `value` 字段
    /// - `source = "kms"`   — 异步调用对应 KMS 解密器解密 `ciphertext` 字段
    pub async fn build(self) -> Result<SecretManager, SecretError> {
        let raw_config = self.config.find(None);

        let secret_table = raw_config.get_table("secret").unwrap_or_default();

        let mut keys = HashMap::new();

        for (key_id, value) in secret_table {
            let entry = value.into_table().map_err(|e| {
                SecretError::Config(format!("secret.{}: must be a table: {}", key_id, e))
            })?;

            let source = entry
                .get("source")
                .and_then(|v| v.clone().into_string().ok())
                .ok_or_else(|| {
                    SecretError::Config(format!("secret.{}: missing 'source' field", key_id))
                })?;

            let key_bytes = match source.as_str() {
                "plain" => {
                    let value_str = entry
                        .get("value")
                        .and_then(|v| v.clone().into_string().ok())
                        .ok_or_else(|| {
                            SecretError::Config(format!("secret.{}: missing 'value' field", key_id))
                        })?;
                    decode_value_str(&value_str, &key_id)?
                }
                "kms" => {
                    let kms_name = entry
                        .get("kms")
                        .and_then(|v| v.clone().into_string().ok())
                        .ok_or_else(|| {
                            SecretError::Config(format!("secret.{}: missing 'kms' field", key_id))
                        })?;
                    let ciphertext_str = entry
                        .get("ciphertext")
                        .and_then(|v| v.clone().into_string().ok())
                        .ok_or_else(|| {
                            SecretError::Config(format!(
                                "secret.{}: missing 'ciphertext' field",
                                key_id
                            ))
                        })?;
                    let ciphertext = decode_value_str(&ciphertext_str, &key_id)?;
                    let decryptor = self
                        .kms_providers
                        .get(&kms_name)
                        .ok_or_else(|| SecretError::KmsNotFound(kms_name.clone()))?;
                    decryptor.decrypt(&ciphertext).await?
                }
                other => {
                    return Err(SecretError::Config(format!(
                        "secret.{}: unknown source '{}', expected 'plain' or 'kms'",
                        key_id, other
                    )));
                }
            };

            keys.insert(key_id, key_bytes);
        }

        Ok(SecretManager { keys })
    }
}

/// 按前缀解码 value 字符串：
/// - `hex:<hex>` — hex 解码
/// - `base64:<b64>` — base64 标准解码
/// - `raw:<str>` — 取 `<str>` 的 UTF-8 字节
/// - 无前缀 — 同 `raw:`
fn decode_value_str(value: &str, key_id: &str) -> Result<Vec<u8>, SecretError> {
    if let Some(hex_str) = value.strip_prefix("hex:") {
        hex::decode(hex_str.trim())
            .map_err(|e| SecretError::Decode(format!("secret.{}: hex decode: {}", key_id, e)))
    } else if let Some(b64_str) = value.strip_prefix("base64:") {
        base64::engine::general_purpose::STANDARD
            .decode(b64_str.trim())
            .map_err(|e| SecretError::Decode(format!("secret.{}: base64 decode: {}", key_id, e)))
    } else if let Some(raw_str) = value.strip_prefix("raw:") {
        Ok(raw_str.as_bytes().to_vec())
    } else {
        Ok(value.as_bytes().to_vec())
    }
}
