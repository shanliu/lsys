use super::app_check_get;
use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::LimitParam;
use crate::common::PageParam;
use crate::common::UserAuthQueryDao;
use lsys_access::dao::AccessSession;
use lsys_access::dao::UserDataParam;
use lsys_access::dao::UserInfo;
use lsys_rbac::dao::RoleAddUser;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct AppRoleUserItemParam {
    pub user_param: Option<String>,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub timeout: u64,
}
#[derive(Debug, Deserialize)]
pub struct AppRoleUserAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    pub user_data: Vec<AppRoleUserItemParam>,
}

pub async fn app_role_user_add(
    param: &AppRoleUserAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;
    let app = app_check_get(role.app_id, true, &auth_data, req_dao).await?;
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
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .add_user(&role, &add_user, app.id, None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct AppRoleUserDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    pub user_data: Vec<String>,
}

pub async fn app_role_user_del(
    param: &AppRoleUserDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;

    let mut user_id_data = vec![];
    for tmp in &param.user_data {
        let user_info = req_dao
            .web_dao
            .web_access
            .access_dao
            .user
            .cache()
            .sync_user(role.app_id, tmp, None, None)
            .await?;
        user_id_data.push(user_info.id);
    }

    let app = app_check_get(role.app_id, true, &auth_data, req_dao).await?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .del_user(&role, &user_id_data, app.id, None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct AppRoleUserDataParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub all: bool,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn app_role_user_data(
    param: &AppRoleUserDataParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;
    app_check_get(role.app_id, false, &auth_data, req_dao).await?;
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

#[derive(Debug, Deserialize)]
pub struct AppRoleUserAvailableParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub user_any: Option<String>,
    pub limit: Option<LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn app_role_user_available(
    param: &AppRoleUserAvailableParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = app_check_get(param.app_id, true, &auth_data, req_dao).await?;
    let user_param = UserDataParam {
        app_id: Some(app.id),
        user_data: None,
        user_account: None,
        user_any: param.user_any.as_deref(),
    };
    let (res, next) = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .user_data(&user_param, param.limit.as_ref().map(|e| e.into()).as_ref())
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_access
                .access_dao
                .user
                .user_count(&user_param)
                .await?,
        )
    } else {
        None
    };
    let out_res = res.into_iter().map(UserInfo::from).collect::<Vec<_>>();
    Ok(JsonResponse::data(JsonData::body(json!({
        "data":out_res,
        "next":next,
        "total":count
    }))))
}
