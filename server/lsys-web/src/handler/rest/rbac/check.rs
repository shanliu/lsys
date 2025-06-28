use super::{inner_app_rbac_check, inner_user_data_to_user_id};
use crate::common::JsonData;
use crate::common::{JsonResponse, JsonResult, RequestDao};
use lsys_app::model::AppModel;
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, AccessSessionRole};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ResReqAuthParam {
    pub op_key: String, //资源KEY
    pub req_auth: bool, //资源KEY
}

#[derive(Debug, Deserialize)]
pub struct ResCheckParam {
    pub res_type: String,           //资源KEY
    pub res_data: String,           //资源KEY
    pub use_app_user: bool,         //使用当前APP用户
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub ops: Vec<ResReqAuthParam>,  //授权列表
}

#[derive(Debug, Deserialize)]
pub struct RoleCheckParam {
    pub role_key: String,
    pub use_app_user: bool,
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
}

#[derive(Debug, Deserialize)]
pub struct AccessCheckParam {
    pub role_key: Vec<RoleCheckParam>, //会话角色,如登录用户角色等
    pub check_res: Vec<Vec<ResCheckParam>>,
}

#[derive(Debug, Deserialize)]
pub struct CheckParam {
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub token_data: Option<String>,
    pub request_ip: Option<String>,
    pub access: AccessCheckParam,
}

pub async fn access_check(
    param: &CheckParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    inner_access_check(param, app, req_dao).await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct RbacMenuItemParam {
    pub name: String,
    pub check_res: CheckParam,
}
#[derive(Debug, Deserialize)]
pub struct RbacMenuListParam {
    pub menu_res: Vec<RbacMenuItemParam>,
}
#[derive(Debug, Serialize)]
pub struct RbacMenuStatus {
    pub status: bool, //是否授权成功
    pub name: String, //key,参见perm_check 定义
}

pub async fn access_list_check(
    param: &RbacMenuListParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    let mut out = Vec::with_capacity(param.menu_res.len());
    for e in param.menu_res.iter() {
        out.push(RbacMenuStatus {
            status: inner_access_check(&e.check_res, app, req_dao)
                .await
                .map(|_| true)
                .unwrap_or(false),
            name: e.name.to_owned(),
        })
    }
    Ok(JsonResponse::data(JsonData::body(json!({"result":out}))))
}

async fn inner_access_check(
    param: &CheckParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<()> {
    let user_data = match param.user_param.as_ref() {
        Some(user_data) => Some(
            req_dao
                .web_dao
                .web_access
                .access_dao
                .user
                .cache()
                .sync_user(app.id, user_data, None, None)
                .await?,
        ),
        None => None,
    };

    let mut user_list = HashMap::new();
    for e in param.access.role_key.iter() {
        if !user_list.contains_key(&e.user_param) {
            let user_id =
                inner_user_data_to_user_id(app, e.use_app_user, e.user_param.as_deref(), req_dao)
                    .await?;
            user_list.insert(e.user_param.clone(), user_id);
        }
    }
    for te in param.access.check_res.iter() {
        for e in te.iter() {
            if !user_list.contains_key(&e.user_param) {
                let user_id = inner_user_data_to_user_id(
                    app,
                    e.use_app_user,
                    e.user_param.as_deref(),
                    req_dao,
                )
                .await?;
                user_list.insert(e.user_param.clone(), user_id);
            }
        }
    }

    let session_role = param
        .access
        .role_key
        .iter()
        .map(|e| AccessSessionRole {
            role_key: &e.role_key,
            user_id: user_list.get(&e.user_param).copied().unwrap_or(app.user_id),
            app_id: app.id,
        })
        .collect::<Vec<_>>();
    let mut req_env = req_dao.req_env.clone();
    req_env.request_ip = param.request_ip.clone();
    let check_env = AccessCheckEnv {
        user_req_env: Some(&req_env),
        user_app_id: user_data.as_ref().map(|e| e.app_id).unwrap_or_default(),
        user_id: user_data.map(|e| e.id).unwrap_or_default(),
        user_login_token: param.token_data.as_deref(),
        session_role,
    };
    let access_checks = param
        .access
        .check_res
        .iter()
        .map(|check_res_group| {
            check_res_group
                .iter()
                .map(|check_res| AccessCheckRes {
                    user_id: user_list
                        .get(&check_res.user_param)
                        .copied()
                        .unwrap_or(app.user_id),
                    res_type: &check_res.res_type,
                    res_data: &check_res.res_data,
                    app_id: app.id,
                    op_key_data: check_res
                        .ops
                        .iter()
                        .map(|e| AccessCheckOp {
                            op_key: &e.op_key,
                            req_auth: e.req_auth,
                        })
                        .collect::<Vec<_>>(),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .list_check(
            &check_env,
            &access_checks
                .iter()
                .map(|e| e.as_slice())
                .collect::<Vec<_>>(),
        )
        .await?;
    Ok(())
}
