use crate::dao::account::UserAccount;
use crate::model::{UserModel, UserStatus};
use async_trait::async_trait;
use base64::Engine;
use ip2location::Record;
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{get_message, FluentMessage};
use lsys_core::{now_time, PageParam};
use redis::aio::ConnectionManager;
use serde::Deserialize;
use serde::Serialize;
use sqlx::{MySql, Pool};
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, debug_span, warn, Instrument};

use base64::{
    alphabet,
    engine::{self, general_purpose},
};

const CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

use super::{
    EmailCodeLoginData, EmailLoginData, ExternalLoginData, MobileCodeLoginData, MobileLoginData,
    NameLoginData, SessionData, SessionToken, SessionTokenData, SessionUserData, UserAuthError,
    UserAuthResult, UserSession,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LoginData {
    Name(NameLoginData),
    Email(EmailLoginData),
    EmailCode(EmailCodeLoginData),
    Mobile(MobileLoginData),
    MobileCode(MobileCodeLoginData),
    External(ExternalLoginData),
}

impl LoginData {
    /// 重新加载登录数据
    async fn reload(&self, db: &Pool<MySql>) -> UserAuthResult<Self> {
        match self {
            LoginData::Name(data) => Ok(LoginData::Name(data.reload(db).await?)),
            LoginData::Email(data) => Ok(LoginData::Email(data.reload(db).await?)),
            LoginData::EmailCode(data) => Ok(LoginData::EmailCode(data.reload(db).await?)),
            LoginData::Mobile(data) => Ok(LoginData::Mobile(data.reload(db).await?)),
            LoginData::MobileCode(data) => Ok(LoginData::MobileCode(data.reload(db).await?)),
            LoginData::External(data) => Ok(LoginData::External(data.reload(db).await?)),
        }
    }
}

//登录类型
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginType {
    pub time_out: u32,
    pub type_name: String,
}

//登录参数接口
#[async_trait]
pub trait LoginParam {
    //获取登录用户数据
    async fn get_user(
        &self,
        db: &Pool<MySql>,
        redis: &Arc<Mutex<ConnectionManager>>,
        fluent: &Arc<FluentMessage>,
        account: &Arc<UserAccount>,
        login_env: &LoginEnv,
    ) -> UserAuthResult<(LoginData, UserModel)>;
    //获取登录类型
    async fn get_type(
        &self,
        db: &Pool<MySql>,
        redis: &Arc<Mutex<ConnectionManager>>,
        fluent: &Arc<FluentMessage>,
    ) -> UserAuthResult<LoginType>;
    fn show_name(&self) -> String;
}

pub struct LoginEnv {
    pub login_ip: Option<IpAddr>,
}

type UserPasswordHashCallback = Box<dyn Fn(&String) -> String + Send + Sync>;

/// 登录密码HASH实现
pub struct UserPasswordHash {
    hash: RwLock<UserPasswordHashCallback>,
}
impl Default for UserPasswordHash {
    fn default() -> Self {
        Self {
            hash: RwLock::new(Self::md5_hash(None)),
        }
    }
}
impl UserPasswordHash {
    /// 使用MD5加salt方式加密
    pub async fn set_md5(&self, salt: Option<String>) {
        self.set_call(Self::md5_hash(salt)).await;
    }
    /// 自定义加密
    pub async fn set_call(&self, hash: UserPasswordHashCallback) {
        *(self.hash.write().await) = hash;
    }
    fn md5_hash(salt: Option<String>) -> UserPasswordHashCallback {
        Box::new(move |password: &String| {
            let mut _passed = password.to_owned();
            if let Some(ref salt_) = salt {
                _passed += salt_.as_str();
            }
            let digest = md5::compute(_passed.as_bytes());
            let hash_password = format!("{:x}", digest);
            hash_password
        })
    }
    pub async fn hash_password(&self, password: &String) -> String {
        self.hash.read().await(password)
    }
}

pub struct UserAuthConfig {
    pub cache_config: LocalCacheConfig,
    pub login_limit_captcha: u32,
    pub login_limit_lock: u32,
    pub login_limit_time: u64,
    pub ip_db: Option<Mutex<ip2location::DB>>,
}

impl Default for UserAuthConfig {
    fn default() -> Self {
        Self {
            login_limit_captcha: 3,
            login_limit_lock: 8,
            login_limit_time: 300,
            ip_db: None,
            cache_config: LocalCacheConfig::new("user-auth"),
        }
    }
}

//登录后数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthData {
    session_data: SessionUserData,
    pub login_type: LoginType,
    pub login_data: LoginData,
}
impl UserAuthData {
    pub fn new(
        session_data: SessionUserData,
        login_type: LoginType,
        login_data: LoginData,
    ) -> Self {
        Self {
            session_data,
            login_type,
            login_data,
        }
    }
}
impl SessionData for UserAuthData {
    fn user_data(&self) -> &SessionUserData {
        &self.session_data
    }
}

