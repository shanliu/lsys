use crate::common::JsonData;
use crate::common::JsonError;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::dao::access::api::system::admin::{
    CheckAdminBase, CheckAdminMailConfig, CheckAdminSmsConfig,
};
use crate::dao::access::api::system::user::CheckUserAppSenderMailView;
use crate::dao::access::api::system::user::CheckUserAppSenderSmsView;
use crate::dao::access::api::system::user::CheckUserAppView;
use crate::dao::access::api::system::user::CheckUserBarCodeView;
use crate::dao::access::api::system::user::CheckUserRbacView;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::RbacCheckAccessDepend;
use crate::dao::WebRbac;
use lsys_access::dao::AccessSession;
use lsys_access::dao::SessionBody;
use lsys_app::model::AppModel;
use lsys_core::fluent_message;
use lsys_core::RequestEnv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
#[derive(Debug, Deserialize)]
pub struct RbacAccessParam {
    pub name: String,
    pub data: Option<Value>,
}
#[derive(Debug, Deserialize)]
pub struct RbacAccessMenuParam {
    pub check_data: Vec<RbacAccessParam>,
}

#[derive(Debug, Serialize)]
pub struct RbacMenuRecord<'t> {
    pub status: i8,              //状态: 1=检查通过, 2=check方法失败, 0=其他失败
    pub name: &'t str,           //参见perm_check 定义
    pub data: &'t Option<Value>, //参见perm_check 定义
    pub msg: Option<String>,
}

async fn perm_check(
    check_dep: &RbacCheckAccessDepend,
    web_rbac: &WebRbac,
    user_session: &SessionBody,
    req_env: &RequestEnv,
) -> JsonResult<()> {
    Ok(web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(user_session, req_env),
            check_dep,
        )
        .await?)
}

#[derive(Debug, Deserialize)]
struct RbackCheckApp {
    app_id: u64,
}

