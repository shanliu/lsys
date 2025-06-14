use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::LimitParam;
use crate::common::UserAuthQueryDao;

use lsys_rbac::dao::AuditDataParam;
use serde::Deserialize;
use serde_json::{json, Value};

use super::app_check_get;
use super::parent_app_check;

#[derive(Debug, Deserialize)]
pub struct AppAuditResParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub res_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub op_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct AppAuditParam {
    pub user_param: Option<String>, //用户标识过滤,不传不过滤
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub user_ip: Option<String>,
    pub device_id: Option<String>,
    pub request_id: Option<String>,
    pub res_data: Option<AppAuditResParam>,
    pub limit: Option<LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

//查自身或子用户授权信息
pub async fn app_audit_data(
    param: &AppAuditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = parent_app_check(req_dao).await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;
    let user_id = if let Some(user_data) = &param.user_param {
        //必须是子用户
        let audit_user = req_dao
            .web_dao
            .web_access
            .access_dao
            .user
            .cache()
            .sync_user(app.id, user_data, None, None)
            .await?;
        Some(audit_user.id)
    } else {
        None
    };
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .audit_data(
            &AuditDataParam {
                user_id,
                user_app_id: Some(app.id),
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
                    user_id,
                    user_app_id: Some(app.id),
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
