use lsys_setting::dao::{SettingDecode, SettingEncode, SettingJson, SettingKey, SettingResult};
use serde::{Deserialize, Serialize};

use tokio::sync::RwLock;
pub type AccountPasswordHashCallback = Box<dyn Fn(&str) -> String + Send + Sync>;

/// 登录密码HASH实现
pub struct AccountPasswordHash {
    hash: RwLock<AccountPasswordHashCallback>,
}
impl Default for AccountPasswordHash {
    fn default() -> Self {
        Self {
            hash: RwLock::new(Self::md5_hash(None)),
        }
    }
}
impl AccountPasswordHash {
    /// 使用MD5加salt方式加密
    pub async fn set_md5(&self, salt: Option<&str>) {
        self.set_call(Self::md5_hash(salt)).await;
    }
    /// 自定义加密
    pub async fn set_call(&self, hash: AccountPasswordHashCallback) {
        *(self.hash.write().await) = hash;
    }
    fn md5_hash(salt: Option<&str>) -> AccountPasswordHashCallback {
        let salt_str = salt.map(|e| e.to_string());
        Box::new(move |password: &str| {
            let mut _passed = password.to_owned();
            if let Some(ref salt_) = salt_str {
                _passed += salt_.as_str();
            }
            let digest = md5::compute(_passed.as_bytes());
            let hash_password = format!("{:x}", digest);
            hash_password
        })
    }
    pub async fn hash_password(&self, password: &str) -> String {
        self.hash.read().await(password)
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
