use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonData, JsonResponse, JsonResult, PageParam, UserAuthQueryDao},
    dao::access::api::system::admin::{CheckAdminRbacEdit, CheckAdminRbacView},
};
use lsys_access::dao::AccessSession;
use lsys_rbac::dao::{RbacResAddData, RbacResData, ResDataParam};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ResAddParam {
    pub res_name: Option<String>,
    pub res_type: String,
    pub res_data: String,
}
//资源添加
pub async fn res_add(param: &ResAddParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminRbacEdit {},
        )
        .await?;
    let id = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .add_res(
            &RbacResAddData {
                user_id: 0,
                app_id: Some(0),
                res_info: RbacResData {
                    res_name: param.res_name.as_deref().and_then(|e| {
                        if !e.is_empty() {
                            Some(e)
                        } else {
                            None
                        }
                    }),
                    res_type: &param.res_type,
                    res_data: &param.res_data,
                },
            },
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({ "id": id }))))
}

#[derive(Debug, Deserialize)]
pub struct ResEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub res_id: u64,
    pub res_name: Option<String>,
    pub res_type: String,
    pub res_data: String,
}
//资源编辑
pub async fn res_edit(
    param: &ResEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminRbacEdit {},
        )
        .await?;
    let op = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .find_by_id(&param.res_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .edit_res(
            &op,
            &RbacResData {
                res_name: param.res_name.as_deref().and_then(|e| {
                    if !e.is_empty() {
                        Some(e)
                    } else {
                        None
                    }
                }),
                res_type: &param.res_type,
                res_data: &param.res_data,
            },
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ResDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub res_id: u64,
}
//资源删除
pub async fn res_del(param: &ResDelParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminRbacEdit {},
        )
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .find_by_id(&param.res_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .del_res(&res, auth_data.user_id(), None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ResParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub user_id: Option<u64>,
    pub res_type: Option<String>,
    pub res_data: Option<String>,
    pub res_name: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::common::deserialize_option_vec_u64"
    )]
    pub ids: Option<Vec<u64>>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}
//资源列表
pub async fn res_data(param: &ResParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminRbacView {},
        )
        .await?;
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_data(
            &ResDataParam {
                user_id: param.user_id,
                app_id: None,
                res_data: param.res_data.as_deref(),
                res_type: param.res_type.as_deref(),
                res_name: param.res_name.as_deref(),
                ids: param.ids.as_deref(),
            },
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .res
                .res_count(&ResDataParam {
                    user_id: param.user_id,
                    app_id: None,
                    res_data: param.res_data.as_deref(),
                    res_type: param.res_type.as_deref(),
                    res_name: param.res_name.as_deref(),
                    ids: param.ids.as_deref(),
                })
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": res,"total":count}),
    )))
}
