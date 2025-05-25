//内部账号关联登陆验证实现
use crate::dao::{AccountResult, UserAuthError, UserAuthResult, UserAuthToken};
use crate::model::{AccountModel, AccountStatus};
use ip2location::Record;
use login::{AccountLoginEnv, AccountLoginParam};
use lsys_access::dao::{AccessAuthLoginData, AccessDao, AccessLoginData, SessionBody};
use lsys_core::now_time;
use lsys_core::{fluent_message, IntoFluentMessage, LimitParam};
use tokio::sync::Mutex;

use std::net::{IpAddr, Ipv4Addr};

use std::sync::Arc;

use tracing::{debug, warn};

use super::{AccountError, AccountLoginHistory};
use login::AccountLoginMeta;
pub mod login;

const ACCESS_LOGIN_DATA: &str = "login-data";

pub struct AuthAccountConfig {
    pub login_limit_captcha: u32,
    pub login_limit_lock: u32,
    pub login_limit_time: u64,
    pub ip_db: Option<Mutex<ip2location::DB>>,
}

impl AuthAccountConfig {
    pub fn new(ip_db: Option<ip2location::DB>) -> Self {
        Self {
            login_limit_captcha: 3,
            login_limit_lock: 8,
            login_limit_time: 300,
            ip_db: ip_db.map(|e| Mutex::new(e)),
        }
    }
}
pub struct AuthAccount {
    account_history: Arc<AccountLoginHistory>,
    access: Arc<AccessDao>,
    login_config: AuthAccountConfig,
}
impl AuthAccount {
    /// 对外对象创建
    pub fn new(
        account_history: Arc<AccountLoginHistory>,
        access: Arc<AccessDao>,
        login_config: AuthAccountConfig,
    ) -> Self {
        Self {
            account_history,
            access,
            login_config,
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
                Some(&LimitParam::new(None, true, 5, false, false)),
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
                if let Some(mut now_city) = self.env_to_city(login_env).await {
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
    async fn env_to_city(&self, login_env: &AccountLoginEnv) -> Option<String> {
        let login_ip = login_env.login_ip?;
        if let Some(ref lock_db) = self.login_config.ip_db {
            let mut db = lock_db.lock().await;
            if let Some(ref ip) = login_env.login_ip {
                let bip = *ip;
                if let Ok(rec) = db.ip_lookup(bip) {
                    match rec {
                        Record::LocationDb(record) => {
                            debug!("parse city: {:?} on ip: {:?}", record, login_ip);
                            let city = [
                                record
                                    .country
                                    .map(|e| e.short_name.to_string())
                                    .unwrap_or_default(),
                                record.region.unwrap_or_default().to_string(),
                                record.city.unwrap_or_default().to_string(),
                            ]
                            .into_iter()
                            .filter(|e| !e.is_empty() && *e != "-")
                            .collect::<Vec<String>>()
                            .join("-");
                            return Some(city);
                        }
                        Record::ProxyDb(_) => {}
                    }
                }
            }
        }
        None
    }
    //执行登录
    pub async fn login<TO: AccountLoginParam>(
        &self,
        login_param: &TO,
        login_env: AccountLoginEnv,
    ) -> AccountResult<UserAuthToken> {
        let login_ip = login_env
            .login_ip
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
            .to_string();
        let city = self.env_to_city(&login_env).await.unwrap_or_default();
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
                let is_login = i8::from(session.is_valid());
                self.account_history
                    .finish_history(login_id, is_login, account.id, "")
                    .await?;
                Ok(UserAuthToken::new(
                    session.session().user_app_id,
                    session.token_data(),
                    session.user_id(),
                    session.session().expire_time,
                ))
            }
            Err(err) => {
                let account_id = match err {
                    AccountError::PasswordNotMatch((uid, _)) => uid,
                    AccountError::PasswordNotSet((uid, _)) => uid,
                    AccountError::AuthStatusError((uid, _)) => uid,
                    _ => 0,
                };
                self.account_history
                    .finish_history(
                        login_id,
                        0,
                        account_id,
                        err.to_fluent_message().default_format(),
                    )
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
