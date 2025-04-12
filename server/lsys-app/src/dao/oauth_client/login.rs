use lsys_access::dao::{AccessError, AccessOAuthCodeData};
use lsys_core::fluent_message;
use serde::{Deserialize, Serialize};

use crate::dao::session::RestAuthData;

use crate::dao::{AppError, AppResult};
use crate::model::{AppModel, AppRequestType};

use super::AppOAuthClient;

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
    pub user_name: &'t str,
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
            let papp = self.app.cache().find_by_id(&app.parent_app_id).await?;
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
        Ok(self
            .access
            .oauth
            .create_code(
                app.parent_app_id, //>0 为外部应用
                app.id,
                &AccessOAuthCodeData {
                    user_data: code_data.user_data,
                    user_name: code_data.user_name,
                    user_account: code_data.user_account,
                    login_ip: code_data.login_ip,
                    device_id: code_data.device_id,
                    device_name: code_data.device_name,
                    session_data,
                },
                self.code_time as usize,
            )
            .await?)
    }
    //根据app,app token,user 数据创建session
    pub async fn create_session(&self, app: &AppModel, code: &str) -> AppResult<RestAuthData> {
        self.check_access(app).await?;
        let session_data = self
            .access
            .oauth
            .code_do_login(
                app.parent_app_id,
                app.id,
                code,
                None,
                self.login_time,
                &[(APP_OAUTH_CODE, code)],
            )
            .await?;
        Ok(RestAuthData::new(app.to_owned(), session_data))
    }
    //删除session
    pub async fn clear_session(&self, app: &AppModel, user_token: &str) -> AppResult<()> {
        let login = self
            .access
            .auth
            .login_data(app.parent_app_id, app.id, user_token)
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
                        .access
                        .oauth
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
        user_token: &str,
        reset_token: bool,
    ) -> AppResult<RestAuthData> {
        self.check_access(app).await?;
        let session_data = self
            .access
            .auth
            .login_data(app.parent_app_id, app.id, user_token)
            .await?;
        let token_data = if reset_token {
            self.access
                .auth
                .refresh_login(&session_data, Some(self.login_time), None)
                .await?
        } else {
            self.access
                .auth
                .extend_login(&session_data, self.login_time)
                .await?
        };
        Ok(RestAuthData::new(app.to_owned(), token_data))
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
