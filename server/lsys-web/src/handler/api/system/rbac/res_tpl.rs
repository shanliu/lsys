use crate::common::{JsonData, JsonResponse, JsonResult, PageParam, UserAuthQueryDao};
use lsys_access::dao::AccessSession;
use lsys_core::FluentMessage;
use serde::Deserialize;
use serde_json::json;

//静态资源类型数据

pub async fn global_res_tpl(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
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

pub async fn data_res_type(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
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
pub struct DataResFromResTypeParam {
    pub res_type: String,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn data_res_from_res_type(
    param: &DataResFromResTypeParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let res_data = vec!["111"]; //@todo 资源标识列表,从某个资源中查询出来
    let res_data = req_dao
        .web_dao
        .web_rbac
        .res_tpl_sync(
            0,
            0,
            &param.res_type,
            &res_data,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;

    let mut out_data = vec![];
    for item in res_data {
        out_data.push(json!({
            "res_type":item.res_type,
            "res_data":item.res_data,
            "res_id":item.res_id,
            "op_data": item.op_data.iter().map(|(op_key,op_id)| {
                json!({
                    "op_key": op_key,
                    "op_id": op_id,
                })
            }).collect::<Vec<_>>(),
        }));
    }
    Ok(JsonResponse::data(JsonData::body(
        json!({ "tpl_data": out_data }),
    )))
}
