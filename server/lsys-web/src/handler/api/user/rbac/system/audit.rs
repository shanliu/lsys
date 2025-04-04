use crate::common::JsonData;
use crate::common::JsonResult;
use crate::common::LimitParam;
use crate::common::UserAuthQueryDao;

use lsys_access::dao::AccessSession;
use lsys_rbac::dao::AuditDataParam;
use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Debug, Deserialize)]
pub struct SystemAuditResParam {
    pub res_id: u64,
    pub op_id: Option<u64>,
}
#[derive(Debug, Deserialize)]
pub struct SystemAuditParam {
    pub user_ip: Option<String>,
    pub device_id: Option<String>,
    pub request_id: Option<String>,
    pub res_data: Option<SystemAuditResParam>,
    pub limit: Option<LimitParam>,
    pub count_num: Option<bool>,
}

//查自身授权信息
pub async fn system_audit_data(
    param: &SystemAuditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .audit_data(
            &AuditDataParam {
                user_id: Some(auth_data.user_id()),
                app_id: Some(0),
                user_ip: param.user_ip.as_deref(),
                device_id: param.device_id.as_deref(),
                request_id: param.request_id.as_deref(),
                res_data: param.res_data.as_ref().map(|e| (e.res_id, e.op_id)),
            },
            param.limit.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .access
                .audit_count(&AuditDataParam {
                    user_id: Some(auth_data.user_id()),
                    app_id: Some(0),
                    user_ip: param.user_ip.as_deref(),
                    device_id: param.device_id.as_deref(),
                    request_id: param.request_id.as_deref(),
                    res_data: param.res_data.as_ref().map(|e| (e.res_id, e.op_id)),
                })
                .await?,
        )
    } else {
        None
    };
    let out_data = res
        .0
        .iter()
        .map(|(a, b)| {
            json!({
                "audit":a,
                "detail":b
            })
        })
        .collect::<Vec<Value>>();
    Ok(JsonData::data(json!({
        "data": out_data,
        "next": res.1,
        "total": count,
    })))
}
