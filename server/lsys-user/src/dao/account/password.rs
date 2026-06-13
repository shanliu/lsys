use lsys_setting::dao::{SettingDecode, SettingEncode, SettingJson, SettingKey, SettingResult};
use serde::{Deserialize, Serialize};

use tokio::sync::RwLock;
pub type AccountPasswordHashCallback = Box<dyn Fn(&str) -> String + Send + Sync>;
pub type AccountPasswordVerifyCallback = Box<dyn Fn(&str, &str) -> bool + Send + Sync>;

/// 登录密码HASH实现
pub struct AccountPasswordHash {
    hash: RwLock<AccountPasswordHashCallback>,
    verify: RwLock<AccountPasswordVerifyCallback>,
}

impl AccountPasswordHash {
    /// MD5 哈希（向后兼容）
    pub fn new_md5_hash(salt_str: Option<&str>) -> Self {
        let salt_str = salt_str.map(|e| e.to_string());
        let salt_str_verify = salt_str.clone();
        Self {
            hash: RwLock::new(Box::new(move |password: &str| {
                let mut _passed = password.to_owned();
                if let Some(ref salt_) = salt_str {
                    _passed += salt_.as_str();
                }
                let digest = md5::compute(_passed.as_bytes());
                let hash_password = format!("{:x}", digest);
                hash_password
            })),
            verify: RwLock::new(Box::new(move |password: &str, stored_hash: &str| {
                let mut _passed = password.to_owned();
                if let Some(ref salt_) = salt_str_verify {
                    _passed += salt_.as_str();
                }
                let digest = md5::compute(_passed.as_bytes());
                format!("{:x}", digest) == stored_hash
            })),
        }
    }

    /// Argon2id 哈希（推荐用于新部署，符合 OWASP 现代安全规范）
    ///
    /// `pepper` 建议从 `SecretManager::require("password_pepper")` 获取；
    /// 传 `None` 时不使用 pepper。
    pub fn new_argon2_hash(pepper: Option<&[u8]>) -> Self {
        use argon2::{
            password_hash::{
                rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
            },
            Algorithm, Argon2, Params, Version,
        };

        let pepper_hash = pepper.map(|p| p.to_vec());
        let pepper_verify = pepper.map(|p| p.to_vec());

        Self {
            hash: RwLock::new(Box::new(move |password: &str| {
                let salt = SaltString::generate(&mut OsRng);
                if let Some(ref p) = pepper_hash {
                    Argon2::new_with_secret(
                        p.as_slice(),
                        Algorithm::Argon2id,
                        Version::V0x13,
                        Params::default(),
                    )
                    .ok()
                    .and_then(|a| a.hash_password(password.as_bytes(), &salt).ok())
                    .map(|h| h.to_string())
                    .unwrap_or_default()
                } else {
                    Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default())
                        .hash_password(password.as_bytes(), &salt)
                        .ok()
                        .map(|h| h.to_string())
                        .unwrap_or_default()
                }
            })),
            verify: RwLock::new(Box::new(move |password: &str, stored_hash: &str| {
                let parsed_hash = match PasswordHash::new(stored_hash) {
                    Ok(h) => h,
                    Err(_) => return false,
                };
                if let Some(ref p) = pepper_verify {
                    Argon2::new_with_secret(
                        p.as_slice(),
                        Algorithm::Argon2id,
                        Version::V0x13,
                        Params::default(),
                    )
                    .ok()
                    .map(|a| a.verify_password(password.as_bytes(), &parsed_hash).is_ok())
                    .unwrap_or(false)
                } else {
                    Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default())
                        .verify_password(password.as_bytes(), &parsed_hash)
                        .is_ok()
                }
            })),
        }
    }
}

impl AccountPasswordHash {
    /// 自定义加密
    pub async fn set_call(&self, hash: AccountPasswordHashCallback) {
        *(self.hash.write().await) = hash;
    }
    /// 自定义验证
    pub async fn set_verify_call(&self, verify: AccountPasswordVerifyCallback) {
        *(self.verify.write().await) = verify;
    }
    pub async fn hash_password(&self, password: &str) -> String {
        self.hash.read().await(password)
    }
    /// 验证密码是否与存储的哈希匹配。
    pub async fn verify_password(&self, password: &str, stored_hash: &str) -> bool {
        self.verify.read().await(password, stored_hash)
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct AccountPasswordConfig {
    pub timeout: u64,
    pub disable_old_password: bool,
}

impl SettingKey for AccountPasswordConfig {
    fn key<'t>() -> &'t str {
        "account-password"
    }
}
impl SettingDecode for AccountPasswordConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        SettingJson::decode(data)
    }
}
impl SettingEncode for AccountPasswordConfig {
    fn encode(&self) -> String {
        SettingJson::encode(self)
    }
}
impl SettingJson<'_> for AccountPasswordConfig {}
