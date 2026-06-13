use crate::fluent_message;
use crate::fluents::{FluentMessage, IntoFluentMessage};

#[derive(Debug)]
pub enum SecretError {
    /// 密钥 ID 未注册（config 中不存在 / `require()` 时缺失）
    KeyNotFound(String),
    /// 配置格式或字段错误
    Config(String),
    /// `source = "kms"` 对应的 KMS 解密器未注册
    KmsNotFound(String),
    /// KMS 解密调用失败
    KmsDecrypt(String),
    /// value 字段解码失败（hex / base64）
    Decode(String),
    /// 字段加密/解密失败
    Encrypt(String),
}

impl IntoFluentMessage for SecretError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            SecretError::KeyNotFound(key) => fluent_message!("secret-key-not-found", key),
            SecretError::Config(msg) => fluent_message!("secret-config-error", msg),
            SecretError::KmsNotFound(name) => fluent_message!("secret-kms-not-found", name),
            SecretError::KmsDecrypt(msg) => fluent_message!("secret-kms-error", msg),
            SecretError::Decode(msg) => fluent_message!("secret-decode-error", msg),
            SecretError::Encrypt(msg) => fluent_message!("secret-encrypt-error", msg),
        }
    }
}
