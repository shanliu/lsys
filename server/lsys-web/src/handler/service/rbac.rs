//! 服务间RBAC权限检查接口
use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, AccessSessionRole};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// 资源操作权限参数
#[derive(Debug, Deserialize)]
pub struct ResReqAuthParam {
    pub op_key: String,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub req_auth: bool,
}

/// 资源检查参数
#[derive(Debug, Deserialize)]
pub struct ResCheckParam {
    pub res_type: String,
    pub res_data: String,
    pub res_user_id: u64,
    pub ops: Vec<ResReqAuthParam>,
}

/// 角色检查参数
#[derive(Debug, Deserialize)]
pub struct RoleCheckParam {
    pub role_key: String,
    pub user_id: u64,
}

/// 访问检查参数
#[derive(Debug, Deserialize)]
pub struct AccessCheckParam {
    pub role_key: Vec<RoleCheckParam>,
    pub check_res: Vec<Vec<ResCheckParam>>,
}

/// RBAC检查参数
#[derive(Debug, Deserialize)]
pub struct RbacCheckParam {
    pub user_id: u64,
    pub token_data: Option<String>,
    pub access: AccessCheckParam,
}

/// RBAC菜单项参数
#[derive(Debug, Deserialize)]
pub struct RbacMenuItemParam {
    pub name: String,
    pub check_res: RbacCheckParam,
}

/// RBAC菜单列表参数
#[derive(Debug, Deserialize)]
pub struct RbacMenuListParam {
    pub menu_res: Vec<RbacMenuItemParam>,
}

/// RBAC菜单状态结果
#[derive(Debug, Serialize)]
pub struct RbacMenuStatus {
    pub status: bool,
    pub name: String,
}

/// 批量检查RBAC权限
///
/// 该接口用于服务间调用批量检查多个菜单项的RBAC权限，
/// 返回每个菜单项的权限状态
pub async fn check_list(
    param: &RbacMenuListParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let mut out = Vec::with_capacity(param.menu_res.len());
    for e in param.menu_res.iter() {
        out.push(RbacMenuStatus {
            status: inner_access_check(&e.check_res, req_dao)
                .await
                .map(|_| true)
                .unwrap_or(false),
            name: e.name.to_owned(),
        });
    }
    Ok(JsonResponse::data(JsonData::body(json!({"result": out}))))
}

/// 内部权限检查实现
async fn inner_access_check(param: &RbacCheckParam, req_dao: &RequestDao) -> Result<(), String> {
    let user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&param.user_id)
        .await
        .map_err(|e| format!("user not found: {:?}", e))?;

    let session_role = param
        .access
        .role_key
        .iter()
        .map(|e| AccessSessionRole {
            role_key: &e.role_key,
            user_id: e.user_id,
            app_id: user.app_id,
        })
        .collect::<Vec<_>>();

    let req_env = req_dao.req_env.clone();
    let check_env = AccessCheckEnv {
        user_req_env: Some(&req_env),
        user_app_id: user.app_id,
        user_id: user.id,
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
                    user_id: check_res.res_user_id,
                    res_type: &check_res.res_type,
                    res_data: &check_res.res_data,
                    app_id: user.app_id,
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
        .await
        .map_err(|e| format!("rbac check failed: {:?}", e))?;
    Ok(())
}
