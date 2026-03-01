use crate::dao::utils::env_to_city;
//内部账号关联登陆验证实现
use crate::dao::{AccountResult, UserAuthError, UserAuthResult};
use crate::model::{AccountLoginStatus, AccountModel, AccountStatus};
use login::{AccountLoginEnv, AccountLoginParam};
use lsys_access::dao::{AccessAuthLoginData, AccessDao, AccessLoginData, SessionBody};
use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::utils::now_time;
use lsys_core::fluent_message;
use lsys_mfa::dao::MfaSubject;
use tokio::sync::Mutex;

use std::net::{IpAddr, Ipv4Addr};

use std::sync::Arc;

use tracing::warn;

use super::{AccountError, AccountLoginHistory};
use login::AccountLoginMeta;
pub mod login;

const ACCESS_LOGIN_DATA: &str = "login-data";

pub struct AuthAccountConfig {
    pub login_limit_captcha: u32,
    pub login_limit_lock: u32,
    pub login_limit_time: u64,
    ip_db: Option<Arc<Mutex<ip2location::DB>>>,
}

impl AuthAccountConfig {
    pub fn new(ip_db: Option<Arc<Mutex<ip2location::DB>>>) -> Self {
        Self {
            login_limit_captcha: 3,
            login_limit_lock: 8,
            login_limit_time: 300,
            ip_db,
        }
    }
}
pub struct AuthAccount {
    account_history: Arc<AccountLoginHistory>,
    access: Arc<AccessDao>,
    login_config: AuthAccountConfig,
    mfa_login: Arc<crate::dao::MfaLoginDao>,
}
impl AuthAccount {
    /// 对外对象创建
    pub fn new(
        account_history: Arc<AccountLoginHistory>,
        access: Arc<AccessDao>,
        login_config: AuthAccountConfig,
        mfa_login: Arc<crate::dao::MfaLoginDao>,
    ) -> Self {
        Self {
            account_history,
            access,
            login_config,
            mfa_login,
        }
    }
    /// 检测用户是否可以登录及是否需要登录验证码
    pub async fn check<TO: AccountLoginParam>(
        &self,
        login_param: &TO,
        login_env: &AccountLoginEnv,
    ) -> UserAuthResult<()> {
        let user_res = self
            .account_history
            .history_data(
                None,
                Some(&login_param.account_name()),
                None,
                None,
                None,
                &CursorPageParam::new(
                    CursorPageDir::Next,
                    CursorConfig::primary(CursorPageSort::Desc),
                    None,
                    CursorLimit::Limit {
                        limit: 5,
                        more: false,
                    },
                ),
            )
            .await;
        match user_res {
            Ok((ues, _)) => {
                let mut last_time = 0;
                let mut is_fail = 0;
                for u in ues.iter() {
                    if u.is_login == 0 {
                        is_fail += 1;
                        if last_time == 0 {
                            last_time = u.add_time;
                        }
                    } else {
                        break;
                    }
                }
                if self.login_config.login_limit_lock > 0
                    && is_fail >= self.login_config.login_limit_lock
                {
                    let now_time = now_time().unwrap_or_default();
                    if self.login_config.login_limit_time > 0
                        && last_time + self.login_config.login_limit_time > now_time
                    {
                        let ctime = last_time + self.login_config.login_limit_time - now_time;
                        return Err(UserAuthError::CheckUserLock((
                            ctime,
                            fluent_message!("check-user-lock",{"user":login_param.account_name(),"time":ctime}),
                        )));
                    }
                }
                let mut is_captcha = false;
                if let Some(ref ip_db) = self.login_config.ip_db {
                    if let Some(mut now_city) = env_to_city(ip_db, login_env).await {
                        for u in ues.iter() {
                            let tmp_c = u.login_city.replace(['-', ' '], "");
                            if tmp_c.is_empty() {
                                continue;
                            }
                            now_city = now_city.replace(['-', ' '], "");
                            if now_city != tmp_c {
                                is_captcha = true;
                            }
                        }
                    }
                }
                if is_captcha
                    || (self.login_config.login_limit_captcha > 0
                        && is_fail >= self.login_config.login_limit_captcha)
                {
                    return Err(UserAuthError::CheckCaptchaNeed(
                        fluent_message!("auth-user-captcha",{"user":login_param.account_name()}), //"{$user} login need captcha code"
                    ));
                }
            }
            Err(err) => {
                warn!(
                    "check captcha fail: {} in account:{}",
                    err.to_fluent_message().default_format(),
                    login_param.account_name()
                );
            }
        };
        Ok(())
    }
    //IP 转成城市
    //执行登录
    pub async fn login<TO: AccountLoginParam>(
        &self,
        login_param: &TO,
        login_env: AccountLoginEnv,
    ) -> AccountResult<SessionBody> {
        let login_ip = login_env
            .login_ip
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
            .to_string();
        let city = if let Some(ref ip_db) = self.login_config.ip_db {
            env_to_city(ip_db, &login_env).await.unwrap_or_default()
        } else {
            "".to_string()
        };
        let login_account = login_param.account_name();
        let login_id = self
            .account_history
            .create_history(
                &login_account,
                &<TO as AccountLoginParam>::Meta::login_type(),
                &login_ip,
                &city,
            )
            .await?;
        let res = self.login_user(login_param, login_env).await;
        match res {
            Ok((account, session)) => {
                self.account_history
                    .finish_history(login_id, AccountLoginStatus::LoginSuccess, account.id, "")
                    .await?;
                Ok(session)
            }
            Err(err) => {
                let (status, account_id) = match &err {
                    AccountError::MfaNeed { account_id, .. } => {
                        (AccountLoginStatus::PreLoginSuccess, *account_id)
                    }
                    AccountError::PasswordNotMatch((uid, _)) => (AccountLoginStatus::Failed, *uid),
                    AccountError::PasswordNotSet((uid, _)) => (AccountLoginStatus::Failed, *uid),
                    AccountError::AuthStatusError((uid, _)) => (AccountLoginStatus::Failed, *uid),
                    _ => (AccountLoginStatus::Failed, 0),
                };
                let login_msg = if matches!(status, AccountLoginStatus::PreLoginSuccess) {
                    "".to_string()
                } else {
                    err.to_fluent_message().default_format()
                };
                self.account_history
                    .finish_history(login_id, status, account_id, login_msg)
                    .await?;
                Err(err)
            }
        }
    }
    async fn login_user<TO: AccountLoginParam>(
        &self,
        login_param: &TO,
        login_env: AccountLoginEnv,
    ) -> AccountResult<(AccountModel, SessionBody)> {
        let (login_type_data, account) = login_param.get_account(&login_env).await?;
        if AccountStatus::Delete.eq(account.status) {
            return Err(AccountError::AuthStatusError((
                account.id,
                fluent_message!("auth-user-disable",{"user":login_param.account_name()}), //"{$user} is disable",
            )));
        }
        let login_ip = login_env
            .login_ip
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
            .to_string();
        let login_account = login_param.account_name();
        let time = now_time()?;
        let session_data = if login_type_data.is_empty() {
            vec![]
        } else {
            vec![(ACCESS_LOGIN_DATA, login_type_data.as_str())]
        };
        let login_data = AccessLoginData {
            user_account: Some(&login_account),
            login_ip: Some(&login_ip),
            device_id: None,
            device_name: None,
            expire_time: time + <TO as AccountLoginParam>::Meta::login_timeout(),
            session_data,
        };

        // If MFA is enabled for this subject, require verification before creating session.
        let subject = MfaSubject::new(0, account.id.to_string());
        if self.mfa_login.is_totp_enabled(&subject).await? {
            let mfa_token = self
                .mfa_login
                .create_prelogin_totp(
                    crate::dao::PreloginTotpParams {
                        subject,
                        app_id: 0,
                        oauth_app_id: 0,
                        user_nickname: account.nickname.clone(),
                        token_data: None,
                        login_type: <TO as AccountLoginParam>::Meta::login_type().to_owned(),
                    },
                    &login_data,
                )
                .await?;
            return Err(AccountError::MfaNeed {
                account_id: account.id,
                mfa_token,
            });
        }
        let session = self
            .access
            .auth
            .do_login(&AccessAuthLoginData {
                app_id: 0,
                oauth_app_id: 0,
                user_data: account.id,
                user_nickname: &account.nickname,
                token_data: None,
                login_type: &<TO as AccountLoginParam>::Meta::login_type(),
                login_data: Some(&login_data),
            })
            .await?;
        Ok((account, session))
    }
}
