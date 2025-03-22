use crate::common::JsonData;
use crate::common::JsonError;
use crate::common::JsonResult;
use crate::common::LimitParam;
use crate::common::UserAuthQueryDao;
use crate::dao::access::api::user::CheckUserAppView;

use lsys_access::dao::AccessSession;
use lsys_core::fluent_message;
use lsys_rbac::dao::AuditDataParam;
use serde_json::{json, Value};

pub struct AppAuditResData {
    pub res_id: u64,
    pub op_id: Option<u64>,
}

pub struct AppAuditParam {
    pub user_data: Option<String>,
    pub app_id: u64,
    pub user_ip: Option<String>,
    pub device_id: Option<String>,
    pub request_id: Option<String>,
    pub res_data: Option<AppAuditResData>,
    pub limit: Option<LimitParam>,
    pub count_num: Option<bool>,
}

//查自身或子用户授权信息
pub async fn app_audit_data(
    param: &AppAuditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    if auth_data.session().user_app_id != 0 {
        return Err(JsonError::Message(fluent_message!("bad-audit-access")));
    }

    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppView {
                res_user_id: app.user_id,
            },
        )
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .inner_feature_sub_app_check(&app)
        .await?;

    let user_id = if let Some(user_data) = &param.user_data {
        //必须是子用户
        let audit_user = req_dao
            .web_dao
            .web_access
            .access_dao
            .user
            .cache()
            .sync_user(param.app_id, user_data, None, None)
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
                app_id: Some(param.app_id),
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
                    app_id: Some(param.app_id),
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
