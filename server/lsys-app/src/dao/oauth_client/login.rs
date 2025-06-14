use lsys_access::dao::AccessError;
use lsys_core::db::{Insert, ModelTableName, Update, WhereOption};
use lsys_core::{fluent_message, model_option_set, now_time, rand_str, sql_format};
use serde::{Deserialize, Serialize};

use crate::dao::oauth_client::access::AccessOAuthCodeData;
use crate::dao::session::RestAuthData;

use super::AppOAuthClient;
use crate::dao::{AppError, AppResult};
use crate::model::{
    AppModel, AppOAuthClientAccessModel, AppOAuthClientAccessModelRef,
    AppOAuthClientRefreshTokenModel, AppOAuthClientRefreshTokenModelRef,
    AppOAuthClientRefreshTokenStatus, AppRequestType,
};
use lsys_core::db::SqlQuote;

// OAUTH流程
// 验证登录用户成功->创建CODE(create_code)并返回->通过CODE创建TOKEN返回->通过TOKEN请求REST接口
// 生成CODE时保存:用户ID,需相关授权信息
// TOKEN 作用应该等于普通登录 UserTokenData

const APP_OAUTH_CODE: &str = "oauth-code";
const APP_OAUTH_SCOPE: &str = "oauth-sopce";
// oauth 登录服务器实现

#[derive(Serialize, Deserialize)]
pub struct AppOAuthCodeData<'t> {
    pub user_data: &'t str,
    pub user_nickname: &'t str,
    pub user_account: Option<&'t str>,
    pub login_ip: Option<&'t str>,
    pub device_id: Option<&'t str>,
    pub device_name: Option<&'t str>,
    pub scope_data: Vec<&'t str>,
    pub session_data: Vec<(&'t str, &'t str)>, //用户登陆相关数据
}

