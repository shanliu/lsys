use std::sync::Arc;

use lsys_core::{
    fluent_message, now_time, rand_str, valid_key, ValidIp, ValidParam, ValidParamCheck,
    ValidPattern, ValidStrlen,
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::dao::AccessError;

use super::{AccessAuth, AccessAuthLoginData, AccessLoginData, AccessResult, SessionBody};

// OAUTH流程
// 验证登录用户成功
//->用用户数据创建CODE(create_code)并返回
//->通过CODE创建TOKEN返回
//->通过TOKEN请求REST接口
// 生成CODE时保存:用户ID,需相关授权信息

// oauth 登录服务器实现
pub struct AccessOAuth {
    auth: Arc<AccessAuth>,
    redis: deadpool_redis::Pool,
}

impl AccessOAuth {
    pub fn new(auth: Arc<AccessAuth>, redis: deadpool_redis::Pool) -> Self {
        Self { auth, redis }
    }
}
fn range_client_key() -> String {
    rand_str(lsys_core::RandType::LowerHex, 32)
}
pub const OAUTH_LOGIN_TYPE: &str = "oauth";

const CODE_SAVE_KEY: &str = "oauth-code";
fn create_save_key(prefix: &str, app_id: u64, oauth_app_id: u64, code: &str) -> String {
    format!(
        "{}-{}-{}-{}-{}",
        CODE_SAVE_KEY, prefix, app_id, oauth_app_id, code
    )
}

#[derive(Serialize, Deserialize)]
pub struct AccessOAuthCodeData<'t> {
    pub user_data: &'t str,
    pub user_name: &'t str,
    pub user_account: Option<&'t str>,
    pub login_ip: Option<&'t str>,
    pub device_id: Option<&'t str>,
    pub device_name: Option<&'t str>,
    pub session_data: Vec<(&'t str, &'t str)>,
}
impl AccessOAuth {
    /// 创建OAUTH CODE
    async fn create_code_param_valid(
        &self,
        code_data: &AccessOAuthCodeData<'_>,
    ) -> AccessResult<()> {
        let mut valid_param = ValidParam::default();
        valid_param
            .add(
                valid_key!("user-data"),
                &code_data.user_data,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(2, 32))
                    .add_rule(ValidPattern::Ident),
            )
            .add(
                valid_key!("user-name"),
                &code_data.user_name,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::max(32))
                    .add_rule(ValidPattern::NotFormat),
            );
        if let Some(ref user_account) = code_data.user_account {
            valid_param.add(
                valid_key!("user-account"),
                user_account,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::max(128)),
            );
        }
        if let Some(ref device_id) = code_data.device_id {
            valid_param.add(
                valid_key!("device-id"),
                device_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::max(32)),
            );
        }
        if let Some(ref device_name) = code_data.device_name {
            valid_param.add(
                valid_key!("device-name"),
                device_name,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::max(255)),
            );
        }
        if let Some(ref login_ip) = code_data.login_ip {
            valid_param.add(
                valid_key!("login-ip"),
                login_ip,
                &ValidParamCheck::default().add_rule(ValidIp::default()),
            );
        }
        for (key, _) in &code_data.session_data {
            valid_param.add(
                valid_key!("session-key"),
                key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, 12)),
            );
        }
        valid_param.check()?;
        Ok(())
    }
    /// 创建OAUTH CODE
    pub async fn create_code(
        &self,
        app_id: u64,
        oauth_app_id: u64,
        code_data: &AccessOAuthCodeData<'_>,
        time_out: usize,
    ) -> AccessResult<String> {
        self.create_code_param_valid(code_data).await?;
        let mut redis = self.redis.get().await?;
        let code = range_client_key();
        let save_key = create_save_key("code", app_id, oauth_app_id, &code);
        let val = serde_json::to_string(code_data)?;
        let _: () = redis.set(save_key.as_str(), val).await?;
        let _: () = redis.expire(save_key.as_str(), time_out as i64).await?;
        Ok(code)
    }
    async fn destroy_code_param_valid(&self, code: &str) -> AccessResult<()> {
        ValidParam::default()
            .add(
                valid_key!("oauth-code"),
                &code,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(16, 64))
                    .add_rule(ValidPattern::Hex),
            )
            .check()?;
        Ok(())
    }
    /// 清理创建的 code
    pub async fn destroy_code(
        &self,
        app_id: u64,
        oauth_app_id: u64,
        code: &str,
    ) -> AccessResult<()> {
        self.destroy_code_param_valid(code).await?;
        let mut redis = self.redis.get().await?;
        let save_key = create_save_key("code", app_id, oauth_app_id, code);
        let find: bool = redis.exists(save_key.as_str()).await?;
        if find {
            match redis.del(save_key).await {
                Ok(()) => {}
                Err(err) => {
                    warn!("remove oauth code fail:{}", err);
                }
            }
        }
        Ok(())
    }
    /// 根据code完成登录
    async fn code_do_login_param_valid(&self, code: &str) -> AccessResult<()> {
        ValidParam::default()
            .add(
                valid_key!("oauth-code"),
                &code,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(16, 64))
                    .add_rule(ValidPattern::Hex),
            )
            .check()?;
        Ok(())
    }
    /// 根据code完成登录
    pub async fn code_do_login(
        &self,
        app_id: u64,
        oauth_app_id: u64,
        code: &str,
        token_data: Option<&str>,
        time_out: u64,
        session_data: &[(&str, &str)],
    ) -> AccessResult<SessionBody> {
        self.code_do_login_param_valid(code).await?;
        let mut redis = self.redis.get().await?;
        let save_key = create_save_key("code", app_id, oauth_app_id, code);
        let data_opt: Option<String> = redis.get(save_key.as_str()).await?;
        match data_opt {
            None => Err(AccessError::System(fluent_message!("access-not-code"))),
            Some(data) => {
                let code_data = serde_json::from_str::<AccessOAuthCodeData>(data.as_str())
                    .map_err(|e| AccessError::System(fluent_message!("access-bad-code", e)))?;
                let expire_time = if time_out > 0 {
                    now_time().unwrap_or_default() + time_out
                } else {
                    0
                };
                let mut session_data = session_data.to_owned();
                session_data.extend(code_data.session_data);
                let login_data = AccessLoginData {
                    expire_time,
                    user_account: code_data.user_account,
                    login_ip: code_data.login_ip,
                    device_id: code_data.device_id,
                    device_name: code_data.device_name,
                    session_data,
                };
                self.auth
                    .do_login(&AccessAuthLoginData {
                        app_id,
                        oauth_app_id,
                        user_data: &code_data.user_data,
                        user_name: code_data.user_name,
                        token_data,
                        login_type: OAUTH_LOGIN_TYPE,
                        login_data: Some(&login_data),
                    })
                    .await
            }
        }
    }
}
