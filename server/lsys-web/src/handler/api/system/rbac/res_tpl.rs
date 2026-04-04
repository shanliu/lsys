use crate::common::{
    JsonData, JsonPageData, JsonResponse, JsonResult, LimitParam, ToCursorPageParam, UserAuthQueryDao,
};
use lsys_access::dao::{AccessSession, UserDataParam};
use lsys_core::api_utils::{PageCursorValue, PageTotalRowValue};
use lsys_core::db::TotalParam;
use lsys_core::{db::CursorPageSort, fluents::FluentMessage};
use serde::Deserialize;
use serde_json::{json, Value};

//静态模板资源数据

pub async fn static_res_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
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

//动态全局模板类型资源
//动态资源按类型返回,传给 dynamic_res_data 返回每个类型的资源数据
pub async fn dynamic_res_type(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    req_dao.user_session.read().await.get_session_data().await?;
    let tpl_data = req_dao.web_dao.web_rbac.res_tpl_data(false, true);
    let mut out_data = vec![];
    for tmp in tpl_data.into_iter() {
        out_data.push(json!({
            "res_type": tmp.key,
            "res_name": req_dao.fluent.format_message(&FluentMessage{
                id: format!("res-user-{}", tmp.key),
                crate_name: env!("CARGO_PKG_NAME").to_string(),
                data: vec![],
            })
        }));
    }
    Ok(JsonResponse::data(JsonData::body(
        json!({ "res_type": out_data }),
    )))
}

#[derive(Debug, Deserialize)]
pub struct DynamicResDataFromUserParam {
    pub user_any: Option<String>,
    pub limit: Option<LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

//获取一批用户的全局资源
pub async fn dynamic_res_data_global_user(
    param: &DynamicResDataFromUserParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let user_res_data = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .user_data(
            &UserDataParam {
                app_id: None,
                user_data: None,
                user_account: None,
                user_any: None,
            },
            &param.limit.to_u64_cursor_page_param(CursorPageSort::Desc),
        )
        .await?;
    let total = if param.count_num.unwrap_or_default() {
        Some(PageTotalRowValue::from(
            req_dao
                .web_dao
                .web_access
                .access_dao
                .user
                .user_count(
                    &UserDataParam {
                        app_id: Some(0),
                        user_data: None,
                        user_account: None,
                        user_any: param.user_any.as_deref(),
                    },
                    &TotalParam::default(),
                )
                .await?,
        ))
    } else {
        None
    };
    let res_data_key = user_res_data
        .0
        .iter()
        .map(|e| e.id.to_string())
        .collect::<Vec<String>>();
    let tpl_data = req_dao.web_dao.web_rbac.res_tpl_data(false, true);
    let res_data = req_dao
        .web_dao
        .web_rbac
        .res_tpl_sync(
            &tpl_data,
            0,
            0,
            "global-user",
            &res_data_key
                .iter()
                .map(|e| e.as_str())
                .collect::<Vec<&str>>(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;

    let mut out_data = vec![];
    for item in res_data {
        let user_data = user_res_data
            .0
            .iter()
            .find(|t| t.id.to_string().as_str() == item.res_data);
        out_data.push(json!({
            "res_type":item.res_type,
            "user_info": user_data,
            "user_data": user_data.map(|t|t.id).unwrap_or_default(),
            "res_id":item.res_id,
            "op_data": item.op_data.iter().map(|(op_key,op_id)| {
                json!({
                    "op_key": op_key,
                    "op_id": op_id,
                    "op_name": req_dao.fluent.format_message(&FluentMessage {
                        id: format!("res-op-admin-{}", op_key),
                        crate_name: env!("CARGO_PKG_NAME").to_string(),
                        data: vec![],
                    })
                })
            }).collect::<Vec<_>>(),
        }));
    }
    let cursor = PageCursorValue::from(&user_res_data.1);
    Ok(JsonResponse::data(JsonData::body(
        JsonPageData::cursor(Value::Null, cursor, total).set_extra("tpl_data", out_data),
    )))
}
