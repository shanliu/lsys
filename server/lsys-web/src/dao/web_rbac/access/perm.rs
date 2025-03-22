use lsys_access::dao::SessionBody;
use serde::Serialize;
use serde_json::Value;

use crate::dao::access::api::system::{
    CheckAdminBase, CheckAdminChangeLogsView, CheckAdminDocs, CheckAdminMailConfig,
    CheckAdminSmsConfig,
};
use crate::dao::access::api::user::CheckAppSenderSmsConfig;
use crate::{common::JsonResult, dao::WebRbac};
use lsys_core::{fluent_message, RequestEnv};
use lsys_rbac::dao::{RbacError, RbacResult};

pub struct RbacAccessData<'t> {
    pub name: &'t str,
    pub data: Value,
}
impl WebRbac {
    pub(crate) async fn perm_check(
        &self,
        req_env: &RequestEnv,
        session_body_opt: Option<&SessionBody>,
        check_res: &RbacAccessData<'_>,
    ) -> RbacResult<()> {
        macro_rules! check {
            //$key 以接口为维度的关键字,外部传入
            //$data 该接口涉及的权限,参考具体结构的权限校验代码
            ($key:literal,$data:expr) => {
                if check_res.name == $key {
                    return self.check(req_env, session_body_opt, &$data).await;
                }
            };
        }
        check!("admin-sms-config", CheckAdminSmsConfig {});
        check!(
            "admin-sender-config",
            CheckAppSenderSmsConfig { res_user_id: 0 }
        );
        check!("admin-mail-config", CheckAdminMailConfig {});

        check!("admin-main", CheckAdminBase {});

        check!("admin-logs", CheckAdminChangeLogsView {});
        check!("docs-edit", CheckAdminDocs {});

        // let res_user_id = check_res.data.as_u64().unwrap_or(0);

        Err(RbacError::System(fluent_message!("rbac-unkown-res",{
            "res":&check_res.name
        })))
    }
}

pub struct RbacMenuParam<'t> {
    pub check_res: &'t [RbacAccessData<'t>],
}

#[derive(Debug, Serialize)]
pub struct RbacMenuStatus {
    pub status: bool, //是否授权成功
    pub name: String, //菜单名或key,参见:MenuItem.name
}

impl WebRbac {
    pub async fn perm_menu_check(
        &self,
        req_env: &RequestEnv,
        session_body_opt: Option<&SessionBody>,
        param: &RbacMenuParam<'_>,
    ) -> JsonResult<Vec<RbacMenuStatus>> {
        let mut out = Vec::with_capacity(param.check_res.len());
        for e in param.check_res.iter() {
            out.push(RbacMenuStatus {
                status: self
                    .perm_check(req_env, session_body_opt, e)
                    .await
                    .map(|_| true)
                    .unwrap_or(false),
                name: e.name.to_owned(),
            })
        }
        Ok(out)
    }
}
