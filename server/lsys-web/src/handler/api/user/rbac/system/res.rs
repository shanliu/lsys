use crate::{
    common::{JsonData, JsonResult, PageParam, UserAuthQueryDao},
    dao::user::RbacUserSyncOpParam,
};
use lsys_access::dao::AccessSession;
use lsys_core::FluentMessage;
use lsys_rbac::dao::ResTypeParam;
use serde::Deserialize;
use serde_json::json;

pub async fn system_res_global_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let tpl_data = req_dao.web_dao.web_rbac.res_tpl_data(true, false);
    let mut out_data = vec![];
    for tmp in tpl_data.into_iter() {
        let key_data = tmp
            .ops
            .iter()
            .map(|e| RbacUserSyncOpParam {
                op_key: e,
                init_op_name: None,
            })
            .collect::<Vec<_>>();
        let key_data = key_data.iter().collect::<Vec<_>>();
        let tpl_data = req_dao
            .web_dao
            .web_rbac
            .user_sync_res_type_op_id(
                &ResTypeParam {
                    res_type: tmp.key,
                    user_id: auth_data.user_id(),
                    app_id: 0,
                },
                &key_data,
                auth_data.user_id(),
                Some(&req_dao.req_env),
            )
            .await?;
        out_data.push(json!({
            "res_type": tmp.key,
            "res_name": req_dao.fluent.format_message(&FluentMessage {
                id: format!("res-user-{}", tmp.key),
                crate_name: env!("CARGO_PKG_NAME").to_string(),
                data: vec![],
            }),
            "op_data": tpl_data.iter().map(|(e,op_id)| {
                json!({
                    "op_key": e.op_key,
                    "op_id": op_id,
                    "op_name":req_dao.fluent.format_message(&FluentMessage {
                        id: format!("res-op-user-{}", e.op_key),
                        crate_name: env!("CARGO_PKG_NAME").to_string(),
                        data: vec![],
                    })
                })
            }).collect::<Vec<_>>(),
        }));
    }
    Ok(JsonData::data(json!({ "tpl_data": out_data })))
}

#[derive(Debug, Deserialize)]
pub struct SystemResParam {
    pub res_type: String,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn system_res_data(
    param: &SystemResParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let res_data = vec!["111"]; //@todo 资源标识列表
    let res_data = req_dao
        .web_dao
        .web_rbac
        .res_tpl_user_sync(
            auth_data.user_id(),
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
    Ok(JsonData::data(json!({ "tpl_data": out_data })))
}
