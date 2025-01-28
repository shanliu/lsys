mod res;
mod user;
use crate::common::JsonError;
use crate::common::JsonResult;
use crate::common::LimitParam;
use crate::common::UserAuthQueryDao;
use crate::dao::WebRbac;
use lsys_access::dao::AccessSession;
use lsys_core::fluent_message;
use lsys_rbac::dao::AuditDataParam;
use lsys_rbac::model::RbacAuditDetailModel;
use lsys_rbac::model::RbacAuditModel;
pub use res::*;
pub use user::*;
pub struct RbacAuditResData {
    pub res_id: u64,
    pub op_id: Option<u64>,
}

pub struct RbacAuditParam {
    pub user_id: Option<u64>,
    pub user_ip: Option<String>,
    pub device_id: Option<String>,
    pub request_id: Option<String>,
    pub res_data: Option<RbacAuditResData>,
    pub limit: Option<LimitParam>,
    pub count_num: Option<bool>,
}

impl WebRbac {
    pub async fn audit_data(
        &self,
        param: &RbacAuditParam,
        req_dao: &UserAuthQueryDao,
    ) -> JsonResult<(
        Vec<(RbacAuditModel, Vec<RbacAuditDetailModel>)>,
        Option<u64>,
        Option<i64>,
    )> {
        let auth_data = req_dao.user_session.read().await.get_session_data().await?;
        if auth_data.session().user_app_id != 0 {
            return Err(JsonError::Message(fluent_message!("bad-audit-access")));
        }
        if let Some(uid) = &param.user_id {
            let audit_user = req_dao
                .web_dao
                .web_access
                .access_dao
                .user
                .find_by_id(uid)
                .await?;
            let app = req_dao
                .web_dao
                .web_app
                .app_dao
                .app
                .find_by_id(&audit_user.app_id)
                .await?;
            if app.user_id != auth_data.user_id() {
                return Err(JsonError::Message(fluent_message!("bad-audit-access")));
            }
        }
        let res = self
            .rbac_dao
            .access
            .audit_data(
                &AuditDataParam {
                    user_id: param.user_id,
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
                self.rbac_dao
                    .access
                    .audit_count(&AuditDataParam {
                        user_id: param.user_id,
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
        Ok((res.0, res.1, count))
    }
}