//登录产生标识
#[derive(Clone, Debug)]
pub struct UserAuthTokenData {
    pub token: String,
    pub user_id: u64,
    pub time_out: u64,
}
impl ToString for UserAuthTokenData {
    fn to_string(&self) -> String {
        let str = format!("{}-{}-{}", self.user_id, self.token, self.time_out);
        let base64 = CUSTOM_ENGINE.encode(str.as_bytes());
        base64
    }
}
impl FromStr for UserAuthTokenData {
    type Err = UserAuthError;
    /// 从TOKEN字符串还原
    fn from_str(token_str: &str) -> Result<Self, Self::Err> {
        let token_str = token_str.to_owned();
        let de64 = &CUSTOM_ENGINE
            .decode(token_str.as_bytes())
            .map_err(|e| UserAuthError::TokenParse(e.to_string()))?;
        let token_str = String::from_utf8(de64.to_owned())
            .map_err(|e| UserAuthError::TokenParse(e.to_string()))?;
        let mut token_split = token_str.split('-');
        let user_id = token_split.next().ok_or_else(|| {
            UserAuthError::TokenParse("token is not split fail:user_id".to_string())
        })?;
        let user_id = user_id
            .parse::<u64>()
            .map_err(|e| UserAuthError::TokenParse(e.to_string()))?;
        let token = token_split
            .next()
            .ok_or_else(|| UserAuthError::TokenParse("token is not split fail:token".to_string()))?
            .to_string();
        let time_out = token_split
            .next()
            .ok_or_else(|| {
                UserAuthError::TokenParse("token is not split fail:timeout".to_string())
            })?
            .parse::<u64>()
            .map_err(|e| UserAuthError::TokenParse(e.to_string()))?;
        Ok(Self::new(token, user_id, time_out))
    }
}
impl SessionTokenData for UserAuthTokenData {}
impl UserAuthTokenData {
    pub fn new(token: String, user_id: u64, time_out: u64) -> Self {
        Self {
            token,
            user_id,
            time_out,
        }
    }
    pub fn is_timeout(&self) -> bool {
        self.time_out <= now_time().unwrap_or_default()
    }
}

impl FromStr for SessionToken<UserAuthTokenData> {
    type Err = UserAuthError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_data(Some(UserAuthTokenData::from_str(s)?)))
    }
}
impl ToString for SessionToken<UserAuthTokenData> {
    fn to_string(&self) -> String {
        self.data()
            .as_ref()
            .map(|e| e.to_string())
            .unwrap_or_default()
    }
}

impl SessionToken<UserAuthTokenData> {
    pub fn is_ok(&self) -> bool {
        if let Some(data) = self.data() {
            return !data.is_timeout();
        }
        false
    }
}

//登录数据存储接口
#[async_trait]
pub trait UserAuthStore {
    //登录后设置登录数据
    async fn set_data(
        &mut self,
        user_token_data: Option<UserAuthTokenData>,
        login_type: LoginType,
        login_data: LoginData,
        account: UserModel,
    ) -> UserAuthResult<UserAuthTokenData>;
    //退出时清理登录数据
    async fn clear_data(&mut self, token: &UserAuthTokenData) -> UserAuthResult<()>;
    //从存储的数据中获取登录数据
    async fn get_data(&self, token: &UserAuthTokenData) -> UserAuthResult<UserAuthData>;
    //是否存在此登录数据
    async fn exist_data(&self, token: &UserAuthTokenData) -> bool;
}

