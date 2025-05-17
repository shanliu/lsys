// oauth 中 scope 数据操作相关封装
use crate::dao::WebApp;
use lsys_app::{
    dao::{AppError, AppResult},
    model::AppModel,
};
use lsys_core::fluent_message;

use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct ScopeItem {
    pub name: String,
    pub key: String,
    pub desc: String,
}

impl WebApp {
    //返回指定应用或系统可用的 oauth server spoce 数据
    pub async fn app_oauth_server_scope_data(
        &self,
        app: Option<&AppModel>,
    ) -> AppResult<Vec<ScopeItem>> {
        match app {
            Some(app) => {
                if app.parent_app_id > 0 {
                    return Ok(vec![]);
                }
                Ok(self
                    .app_dao
                    .oauth_server
                    .cache()
                    .get_scope(app)
                    .await?
                    .into_iter()
                    .map(|e| ScopeItem {
                        name: e.scope_name,
                        key: e.scope_key,
                        desc: e.scope_desc,
                    })
                    .collect::<Vec<_>>())
            }
            None => Ok(vec![
                ScopeItem {
                    name: "用户资料".to_string(),
                    key: "user_info".to_string(),
                    desc: "用户资料".to_string(),
                },
                ScopeItem {
                    name: "用户邮箱".to_string(),
                    key: "user_email".to_string(),
                    desc: "用户邮箱".to_string(),
                },
                ScopeItem {
                    name: "用户手机号".to_string(),
                    key: "user_mobile".to_string(),
                    desc: "用户手机号".to_string(),
                },
                ScopeItem {
                    name: "用户收货地址".to_string(),
                    key: "user_address".to_string(),
                    desc: "用户收货地址".to_string(),
                },
            ]),
        }
    }
    //解析并校验OAUTH登录请求的 spoce 数据
    pub async fn app_oauth_server_parse_scope_data(
        &self,
        app: &AppModel,
        scope_data: &[&str],
    ) -> AppResult<Vec<ScopeItem>> {
        let papp = if app.parent_app_id > 0 {
            Some(
                self.app_dao
                    .app
                    .cache()
                    .find_by_id(app.parent_app_id)
                    .await?,
            )
        } else {
            None
        };
        let server_spoce = self.app_oauth_server_scope_data(papp.as_ref()).await?;
        let mut out = vec![];
        for tmp in scope_data {
            if let Some(t) = server_spoce.iter().find(|e| e.key.as_str() == *tmp) {
                out.push(t.to_owned());
            } else {
                return Err(AppError::System(
                    fluent_message!("app-oauth-login-bad-scope",{
                        "scope_data":tmp
                    }),
                ));
            }
        }
        let oauth_spoce_str = self
            .app_dao
            .oauth_client
            .cache()
            .find_by_app(app)
            .await?
            .scope_data;
        let mut oauth_spoce = oauth_spoce_str.split(",");
        for tmp in out.iter() {
            if !oauth_spoce.any(|e| e == tmp.key.as_str()) {
                return Err(AppError::System(fluent_message!("app-bad-scope",{
                    "scope":&tmp.key
                })));
            }
        }
        Ok(out)
    }
    //获取指定APP的 spoce 数据
    pub async fn app_oauth_client_get_scope_data(
        &self,
        app: &AppModel,
    ) -> AppResult<Vec<ScopeItem>> {
        let papp = if app.parent_app_id > 0 {
            Some(
                self.app_dao
                    .app
                    .cache()
                    .find_by_id(app.parent_app_id)
                    .await?,
            )
        } else {
            None
        };
        let oauth_spoce_str = self
            .app_dao
            .oauth_client
            .cache()
            .find_by_app(app)
            .await?
            .scope_data;
        let oauth_spoce = oauth_spoce_str.split(",");
        let server_spoce = self.app_oauth_server_scope_data(papp.as_ref()).await?;
        let mut out = vec![];
        for tmp in oauth_spoce {
            if let Some(t) = server_spoce.iter().find(|e| e.key.as_str() == tmp) {
                out.push(t.to_owned());
            }
        }
        Ok(out)
    }
}
