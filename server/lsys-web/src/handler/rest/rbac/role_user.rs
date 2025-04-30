use super::inner_app_rbac_check;
use super::inner_app_self_check;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::PageParam;
use crate::common::RequestDao;

use crate::common::JsonData;
use lsys_app::model::AppModel;

use lsys_rbac::dao::RoleAddUser;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct RoleUserItemParam {
    pub user_param: Option<String>,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub timeout: u64,
}
#[derive(Debug, Deserialize)]
pub struct RoleUserAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    pub user_data: Vec<RoleUserItemParam>,
}

pub async fn role_user_add(
    param: &RoleUserAddParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    let mut add_user = Vec::with_capacity(param.user_data.len());
    for e in param.user_data.iter() {
        match e.user_param.as_ref().and_then(|s| {
            if s.trim_matches(['\n', ' ', '\t']).is_empty() {
                None
            } else {
                Some(s)
            }
        }) {
            Some(user_param) => {
                let user_info = req_dao
                    .web_dao
                    .web_access
                    .access_dao
                    .user
                    .cache()
                    .sync_user(app.id, user_param, None, None)
                    .await?;
                add_user.push(RoleAddUser {
                    user_id: user_info.id,
                    timeout: e.timeout,
                });
            }
            None => {
                add_user.push(RoleAddUser {
                    user_id: app.user_id,
                    timeout: e.timeout,
                });
            }
        }
    }

    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;
    inner_app_self_check(app, role.app_id)?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .add_user(&role, &add_user, app.user_id, None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}
#[derive(Debug, Deserialize)]
pub struct RoleUserDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_vec_u64")]
    pub user_data: Vec<u64>,
}

pub async fn role_user_del(
    param: &RoleUserDelParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;

    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;
    inner_app_self_check(app, role.app_id)?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .del_user(
            &role,
            &param.user_data,
            app.id,
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct RoleUserDataParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub all: bool,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn role_user_data(
    param: &RoleUserDataParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;
    inner_app_self_check(app, role.app_id)?;
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .role_user_data(
            &role,
            param.all,
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .role
                .role_user_count(&role, param.all)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": res,
        "count": count
    }))))
}