//验证登录相关接口
//不包含登录状态
pub struct UserAuth<T: UserAuthStore> {
    db: Pool<MySql>,
    redis: Arc<Mutex<ConnectionManager>>,
    fluent: Arc<FluentMessage>,
    account: Arc<UserAccount>,
    login_store: RwLock<T>,
    pub cache: LocalCache<String, UserAuthData>,
    login_config: UserAuthConfig,
}
impl<T: UserAuthStore + Send + Sync> UserAuth<T> {
    /// 对外对象创建
    /// 对外对象创建
    pub fn new(
        db: Pool<MySql>,
        redis: Arc<Mutex<ConnectionManager>>,
        fluent: Arc<FluentMessage>,
        account: Arc<UserAccount>,
        store: T,
        config: Option<UserAuthConfig>,
    ) -> Self {
        let login_config = config.unwrap_or_default();
        UserAuth {
            cache: LocalCache::new(redis.clone(), login_config.cache_config),
            login_store: RwLock::new(store),
            account,
            login_config,
            redis,
            db,
            fluent,
        }
    }
    /// 检测用户是否可以登录及是否需要登录验证码
    pub async fn check<TO: LoginParam>(
        &self,
        login_param: &TO,
        _login_env: &LoginEnv,
    ) -> UserAuthResult<()> {
        let user_res = self
            .account
            .user_login
            .history_data(
                None,
                Some(login_param.show_name().to_owned()),
                None,
                None,
                &Some(PageParam::page(1, 5)),
            )
            .await;
        match user_res {
            Ok(ues) => {
                let mut last_time = 0;
                let mut is_fail = 0;
                let mut last_city_check = false;
                let mut last_city: Option<String> = None;
                let mut is_captcha = false;
                for u in ues.into_iter() {
                    if let Some(city) = last_city.clone() {
                        if last_city_check {
                            last_city_check = true;
                            if !city.is_empty() && city != u.login_city {
                                is_captcha = true;
                            }
                        }
                        if u.is_login == 0 {
                            if last_time == 0 {
                                last_time = u.add_time;
                            }
                            is_fail += 1;
                        } else {
                            break;
                        }
                    } else {
                        last_city = Some(u.login_city.replace(['-', ' '], ""));
                    }
                }
                if self.login_config.login_limit_lock > 0
                    && is_fail >= self.login_config.login_limit_lock
                {
                    let now_time = now_time().unwrap_or_default();
                    if self.login_config.login_limit_time > 0
                        && last_time + self.login_config.login_limit_time > now_time
                    {
                        return Err(UserAuthError::CheckUserLock(
                            last_time + self.login_config.login_limit_time - now_time,
                        ));
                    }
                }
                if is_captcha
                    || (self.login_config.login_limit_captcha > 0
                        && is_fail >= self.login_config.login_limit_captcha)
                {
                    return Err(UserAuthError::CheckCaptchaNeed(
                        get_message!(&self.fluent,"auth-user-captcha","{$user} login need captcha code",["user"=>login_param.show_name()]),
                    ));
                }
            }
            Err(err) => {
                warn!(
                    "check captcha fail: {} in account:{}",
                    err.to_string(),
                    login_param.show_name()
                );
            }
        };
        Ok(())
    }
    //执行登录
    pub async fn login<TO: LoginParam>(
        &self,
        login_param: TO,
        login_env: LoginEnv,
    ) -> UserAuthResult<UserAuthTokenData> {
        let login_type = login_param
            .get_type(&self.db, &self.redis, &self.fluent)
            .await?;
        let login_ip = login_env
            .login_ip
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
            .to_string();
        let mut city = String::from("");
        if let Some(ref lock_db) = self.login_config.ip_db {
            let mut db = lock_db.lock().await;
            if let Some(ref ip) = login_env.login_ip {
                let bip = *ip;
                if let Ok(rec) = db.ip_lookup(bip) {
                    match rec {
                        Record::LocationDb(record) => {
                            debug!("parse city: {:?} on ip: {:?}", record, login_ip);
                            city = [
                                record
                                    .country
                                    .map(|e| e.short_name)
                                    .unwrap_or_else(String::new),
                                record.region.unwrap_or_default(),
                                record.city.unwrap_or_default(),
                            ]
                            .into_iter()
                            .filter(|e| !e.is_empty() && *e != "-")
                            .collect::<Vec<String>>()
                            .join("-");
                        }
                        Record::ProxyDb(_) => {}
                    }
                }
            }
        }
        let login_account = login_param.show_name();
        let login_id = self
            .account
            .user_login
            .create_history(login_account, login_type.type_name.clone(), login_ip, city)
            .await?;
        let res = self.login_user(login_param, login_env).await;
        match res {
            Ok((login_type_data, account)) => {
                let store = self.login_store.write();
                let user_id = account.id;
                let user_token_res = store
                    .await
                    .set_data(None, login_type, login_type_data, account)
                    .instrument(debug_span!("auth_login"))
                    .await;
                let is_login = i8::from(user_token_res.is_ok());
                let login_msg = match &user_token_res {
                    Ok(_) => "".to_string(),
                    Err(err) => err.to_string(),
                };
                let login_token = match &user_token_res {
                    Ok(user) => user.to_string(),
                    Err(err) => err.to_string(),
                };
                self.account
                    .user_login
                    .finish_history(login_id, is_login, user_id, login_msg, login_token)
                    .await?;
                user_token_res
            }
            Err(err) => {
                let user_id = match err {
                    UserAuthError::PasswordNotMatch((uid, _)) => uid,
                    UserAuthError::PasswordNotSet((uid, _)) => uid,
                    UserAuthError::StatusError((uid, _)) => uid,
                    _ => 0,
                };
                self.account
                    .user_login
                    .finish_history(login_id, 0, user_id, err.to_string(), "".to_string())
                    .await?;
                Err(err)
            }
        }
    }
    async fn login_user<TO: LoginParam>(
        &self,
        login_param: TO,
        login_env: LoginEnv,
    ) -> UserAuthResult<(LoginData, UserModel)> {
        let show_name = login_param.show_name().to_owned();
        let (login_type_data, account) = login_param
            .get_user(
                &self.db,
                &self.redis,
                &self.fluent,
                &self.account,
                &login_env,
            )
            .await?;
        if UserStatus::Delete.eq(account.status) {
            return Err(UserAuthError::StatusError((
                account.id,
                get_message!(&self.fluent,"auth-user-disable","{$user} is disable",["user"=>show_name]),
            )));
        }
        Ok((login_type_data, account))
    }
    fn token_result<'t>(
        &self,
        user_token: &'t SessionToken<UserAuthTokenData>,
    ) -> UserAuthResult<&'t UserAuthTokenData> {
        user_token
            .data()
            .and_then(|data| if data.is_timeout() { None } else { Some(data) })
            .ok_or_else(|| {
                UserAuthError::NotLogin(get_message!(
                    &self.fluent,
                    "auth-not-login",
                    "user not login"
                ))
            })
    }
    //得到当前登陆用户
    pub async fn get_session_data(
        &self,
        user_token: &SessionToken<UserAuthTokenData>,
    ) -> UserAuthResult<UserAuthData> {
        let user_token_data = self.token_result(user_token)?;
        let store = self.login_store.read();
        let now_time = now_time()?;
        if let Some(data) = self.cache.get(&user_token_data.token).await {
            return Ok(data);
        }
        let ua = store.await.get_data(user_token_data).await?;
        if ua.user_data().time_out > now_time {
            self.cache
                .set(
                    user_token_data.token.clone(),
                    ua.clone(),
                    ua.user_data().time_out - now_time,
                )
                .await;
        } else {
            self.cache.clear(&user_token_data.token).await;
        }
        Ok(ua)
    }
    //重新加载当前用户
    //user_token 当前登陆的 UserAuthTokenData
    //reset_token 是否重新生成 UserAuthTokenData
    //返回UserAuthTokenData 但reset_token为true时为新生成的 UserAuthTokenData
    pub async fn reload_auth(
        &self,
        user_token: &SessionToken<UserAuthTokenData>,
        reset_token: bool,
    ) -> UserAuthResult<UserAuthTokenData> {
        let user_token_data = self.token_result(user_token)?;
        let read_store = self.login_store.read();
        let user = read_store.await.get_data(user_token_data).await?;
        let account = self
            .account
            .user
            .find_by_id(&user.user_data().user_id)
            .await?;
        let login_data = user.login_data.reload(&self.db).await?;
        let store = self.login_store.write();
        let usertoken = store
            .await
            .set_data(
                if !reset_token {
                    Some(user_token_data.to_owned())
                } else {
                    None
                },
                user.login_type,
                login_data,
                account,
            )
            .instrument(debug_span!("reload_user"))
            .await?;
        self.cache.clear(&user_token_data.token).await;
        Ok(usertoken)
    }
    //退出登录
    pub async fn logout(&self, user_token: &SessionToken<UserAuthTokenData>) -> UserAuthResult<()> {
        match user_token.data() {
            Some(user_token_data) => {
                self.cache.clear(&user_token_data.token).await;
                self.login_store
                    .write()
                    .await
                    .clear_data(user_token_data)
                    .await
            }
            None => Ok(()),
        }
    }
    //获取当前登录
    pub async fn is_login(&self, user_token: &SessionToken<UserAuthTokenData>) -> bool {
        if let Some(user_token_data) = user_token.data() {
            if let Ok(now_time) = now_time() {
                if self.cache.get(&user_token_data.token).await.is_some() {
                    return true;
                }
                let store = self.login_store.read().await;
                if store.exist_data(user_token_data).await {
                    if self.cache.config().cache_time > 0 {
                        if let Ok(ua) = store.get_data(user_token_data).await {
                            if ua.user_data().time_out > now_time {
                                let set_time = ua.user_data().time_out - now_time;
                                self.cache
                                    .set(user_token_data.token.clone(), ua, set_time)
                                    .await;
                                return true;
                            } else {
                                self.cache.clear(&user_token_data.token).await;
                            }
                        }
                    } else {
                        return true;
                    }
                }
            }
        }
        false
    }
}

