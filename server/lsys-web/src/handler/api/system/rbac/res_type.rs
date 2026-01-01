use crate::{
    common::{JsonData, JsonResponse, JsonResult, PageParam, UserAuthQueryDao},
    dao::access::api::system::admin::{CheckAdminRbacEdit, CheckAdminRbacView},
};
use lsys_access::dao::AccessSession;
use lsys_rbac::{
    dao::{ResTypeListParam as DaoResTypeListParam, ResTypeParam},
    model::RbacOpModel,
};
use crate::dao::access::RbacAccessCheckEnv;
use serde::Deserialize;
use serde_json::json;

//从现有资源统计资源类型
#[derive(Debug, Deserialize)]
pub struct ResTypeListParam {
    pub res_type: Option<String>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn res_type_data(
    param: &ResTypeListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env), &CheckAdminRbacView {})
        .await?;

    let res_param = DaoResTypeListParam {
        user_id: Some(0),
        app_id: Some(0),
        res_type: param
            .res_type
            .as_deref()
            .and_then(|e| if !e.is_empty() { Some(e) } else { None }),
    };

    let rows = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_type_data(&res_param, param.page.as_ref().map(|e| e.into()).as_ref())
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .res
                .res_type_count(&res_param)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": rows,"total":count}),
    )))
}

//往资源类型加操作
#[derive(Debug, Deserialize)]
pub struct ResTypeAddOpParam {
    pub res_type: String,
    #[serde(deserialize_with = "crate::common::deserialize_vec_u64")]
    pub op_ids: Vec<u64>,
}

pub async fn res_type_op_add(
    param: &ResTypeAddOpParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env), &CheckAdminRbacEdit {})
        .await?;

    let op_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_ids(&param.op_ids)
        .await?
        .into_iter()
        .map(|e| e.1)
        .collect::<Vec<RbacOpModel>>();
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_type_add_op(
            &ResTypeParam {
                res_type: &param.res_type,
                user_id: 0,
                app_id: 0,
            },
            &op_data,
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ResDelOpParam {
    pub res_type: String,
    #[serde(deserialize_with = "crate::common::deserialize_vec_u64")]
    pub op_ids: Vec<u64>,
}

//往资源类型删操作
pub async fn res_type_op_del(
    param: &ResDelOpParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env), &CheckAdminRbacEdit {})
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_type_del_op(
            &ResTypeParam {
                res_type: &param.res_type,
                user_id: 0,
                app_id: 0,
            },
            &param.op_ids,
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ResTypeOpListParam {
    pub res_type: String,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

//指定资源类型已绑定操作数据
pub async fn res_type_op_data(
    param: &ResTypeOpListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env), &CheckAdminRbacView {})
        .await?;

    let res_param = ResTypeParam {
        res_type: &param.res_type,
        user_id: 0,
        app_id: 0,
    };

    let rows = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_type_op_data(
            &res_param,
            None,
            true,
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
                .res_type_op_count(&res_param)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": rows,"total":count}),
    )))
}