async fn perm_parse_app_id(
    check_res: &RbacAccessParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<AppModel> {
    let data: RbackCheckApp = serde_json::from_value(
        check_res
            .data
            .as_ref()
            .ok_or(JsonError::Message(fluent_message!("rbac-need-data",{
                "res":&check_res.name
            })))?
            .to_owned(),
    )
    .map_err(|e| {
        JsonError::Message(fluent_message!("rbac-parse-data-fail",{
            "res":&check_res.name,
            "err":&e.to_string()
        }))
    })?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_id(data.app_id)
        .await?;
    Ok(app)
}

async fn perm_map_check(check_res: &RbacAccessParam, req_dao: &UserAuthQueryDao) -> Result<i8, (i8, JsonError)> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await.map_err(|e| (0, e.into()))?;

    match check_res.name.as_str() {
        "admin:app:request:list" => Err((0, JsonError::Message(fluent_message!("rbac-unkown-res",{
            "res":&check_res.name
        })))),
        "app:subapp" => {
            let app = perm_parse_app_id(check_res, req_dao).await.map_err(|e| (0, e))?;
            if app.parent_app_id != 0 {
                return Err((0, JsonError::Message(fluent_message!("app-is-subapp"))));
            }
            req_dao
                .web_dao
                .web_rbac
                .check(
                    &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
                    &CheckUserAppView {
                        res_user_id: app.user_id,
                    },
                )
                .await.map_err(|e| (2, e.into()))?;
            req_dao
                .web_dao
                .web_app
                .app_dao
                .app
                .inner_feature_sub_app_check(&app)
                .await.map_err(|e| (2, e.into()))?;
            Ok(1)
        }
        "app:oauth:server" => {
            let app = perm_parse_app_id(check_res, req_dao).await.map_err(|e| (0, e))?;
            if app.parent_app_id != 0 {
                return Err((0, JsonError::Message(fluent_message!("app-is-subapp"))));
            }
            req_dao
                .web_dao
                .web_rbac
                .check(
                    &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
                    &CheckUserAppView {
                        res_user_id: app.user_id,
                    },
                )
                .await.map_err(|e| (2, e.into()))?;
            req_dao
                .web_dao
                .web_app
                .app_dao
                .app
                .cache()
                .feature_check(
                    &app,
                    &[lsys_app::model::AppRequestType::OAuthServer.feature_key()],
                )
                .await.map_err(|e| (2, e.into()))?;
            Ok(1)
        }
        "app:mail" => {
            let app = perm_parse_app_id(check_res, req_dao).await.map_err(|e| (0, e))?;
            req_dao
                .web_dao
                .web_rbac
                .check(
                    &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
                    &CheckUserAppSenderMailView {
                        res_user_id: app.user_id,
                    },
                )
                .await.map_err(|e| (2, e.into()))?;
            req_dao
                .web_dao
                .web_app
                .app_dao
                .app
                .cache()
                .exter_feature_check(&app, &[crate::handler::APP_FEATURE_MAIL])
                .await.map_err(|e| (2, e.into()))?;
            Ok(1)
        }
        "app:sms" => {
            let app = perm_parse_app_id(check_res, req_dao).await.map_err(|e| (0, e))?;
            req_dao
                .web_dao
                .web_rbac
                .check(
                    &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
                    &CheckUserAppSenderSmsView {
                        res_user_id: app.user_id,
                    },
                )
                .await.map_err(|e| (2, e.into()))?;
            req_dao
                .web_dao
                .web_app
                .app_dao
                .app
                .cache()
                .exter_feature_check(&app, &[crate::handler::APP_FEATURE_SMS])
                .await.map_err(|e| (2, e.into()))?;
            Ok(1)
        }
        "app:rbac" => {
            let app = perm_parse_app_id(check_res, req_dao).await.map_err(|e| (0, e))?;
            req_dao
                .web_dao
                .web_rbac
                .check(
                    &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
                    &CheckUserRbacView {
                        res_user_id: app.user_id,
                    },
                )
                .await.map_err(|e| (2, e.into()))?;
            req_dao
                .web_dao
                .web_app
                .app_dao
                .app
                .cache()
                .exter_feature_check(&app, &[crate::handler::APP_FEATURE_RBAC])
                .await.map_err(|e| (2, e.into()))?;
            Ok(1)
        }
        "app:barcode" => {
            let app = perm_parse_app_id(check_res, req_dao).await.map_err(|e| (0, e))?;
            req_dao
                .web_dao
                .web_rbac
                .check(
                    &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
                    &CheckUserBarCodeView {
                        res_user_id: app.user_id,
                    },
                )
                .await.map_err(|e| (2, e.into()))?;
            req_dao
                .web_dao
                .web_app
                .app_dao
                .app
                .cache()
                .exter_feature_check(&app, &[crate::handler::APP_FEATURE_BARCODE])
                .await.map_err(|e| (2, e.into()))?;
            Ok(1)
        }
        "system:app:sub" => Ok(1),
        "admin-sms-config" => {
            perm_check(
                &CheckAdminBase {},
                &req_dao.web_dao.web_rbac,
                &auth_data,
                &req_dao.req_env,
            )
            .await.map_err(|e| (2, e))?;
            perm_check(
                &CheckAdminSmsConfig {},
                &req_dao.web_dao.web_rbac,
                &auth_data,
                &req_dao.req_env,
            )
            .await.map_err(|e| (2, e))?;
            Ok(1)
        }
        "admin-mail-config" => {
            perm_check(
                &CheckAdminBase {},
                &req_dao.web_dao.web_rbac,
                &auth_data,
                &req_dao.req_env,
            )
            .await.map_err(|e| (2, e))?;
            perm_check(
                &CheckAdminMailConfig {},
                &req_dao.web_dao.web_rbac,
                &auth_data,
                &req_dao.req_env,
            )
            .await.map_err(|e| (2, e))?;
            Ok(1)
        }
        _ => {
            if req_dao.web_dao.web_rbac.is_root(auth_data.user_id()) {
                Ok(1)
            } else {
                Err((0, JsonError::Message(fluent_message!("rbac-unkown-res",{
                    "res":&check_res.name
                }))))
            }
        }
    }
}

pub async fn perm_check_list(
    param: &RbacAccessMenuParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let mut out = Vec::with_capacity(param.check_data.len());
    for e in param.check_data.iter() {
        let res = perm_map_check(e, req_dao).await;
        let (status, msg) = match res {
            Ok(status) => (status, None),
            Err((status, err)) => (status, Some(req_dao.fluent_error_string(&err))),
        };
        out.push(RbacMenuRecord {
            status,
            name: e.name.as_str(),
            data: &e.data,
            msg,
        });
    }
    Ok(JsonResponse::data(JsonData::body(json!({"record":out}))))
}
