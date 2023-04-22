use std::fmt::Display;
use std::ops::Add;
use std::sync::Arc;
use std::time::SystemTime;

use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{get_message, now_time, FluentMessage};

use lsys_user::dao::account::UserAccount;
use lsys_user::dao::auth::{SessionToken, SessionUserData};
use lsys_user::model::UserModel;

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use sqlx_model::{SqlQuote, Update, WhereOption};

use sqlx_model::{model_option_set, sql_format, Insert, Select};

use crate::dao::session::{RestAuthData, RestAuthTokenData};
use crate::model::AppsTokenStatus;
use crate::{
    dao::app::range_client_key,
    model::{AppsModel, AppsTokenModel, AppsTokenModelRef},
};

use super::{AppsError, AppsResult};

// OAUTH流程
// 验证登录用户成功->创建CODE(create_code)并返回->通过CODE创建TOKEN返回->通过TOKEN请求REST接口
// 生成CODE时保存:用户ID,需相关授权信息
// TOKEN 作用应该等于普通登录 UserTokenData

// oauth 登录服务器实现
pub struct AppsOauth {
    user_account: Arc<UserAccount>,
    db: Pool<MySql>,
    redis: deadpool_redis::Pool,
    fluent: Arc<FluentMessage>,
    pub cache: Arc<LocalCache<String, RestAuthData>>,
    time_out: u64,
    duration_time: usize,
}

const CODE_SAVE_KEY: &str = "oauth-code";

fn create_save_key(prefix: &str, app: impl Display, code: &str) -> String {
    format!("{}-{}-{}-{}", CODE_SAVE_KEY, prefix, app, code)
}

#[derive(Debug, Deserialize, Serialize)]
struct OauthData {
    scope: String,
    user_id: u64,
}

