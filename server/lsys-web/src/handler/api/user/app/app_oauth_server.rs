use crate::common::{JsonData, UserAuthQueryDao};
use crate::common::{JsonError, JsonResult};
use crate::dao::access::api::user::CheckUserAppEdit;
use lsys_access::dao::AccessSession;
use lsys_app::dao::AppOAuthServerScopeParam;
use lsys_app::model::AppRequestStatus;
use lsys_app::{
    dao::{AppError, AppResult},
    model::AppModel,
};
use lsys_core::fluent_message;

use serde::Serialize;
use serde_json::json;

pub struct ConfirmOAuthClientParam {
    pub app_id: u64,
    pub confirm_status: i8,
    pub confirm_note: String,
}

pub async fn oauth_server_client_confirm(
    param: &ConfirmOAuthClientParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let confirm_status = AppRequestStatus::try_from(param.confirm_status)?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;

    if app.user_app_id == 0 {
        return Err(JsonError::Message(fluent_message!("not-user-app-confirm")));
    }

    let parent_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&app.parent_app_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppEdit {
                res_user_id: parent_app.user_id,
            },
        )
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_server
        .oauth_check(&parent_app)
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .oauth_confirm(
            &app,
            confirm_status,
            &param.confirm_note,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

pub struct ConfirmOAuthClientScopeParam {
    pub app_id: u64,
    pub app_req_id: u64,
    pub confirm_status: i8,
    pub confirm_note: String,
}

pub async fn oauth_server_client_scope_confirm(
    param: &ConfirmOAuthClientScopeParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let req_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .request_find_by_id(&param.app_req_id)
        .await?;

    let confirm_status = AppRequestStatus::try_from(param.confirm_status)?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;

    if app.user_app_id == 0 {
        return Err(JsonError::Message(fluent_message!("not-user-app-confirm")));
    }

    let parent_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&app.parent_app_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppEdit {
                res_user_id: parent_app.user_id,
            },
        )
        .await?;

    //开通过的不影响scope申请审核

    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .scope_confirm(
            &app,
            &req_app,
            confirm_status,
            &param.confirm_note,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

#[derive(Serialize, Debug, Clone)]
pub struct ScopeItem {
    pub name: String,
    pub key: String,
    pub desc: String,
}

//返回指定应用或系统可用的 oauth server spoce 数据
pub async fn oauth_server_scope_data(
    app: Option<&AppModel>,
    req_dao: &UserAuthQueryDao,
) -> AppResult<Vec<ScopeItem>> {
    match app {
        Some(app) => {
            if app.parent_app_id > 0 {
                return Ok(vec![]);
            }
            Ok(req_dao
                .web_dao
                .web_app
                .app_dao
                .oauth_server
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
pub async fn oauth_server_parse_scope_data(
    app: &AppModel,
    scope_data: &[&str],
    req_dao: &UserAuthQueryDao,
) -> AppResult<JsonData> {
    let papp = if app.parent_app_id > 0 {
        let papp = req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .cache()
            .find_by_id(&app.parent_app_id)
            .await?;
        req_dao
            .web_dao
            .web_app
            .app_dao
            .oauth_server
            .oauth_check(&papp)
            .await?;
        Some(papp)
    } else {
        None
    };
    let server_spoce = req_dao
        .web_dao
        .web_app
        .app_oauth_server_scope_data(papp.as_ref())
        .await?;
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
    let oauth_spoce_str = req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .get_oauth_client_scope_data(app)
        .await?;
    let mut oauth_spoce = oauth_spoce_str.split(",");
    for tmp in out.iter() {
        if !oauth_spoce.any(|e| e == tmp.key.as_str()) {
            return Err(AppError::System(fluent_message!("app-bad-scope",{
                "scope":&tmp.key
            })));
        }
    }
    Ok(JsonData::data(json!({
        "data":out
    })))
}

pub struct OAuthServerRequestData {
    pub app_id: u64,
}

pub async fn oauth_server_request(
    param: &OAuthServerRequestData,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_server
        .oauth_request(&app, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}

pub struct ConfirmOAuthServerSettingScopeParam {
    pub key: String,
    pub name: String,
    pub desc: String,
}

pub struct ConfirmOAuthServerSettingParam {
    app_id: u64,
    scope_data: Vec<ConfirmOAuthServerSettingScopeParam>,
}

pub async fn oauth_server_setting(
    param: &ConfirmOAuthServerSettingParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_server
        .oauth_check(&app)
        .await?;

    let scope_data = param
        .scope_data
        .iter()
        .map(|e| AppOAuthServerScopeParam {
            key: &e.key,
            name: &e.name,
            desc: &e.desc,
        })
        .collect::<Vec<_>>();
    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_server
        .oauth_setting(
            &app,
            &scope_data,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}
