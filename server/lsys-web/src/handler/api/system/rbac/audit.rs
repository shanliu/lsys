use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::LimitParam;
use crate::common::UserAuthQueryDao;
use crate::dao::access::api::system::CheckAdminRbacView;
use lsys_access::dao::AccessSession;
use lsys_rbac::dao::AuditDataParam;
use serde::Deserialize;
use serde_json::{json, Value};

//查看用户访问授权日志数据

#[derive(Debug, Deserialize)]
pub struct AuditResParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub res_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub op_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct AuditParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub user_id: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub user_app_id: Option<u64>,
    pub user_ip: Option<String>,
    pub device_id: Option<String>,
    pub request_id: Option<String>,
    pub res_data: Option<AuditResParam>,
    pub limit: Option<LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn audit_data(
    param: &AuditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminRbacView {})
        .await?;
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .audit_data(
            &AuditDataParam {
                user_id: param.user_id,
                user_app_id: param.user_app_id,
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
                    user_id: param.user_id,
                    user_app_id: param.user_app_id,
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
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": out_data,
        "next": res.1,
        "total": count,
    }))))
}