impl AppOAuthClient {
    async fn check_access(&self, app: &AppModel) -> AppResult<()> {
        self.oauth_check(app).await?;
        if app.parent_app_id > 0 {
            let papp = self.app.cache().find_by_id(app.parent_app_id).await?;
            papp.app_status_check()?;
            self.app
                .cache()
                .feature_check(&papp, &[AppRequestType::OAuthServer.feature_key()])
                .await?;
        }
        Ok(())
    }
    /// 创建OAUTH CODE
    pub async fn create_code(
        &self,
        app: &AppModel,
        code_data: &AppOAuthCodeData<'_>,
    ) -> AppResult<String> {
        self.check_access(app).await?;
        let oauth_client = self.find_by_app(app).await?;
        let app_scope_data = oauth_client.scope_data.split(",").collect::<Vec<&str>>();
        let mut bad_sopce = vec![];
        for tmp in code_data.scope_data.iter() {
            if !app_scope_data.contains(tmp) {
                bad_sopce.push(tmp.to_owned());
            }
        }
        if !bad_sopce.is_empty() {
            return Err(AppError::System(
                fluent_message!("app-oauth-login-bad-scope",{
                    "scope_data":bad_sopce.join(",")
                }),
            ));
        }
        let mut session_data = code_data.session_data.clone();
        let scope_data = code_data.scope_data.join(",");
        session_data.push((APP_OAUTH_SCOPE, scope_data.as_str()));
        self.oauth_access
            .save_code(
                app.parent_app_id, //>0 为外部应用
                app.id,
                &AccessOAuthCodeData {
                    user_data: code_data.user_data,
                    user_nickname: code_data.user_nickname,
                    user_account: code_data.user_account,
                    login_ip: code_data.login_ip,
                    device_id: code_data.device_id,
                    device_name: code_data.device_name,
                    session_data,
                },
                self.code_time as usize,
            )
            .await
    }
    //根据app,app token,user 数据创建session
    pub async fn create_session(
        &self,
        app: &AppModel,
        code: &str,
    ) -> AppResult<(RestAuthData, String)> {
        self.check_access(app).await?;
        let code_data_str = self
            .oauth_access
            .get_code(app.parent_app_id, app.id, code)
            .await?;
        let code_data = serde_json::from_str::<AccessOAuthCodeData>(code_data_str.as_str())
            .map_err(|e| AppError::System(fluent_message!("access-bad-code", e)))?;

        let session_data = self
            .oauth_access
            .do_login(
                app.parent_app_id,
                app.id,
                None,
                self.login_time,
                code_data,
                &[(APP_OAUTH_CODE, code)],
            )
            .await?;
        let mut db = self.db.begin().await?;
        let refresh_token_data = rand_str(lsys_core::RandType::Lower, 32);
        let status = AppOAuthClientRefreshTokenStatus::Init as i8;
        let add_time = now_time().unwrap_or_default();
        let time_out = add_time + self.refresh_time;
        let source_code = code.to_owned();
        let add_data = model_option_set!(AppOAuthClientRefreshTokenModelRef,{
            app_id:app.id,
            time_out:time_out,
            refresh_token_data:refresh_token_data,
            source_code:source_code,
            code_data:code_data_str,
            status:status,
            add_time:add_time,
        });
        if let Err(e) = Insert::<AppOAuthClientRefreshTokenModel, _>::new(add_data)
            .execute(&mut *db)
            .await
        {
            db.rollback().await?;
            return Err(e)?;
        };
        let add_data = model_option_set!(AppOAuthClientAccessModelRef,{
            app_id:app.id,
            access_token_data:session_data.session().token_data,
            refresh_token_data:refresh_token_data,
            add_time:add_time,
        });
        if let Err(e) = Insert::<AppOAuthClientAccessModel, _>::new(add_data)
            .execute(&mut *db)
            .await
        {
            db.rollback().await?;
            return Err(e)?;
        };
        db.commit().await?;

        Ok((
            RestAuthData::new(app.to_owned(), session_data),
            refresh_token_data,
        ))
    }
    pub async fn clear_refresh_token(&self, app: &AppModel, refresh_token: &str) -> AppResult<()> {
        let status = AppOAuthClientRefreshTokenStatus::Delete as i8;
        let delete_time = now_time().unwrap_or_default();
        let change = lsys_core::model_option_set!(AppOAuthClientRefreshTokenModelRef,{
            status:status,
            delete_time:delete_time,
        });
        Update::<AppOAuthClientRefreshTokenModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!(
                    "app_id={} and refresh_token_data={} and status={}",
                    app.id,
                    refresh_token,
                    AppOAuthClientRefreshTokenStatus::Init as i8
                )),
                &self.db,
            )
            .await?;
        let mut start_id = 0;
        loop {
            let tmp_vec=sqlx::query_as::<_,(u64,String)>(&sql_format!(
                "select id,access_token_data from {} where app_id={} and refresh_token_data={} and id>{} order by id asc limit 100",
                AppOAuthClientAccessModel::table_name(),
                app.id,
                refresh_token,
                start_id,
            ))
            .fetch_all(&self.db)
            .await?;
            if tmp_vec.is_empty() {
                break;
            }
            for (row_id, access_token_data) in tmp_vec {
                start_id = row_id;
                let login = self
                    .access
                    .auth
                    .login_data(app.parent_app_id, app.id, &access_token_data)
                    .await;
                match login {
                    Ok(session) => {
                        if let Ok(Some(code)) = self
                            .access
                            .auth
                            .session_get_data(&session, APP_OAUTH_CODE)
                            .await
                        {
                            let _ = self
                                .oauth_access
                                .destroy_code(app.parent_app_id, app.id, &code)
                                .await;
                        }
                        self.access.auth.do_logout(&session).await?
                    }
                    Err(e) => match e {
                        AccessError::NotLogin => {}
                        _ => {
                            return Err(e.into());
                        }
                    },
                }
            }
        }

        Ok(())
    }
    //删除session
    pub async fn clear_access_token(&self, app: &AppModel, access_token: &str) -> AppResult<()> {
        let login = self
            .access
            .auth
            .login_data(app.parent_app_id, app.id, access_token)
            .await;
        match login {
            Ok(session) => {
                if let Ok(Some(code)) = self
                    .access
                    .auth
                    .session_get_data(&session, APP_OAUTH_CODE)
                    .await
                {
                    let _ = self
                        .oauth_access
                        .destroy_code(app.parent_app_id, app.id, &code)
                        .await;
                }
                self.access.auth.do_logout(&session).await?
            }
            Err(e) => match e {
                AccessError::NotLogin => {
                    return Ok(());
                }
                _ => {
                    return Err(e.into());
                }
            },
        }
        Ok(())
    }
    //刷新登陆session数据
    pub async fn refresh_session(
        &self,
        app: &AppModel,
        refresh_token: &str,
    ) -> AppResult<RestAuthData> {
        self.check_access(app).await?;
        let (code_data,source_code)=sqlx::query_as::<_,(String,String)>(&sql_format!(
            "select code_data,source_code from {} where app_id={} and refresh_token_data={} and status={} and time_out>{}",
            AppOAuthClientRefreshTokenModel::table_name(),
            app.id,
            refresh_token,
            AppOAuthClientRefreshTokenStatus::Init as i8,
            now_time().unwrap_or_default()
        ))
        .fetch_one(&self.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::Access(AccessError::NotLogin),
            _ => AppError::Sqlx(e),
        })?;
        let code_data = serde_json::from_str::<AccessOAuthCodeData>(code_data.as_str())
            .map_err(|e| AppError::System(fluent_message!("access-bad-code", e)))?;

        let session_data = self
            .oauth_access
            .do_login(
                app.parent_app_id,
                app.id,
                None,
                self.login_time,
                code_data,
                &[(APP_OAUTH_CODE, &source_code)],
            )
            .await?;
        let refresh_token_data = refresh_token.to_owned();
        let add_time = now_time().unwrap_or_default();
        let add_data = model_option_set!(AppOAuthClientAccessModelRef,{
            app_id:app.id,
            access_token_data:session_data.session().token_data,
            refresh_token_data:refresh_token_data,
            add_time:add_time,
        });
        Insert::<AppOAuthClientAccessModel, _>::new(add_data)
            .execute(&self.db)
            .await?;
        //@todo 缩短旧access_token的有效期。。。
        Ok(RestAuthData::new(app.to_owned(), session_data))
    }
    //根据rest token获取session数据
    pub async fn get_session_data(
        &self,
        app: &AppModel,
        user_token: &str,
    ) -> AppResult<RestAuthData> {
        let session_body = self
            .access
            .auth
            .cache()
            .login_data(app.parent_app_id, app.id, user_token)
            .await?;

        Ok(RestAuthData::new(app.to_owned(), session_body))
    }
    //获取登录的SCOPE数据
    pub async fn get_session_scope_data(&self, auth_data: &RestAuthData) -> AppResult<Vec<String>> {
        let scope_data = self
            .access
            .auth
            .cache()
            .session_get_data(auth_data, APP_OAUTH_SCOPE)
            .await?
            .map(|e| e.split(",").map(|t| t.to_string()).collect::<Vec<_>>())
            .unwrap_or_default();
        Ok(scope_data)
    }
    //检测请求的SCOPE数据是否符合登录授权
    pub async fn check_session_scope_data(
        &self,
        auth_data: &RestAuthData,
        check_scope: &[&str],
    ) -> AppResult<()> {
        let scope_data = self.get_session_scope_data(auth_data).await?;
        let mut bad = vec![];
        for sp in check_scope {
            let tmp_sp = sp.to_string();
            if !scope_data.contains(&tmp_sp) {
                bad.push(tmp_sp);
            }
        }
        if !bad.is_empty() {
            return Err(AppError::ScopeBad(bad));
        }
        Ok(())
    }
}