impl AppsOauth {
    pub fn new(
        user_account: Arc<UserAccount>,
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        fluent: Arc<FluentMessage>,
        time_out: u64,
    ) -> Self {
        Self {
            cache: Arc::from(LocalCache::new(
                redis.clone(),
                LocalCacheConfig::new("apps-oauth"),
            )),
            user_account,
            db,
            redis,
            fluent,
            time_out,
            duration_time: time_out as usize,
        }
    }
    /// 创建OAUTH CODE
    pub async fn create_code<'t>(
        &self,
        app: &AppsModel,
        scope: &String,
        user_id: u64,
    ) -> AppsResult<String> {
        let mut redis = self.redis.get().await?;
        let code = range_client_key();
        let save_key = create_save_key("code", app.id, &code);
        let val = serde_json::to_string(&OauthData {
            scope: scope.to_owned(),
            user_id,
        })?;
        redis.set(save_key.as_str(), val).await?;
        redis.expire(save_key.as_str(), self.duration_time).await?;
        Ok(code)
    }
    //从数据库中移除app token，返回移除数量
    pub async fn remove_token(&self, app: &AppsTokenModel) -> AppsResult<u64> {
        let status = AppsTokenStatus::Delete.to();
        let change = sqlx_model::model_option_set!(AppsTokenModelRef,{
            status:status,
        });
        let out = Update::<sqlx::MySql, AppsTokenModel, _>::new(change)
            .execute_by_pk(app, &self.db)
            .await?;
        Ok(out.rows_affected())
    }
    //从数据库中添加 app token
    async fn add_token(
        &self,
        app: &AppsModel,
        scope: String,
        user_id: u64,
        code: String,
    ) -> AppsResult<AppsTokenModel> {
        let token_time = now_time()?;
        let timeout = token_time + 3600 * 24 * 7;
        let token = range_client_key();
        let status = AppsTokenStatus::Enable.to();
        let idata = model_option_set!(AppsTokenModelRef,{
            code:code,
            token:token,
            access_user_id:user_id,
            scope:scope,
            timeout:timeout,
            app_id:app.id,
            token_time:token_time,
            status:status
        });
        let res = Insert::<sqlx::MySql, AppsTokenModel, _>::new(idata)
            .execute(&self.db)
            .await?;
        Ok(AppsTokenModel {
            id: res.last_insert_id(),
            app_id: app.id,
            access_user_id: user_id,
            code,
            token,
            scope,
            token_time,
            timeout,
            status: AppsTokenStatus::Enable.to(),
        })
    }
    //根据用户id获取用户数据
    async fn find_user_id(&self, user_id: &u64) -> AppsResult<UserModel> {
        let user = self
            .user_account
            .user
            .find_by_id(user_id)
            .await
            .map_err(|err| {
                if err.is_not_found() {
                    AppsError::System(get_message!(
                        &self.fluent,
                        "user-not-find",
                        "login user data is not find,may be is delete"
                    ))
                } else {
                    err.into()
                }
            })?;
        Ok(user)
    }
    /// 创建OAUTH TOKEN
    pub async fn create_token(
        &self,
        app: &AppsModel,
        code: String,
    ) -> AppsResult<(AppsTokenModel, UserModel)> {
        let save_key = create_save_key("code", app.id, &code);
        let mut redis = self.redis.get().await?;
        let data_opt: Option<String> = redis.get(save_key.as_str()).await?;
        let data = data_opt.unwrap_or_default();
        if data.is_empty() {
            return Err(AppsError::System(get_message!(
                &self.fluent,
                "token-not-find",
                "your submit token is not find"
            )));
        }
        let oauth_data = serde_json::from_slice::<OauthData>(data.as_bytes());
        match oauth_data {
            Ok(oauth) => {
                let user = self.find_user_id(&oauth.user_id).await?;
                let status = AppsTokenStatus::Enable.to();
                let data = Select::type_new::<AppsTokenModel>()
                    .fetch_one_by_where::<AppsTokenModel, _>(
                        &WhereOption::Where(sql_format!(
                            "app_id={} and code={} and status={}",
                            app.id,
                            code,
                            status,
                        )),
                        &self.db,
                    )
                    .await;
                match data {
                    Ok(code) => Ok((code, user)),
                    Err(sqlx::Error::RowNotFound) => {
                        let otoken = self
                            .add_token(app, oauth.scope, oauth.user_id, code)
                            .await?;
                        Ok((otoken, user))
                    }
                    Err(err) => Err(err.into()),
                }
            }
            Err(err) => {
                redis.del(save_key).await?;
                Err(err.into())
            }
        }
    }
    //根据app,app token,user 数据创建session
    pub async fn create_session(
        &self,
        app: &AppsModel,
        app_token: &AppsTokenModel,
        user: &UserModel,
    ) -> AppsResult<()> {
        let save_key = create_save_key("token", &app.client_id, &app_token.token);
        let data = self.cache.get(&save_key).await;
        match data {
            Some(user_auth) => {
                if user_auth.token.id != app_token.id {
                    self.cache.clear(&save_key).await;
                }
            }
            None => {
                if app_token.timeout < now_time().unwrap_or_default() {
                    return Err(AppsError::System(get_message!(
                        &self.fluent,
                        "token-is-timeout",
                        "your submit code is timeout or wrong"
                    )));
                }
                let time_out = SystemTime::now()
                    .add(std::time::Duration::from_secs(self.time_out))
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs();
                let nikename = user.nickname.to_owned();
                let save_data = RestAuthData::new(
                    SessionUserData {
                        user_id: app_token.access_user_id,
                        user_nickname: nikename,
                        user_password_id: user.password_id,
                        time_out,
                    },
                    app_token.to_owned(),
                );
                self.cache.set(save_key, save_data, self.time_out).await;
            }
        }
        Ok(())
    }
    //根据rest token获取session数据
    pub async fn get_session_data(
        &self,
        user_token_data: &RestAuthTokenData,
    ) -> AppsResult<RestAuthData> {
        let save_key = create_save_key("token", &user_token_data.client_id, &user_token_data.token);
        let data = self.cache.get(&save_key).await;
        match data {
            Some(ad) => Ok(ad),
            None => Err(AppsError::System(get_message!(
                &self.fluent,
                "token-is-timeout",
                "your submit code is timeout or wrong"
            ))),
        }
    }
    //删除session
    pub async fn clear_session(
        &self,
        user_token: &SessionToken<RestAuthTokenData>,
    ) -> AppsResult<()> {
        if let Some(user_token_data) = user_token.data() {
            let rest_data = self.get_session_data(user_token_data).await?;
            self.remove_token(&rest_data.token).await?;
            let save_key =
                create_save_key("token", &user_token_data.client_id, &user_token_data.token);
            self.cache.clear(&save_key).await;
        }
        Ok(())
    }
    //刷新登陆session数据
    pub async fn refresh_session(
        &self,
        app: &AppsModel,
        user_token_data: &RestAuthTokenData,
        reset_token: bool,
    ) -> AppsResult<RestAuthTokenData> {
        let auth_data = self.get_session_data(user_token_data).await?;
        let token = if reset_token {
            let oldtoken = auth_data.token.clone();
            let token = self
                .add_token(app, oldtoken.scope, oldtoken.access_user_id, oldtoken.code)
                .await?;
            self.remove_token(&auth_data.token).await?;
            token
        } else {
            auth_data.token
        };
        let user = self.find_user_id(&token.access_user_id).await?;
        self.create_session(app, &token, &user).await?;
        Ok(RestAuthTokenData {
            client_id: app.client_id.clone(),
            token: token.token,
        })
    }
}
