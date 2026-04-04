use crate::dao::login::AccountLoginEnv;
use crate::dao::utils::env_to_city;
use crate::dao::{AccountError, AccountLoginHistory, AccountResult};

use deadpool_redis::Pool;
use lsys_access::dao::{AccessAuthLoginData, AccessDao, AccessLoginData, SessionBody};
use lsys_core::app_core::AppCore;
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::utils::{now_time, rand_str, RandType};
use lsys_core::fluent_message;
use lsys_mfa::dao::MfaError;
use lsys_mfa::dao::{MfaSubject, MfaTotpDao};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tokio::sync::Mutex;

const KEY_PREFIX: &str = "mfa-login:";

/// Parameters for creating a TOTP prelogin token.
pub struct PreloginTotpParams {
    pub subject: MfaSubject,
    pub app_id: u64,
    pub oauth_app_id: u64,
    pub user_nickname: String,
    pub token_data: Option<String>,
    pub login_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PendingAccessLoginData {
    user_account: Option<String>,
    login_ip: Option<String>,
    device_id: Option<String>,
    device_name: Option<String>,
    expire_time: u64,
    session_data: Vec<(String, String)>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PendingLoginData {
    subject: MfaSubject,

    app_id: u64,
    oauth_app_id: u64,

    user_nickname: String,
    token_data: Option<String>,

    login_type: String,
    login_data: PendingAccessLoginData,
}

pub struct MfaLoginDao {
    redis: Pool,
    access: Arc<AccessDao>,
    totp: Arc<MfaTotpDao>,
    ttl_seconds: u64,
    account_history: Arc<AccountLoginHistory>,
    ip_db: Option<Arc<Mutex<ip2location::DB>>>,
}

impl MfaLoginDao {
    pub fn new(
        redis: Pool,
        access: Arc<AccessDao>,
        totp: Arc<MfaTotpDao>,
        app_core: Arc<AppCore>,
        account_history: Arc<AccountLoginHistory>,
        ip_db: Option<Arc<Mutex<ip2location::DB>>>,
    ) -> Self {
        let ttl_seconds = app_core
            .config
            .find(None)
            .get_int("mfa_login_ttl")
            .ok()
            .map(|e| e as u64)
            .filter(|&e| e > 0)
            .unwrap_or(300)
            .clamp(30, 3600);

        Self {
            redis,
            access,
            totp,
            ttl_seconds,
            account_history,
            ip_db,
        }
    }

    pub async fn is_totp_enabled(&self, subject: &MfaSubject) -> AccountResult<bool> {
        self.totp
            .is_enabled(subject)
            .await
            .map_err(|e| AccountError::System(e.to_fluent_message()))
    }

    pub async fn create_prelogin_totp(
        &self,
        params: PreloginTotpParams,
        login_data: &AccessLoginData<'_>,
    ) -> AccountResult<String> {
        let pending = PendingLoginData {
            subject: params.subject,
            app_id: params.app_id,
            oauth_app_id: params.oauth_app_id,
            user_nickname: params.user_nickname,
            token_data: params.token_data,
            login_type: params.login_type,
            login_data: PendingAccessLoginData {
                user_account: login_data.user_account.map(|s| s.to_owned()),
                login_ip: login_data.login_ip.map(|s| s.to_owned()),
                device_id: login_data.device_id.map(|s| s.to_owned()),
                device_name: login_data.device_name.map(|s| s.to_owned()),
                expire_time: login_data.expire_time,
                session_data: login_data
                    .session_data
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
        };

        let token = rand_str(RandType::LowerHex, 48);
        let key = format!("{}{}", KEY_PREFIX, token);
        let value = serde_json::to_string(&pending)?;

        let mut conn = self.redis.get().await?;
        conn.set_ex::<_, _, ()>(key, value, self.ttl_seconds)
            .await?;

        Ok(token)
    }

    pub async fn verify_totp_and_login(
        &self,
        mfa_token: &str,
        totp_code: &str,
        login_env: &AccountLoginEnv,
    ) -> AccountResult<SessionBody> {
        if mfa_token.trim().is_empty() {
            return Err(AccountError::Param(fluent_message!("mfa-token-empty")));
        }

        let key = format!("{}{}", KEY_PREFIX, mfa_token);
        let mut conn = self.redis.get().await?;
        let raw: Option<String> = conn.get(&key).await?;
        let raw = raw.ok_or_else(|| AccountError::MfaError(MfaError::TokenExpired))?;

        // Best-effort one-time consumption.
        let _: i64 = conn.del(&key).await.unwrap_or(0);

        let pending: PendingLoginData = serde_json::from_str(&raw)?;

        if let (Some(expect_ip), Some(got_ip)) =
            (pending.login_data.login_ip.as_deref(), login_env.login_ip)
            && expect_ip != got_ip.to_string().as_str() {
                return Err(AccountError::Param(fluent_message!(
                    "mfa-token-ip-mismatch"
                )));
            }

        self.totp.verify_totp(&pending.subject, totp_code).await?;

        let session_data_pairs: Vec<(&str, &str)> = pending
            .login_data
            .session_data
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let login_data = AccessLoginData {
            user_account: pending.login_data.user_account.as_deref(),
            login_ip: pending.login_data.login_ip.as_deref(),
            device_id: pending.login_data.device_id.as_deref(),
            device_name: pending.login_data.device_name.as_deref(),
            expire_time: pending.login_data.expire_time,
            session_data: session_data_pairs,
        };

        // Guard against already-expired pending logins.
        let now = now_time().unwrap_or_default();
        if login_data.expire_time <= now {
            return Err(AccountError::MfaError(MfaError::TokenExpired));
        }
        let res = self
            .access
            .auth
            .do_login(&AccessAuthLoginData {
                app_id: pending.app_id,
                oauth_app_id: pending.oauth_app_id,
                user_data: pending.subject.user_data.as_str(),
                user_nickname: &pending.user_nickname,
                token_data: pending.token_data.as_deref(),
                login_type: &pending.login_type,
                login_data: Some(&login_data),
            })
            .await?;
        let login_ip = login_env
            .login_ip
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
            .to_string();
        let city = if let Some(ref ip_db) = self.ip_db {
            env_to_city(ip_db, login_env).await.unwrap_or_default()
        } else {
            "".to_string()
        };

        self.account_history
            .create_history(
                &pending.login_data.user_account.unwrap_or_default(),
                &pending.login_type,
                &login_ip,
                &city,
            )
            .await?;
        Ok(res)
    }
}