pub struct UserAuthSession<T: UserAuthStore + Send + Sync> {
    pub auth: Arc<UserAuth<T>>,
    pub user_token: SessionToken<UserAuthTokenData>,
}
#[async_trait]
impl<T: UserAuthStore + Send + Sync> UserSession<UserAuthTokenData, UserAuthData>
    for UserAuthSession<T>
{
    fn get_session_token(&self) -> &SessionToken<UserAuthTokenData> {
        &self.user_token
    }
    fn set_session_token(&mut self, token: SessionToken<UserAuthTokenData>) {
        self.user_token = token
    }
    async fn get_session_data(&self) -> UserAuthResult<UserAuthData> {
        self.auth.get_session_data(&self.user_token).await
    }
    async fn refresh_session(&mut self, reset_token: bool) -> UserAuthResult<UserAuthTokenData> {
        let token = self.auth.reload_auth(&self.user_token, reset_token).await?;
        self.user_token = token.clone().into();
        Ok(token)
    }
    async fn clear_session(&mut self) -> UserAuthResult<()> {
        self.auth.logout(&self.user_token).await?;
        self.user_token.clear();
        Ok(())
    }
}

impl<T: UserAuthStore + Send + Sync> UserAuthSession<T> {
    pub fn new(
        auth: Arc<UserAuth<T>>,
        user_token: SessionToken<UserAuthTokenData>,
    ) -> UserAuthSession<T> {
        Self { auth, user_token }
    }
    pub async fn is_login(&self) -> bool {
        self.auth.is_login(&self.user_token).await
    }
    pub fn set_session_token_str(&mut self, token_str: &str) {
        self.set_session_token(
            SessionToken::<UserAuthTokenData>::from_str(token_str).unwrap_or_default(),
        )
    }
    pub async fn get_user(&self) -> UserAuthResult<UserModel> {
        let auth_data = self.get_session_data().await?;
        Ok(self
            .auth
            .account
            .user
            .find_by_id(&auth_data.user_data().user_id)
            .await?)
    }
}
