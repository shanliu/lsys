use crate::{
    common::{JsonData, JsonResponse, JsonResult, PageParam, UserAuthQueryDao},
    dao::access::api::system::{CheckAdminRbacEdit, CheckAdminRbacView},
};
use lsys_access::dao::AccessSession;
use lsys_core::FluentMessage;
use lsys_rbac::{
    dao::{ResTypeListParam as DaoResTypeListParam, ResTypeParam},
    model::RbacOpModel,
};
use serde::Deserialize;
use serde_json::json;

//静态资源类型数据

pub async fn res_tpl_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    req_dao.user_session.read().await.get_session_data().await?;
    let tpl_data = req_dao
        .web_dao
        .web_rbac
        .res_tpl_data(false, false)
        .into_iter()
        .map(|e| {
            json!({
                "res_type":e.key,
                "res_name":req_dao.fluent.format_message(&FluentMessage {
                    id: format!("res-admin-{}", e.key),
                    crate_name: env!("CARGO_PKG_NAME").to_string(),
                    data: vec![],
                }),
                "op_data": e.ops
                .iter()
                .map(|eop| {
                    json!({
                        "key":eop,
                        "name": req_dao.fluent.format_message(&FluentMessage {
                            id: format!("res-op-admin-{}", eop),
                            crate_name: env!("CARGO_PKG_NAME").to_string(),
                            data: vec![],
                        })
                    })
                })
                .collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonResponse::data(JsonData::body(
        json!({ "tpl_data": tpl_data }),
    )))
}

//从现有资源统计资源类型
#[derive(Debug, Deserialize)]
pub struct ResTypeListParam {
    pub res_type: String,
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
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminRbacView {})
        .await?;

    let res_param = DaoResTypeListParam {
        user_id: Some(0),
        app_id: Some(0),
        res_type: if param.res_type.is_empty() {
            None
        } else {
            Some(&param.res_type)
        },
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
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminRbacEdit {})
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
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminRbacEdit {})
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
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminRbacView {})
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
