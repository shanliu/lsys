use super::inner_app_rbac_check;
use super::inner_app_self_check;
use super::inner_user_data_to_user_id;
use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::PageParam;
use crate::common::RequestDao;
use lsys_access::dao::UserInfo;
use lsys_app::model::AppModel;
use lsys_rbac::dao::RbacRoleAddData;
use lsys_rbac::dao::RbacRoleUserRangeData;
use lsys_rbac::dao::RoleDataAttrParam;
use lsys_rbac::{
    dao::RoleDataParam as DaoRoleDataParam,
    model::{RbacRoleResRange, RbacRoleUserRange},
};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct RoleAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub use_app_user: bool,
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub role_key: String,
    pub role_name: Option<String>,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub user_range: i8,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub res_range: i8,
}

pub async fn role_add(
    param: &RoleAddParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    let target_user_id = inner_user_data_to_user_id(
        app,
        param.use_app_user,
        param.user_param.as_deref(),
        req_dao,
    )
    .await?;

    let role_info = match RbacRoleUserRange::try_from(param.user_range)? {
        RbacRoleUserRange::Custom => RbacRoleUserRangeData::Custom {
            role_name: param.role_name.as_deref().unwrap_or_default(),
        },
        RbacRoleUserRange::Session => RbacRoleUserRangeData::Session {
            role_key: &param.role_key,
            role_name: param.role_name.as_deref().and_then(|e| {
                if !e.is_empty() {
                    Some(e)
                } else {
                    None
                }
            }),
        },
    };
    let res_range = RbacRoleResRange::try_from(param.res_range)?;
    let id = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .add_role(
            &RbacRoleAddData {
                user_id: target_user_id,
                app_id: Some(app.id),
                role_info,
                res_range,
            },
            app.user_id,
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": id.id }))))
}

#[derive(Debug, Deserialize)]
pub struct RoleEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    pub role_key: String,
    pub role_name: Option<String>,
}

pub async fn role_edit(
    param: &RoleEditParam,
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
    let role_data = match RbacRoleUserRange::try_from(role.user_range)? {
        RbacRoleUserRange::Custom => RbacRoleUserRangeData::Custom {
            role_name: param.role_name.as_deref().unwrap_or_default(),
        },
        RbacRoleUserRange::Session => RbacRoleUserRangeData::Session {
            role_key: &param.role_key,
            role_name: param.role_name.as_deref().and_then(|e| {
                if !e.is_empty() {
                    Some(e)
                } else {
                    None
                }
            }),
        },
    };
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .edit_role(&role, &role_data, app.user_id, None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct RoleDelParam {
    pub role_id: u64,
}

pub async fn role_del(
    param: &RoleDelParam,
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
        .del_role(&role, app.user_id, None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}
#[derive(Debug, Deserialize)]
pub struct RoleDataParam {
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub use_app_user: bool,
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub role_key: Option<String>,
    pub role_name: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::common::deserialize_option_vec_u64"
    )]
    pub ids: Option<Vec<u64>>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub user_range: Option<i8>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub user_count: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub user_data: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub res_range: Option<i8>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub res_count: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub res_op_count: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct RoleUserDataRecord {
    pub user_data: Option<UserInfo>,
    pub timeout: u64,
    pub change_time: u64,
}

#[derive(Debug, Serialize)]
pub struct RoleDataRecord {
    pub id: u64,
    pub user_id: u64,
    pub role_key: String,
    pub user_range: i8,
    pub res_range: i8,
    pub role_name: String,
    pub change_time: u64,
    pub user_count: Option<i64>,
    pub user_list: Option<Vec<RoleUserDataRecord>>,
    pub res_count: Option<i64>,
    pub res_op_count: Option<i64>,
}

pub async fn role_data(
    param: &RoleDataParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;

    let target_user_id = inner_user_data_to_user_id(
        app,
        param.use_app_user,
        param.user_param.as_deref(),
        req_dao,
    )
    .await?;

    let user_range = if let Some(e) = param.user_range {
        Some(RbacRoleUserRange::try_from(e)?)
    } else {
        None
    };
    let res_range = if let Some(e) = param.res_range {
        Some(RbacRoleResRange::try_from(e)?)
    } else {
        None
    };

    let role_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .role_info(
            &DaoRoleDataParam {
                user_id: target_user_id,
                app_id: Some(app.id),
                ids: param.ids.as_deref(),
                user_range,
                res_range,
                role_key: param.role_key.as_deref(),
                role_name: param.role_name.as_deref(),
            },
            &RoleDataAttrParam {
                user_count: param.user_count,
                user_data: param.user_data,
                res_count: param.res_count,
                res_op_count: param.res_op_count,
            },
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let user_data_limit = param.user_data.unwrap_or(0);
    let user_info_set = if !role_data.is_empty() && user_data_limit > 0 {
        let role_ids = role_data.iter().map(|e| e.0.id).collect::<Vec<_>>();
        let user_data = req_dao
            .web_dao
            .web_rbac
            .rbac_dao
            .role
            .role_user_group_data(
                &role_ids,
                None,
                false,
                Some(&lsys_core::PageParam::new(0, user_data_limit)),
            )
            .await?;

        let user_data_ids = user_data
            .iter()
            .flat_map(|e| e.1.iter().map(|f| f.user_id).collect::<Vec<u64>>())
            .collect::<Vec<_>>();
        Some(
            req_dao
                .web_dao
                .web_access
                .access_dao
                .user
                .cache()
                .find_users_by_ids(&user_data_ids)
                .await?,
        )
    } else {
        None
    };
    let role_data = role_data
        .into_iter()
        .map(|(e, info)| {
            let user_list = info.user_data.map(|e| {
                e.iter()
                    .map(|f| RoleUserDataRecord {
                        timeout: f.timeout,
                        change_time: f.change_time,
                        user_data: user_info_set
                            .as_ref()
                            .and_then(|t| t.get(f.user_id).to_owned()),
                    })
                    .collect::<Vec<_>>()
            });
            RoleDataRecord {
                id: e.id,
                user_id: e.user_id,
                role_key: e.role_key,
                user_range: e.user_range,
                res_range: e.res_range,
                role_name: e.role_name,
                change_time: e.change_time,
                user_count: info.user_count,
                user_list,
                res_count: info.res_count,
                res_op_count: info.res_op_count,
            }
        })
        .collect::<Vec<_>>();

    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .role
                .role_count(&DaoRoleDataParam {
                    user_id: target_user_id,
                    app_id: Some(app.id),
                    ids: param.ids.as_deref(),
                    user_range,
                    res_range,
                    role_key: param.role_key.as_deref(),
                    role_name: param.role_name.as_deref(),
                })
                .await?,
        )
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": bind_vec_user_info_from_req!(req_dao, role_data, user_id),
        "count": count
    }))))
}
