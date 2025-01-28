//可用权限映射
use lsys_rbac::dao::AccessCheckEnv;
use serde::Serialize;
use serde_json::Value;

use crate::dao::access::api::system::{
    CheckAdminBase, CheckAdminChangeLogsView, CheckAdminDocs, CheckAdminMailConfig,
    CheckAdminSmsConfig,
};
use crate::dao::access::api::user::CheckAppSenderSmsConfig;
use crate::dao::CheckRelationRole;
use crate::{common::JsonResult, dao::WebRbac};
use lsys_core::fluent_message;
use lsys_rbac::dao::{RbacError, RbacResult};

pub struct RelationParam<'t> {
    pub role_key: &'t str,
    pub user_id: u64,
}

pub struct RbacAccessData<'t> {
    pub name: &'t str,
    pub data: Value,
    pub relation: Option<&'t [RelationParam<'t>]>,
}
impl WebRbac {
    pub(crate) async fn perm_check(
        &self,
        access_env: &AccessCheckEnv<'_>,
        check_res: &RbacAccessData<'_>,
    ) -> RbacResult<()> {
        let relation = match check_res.relation {
            Some(e) => e
                .iter()
                .map(|p| CheckRelationRole {
                    role_key: p.role_key.to_owned(),
                    user_id: p.user_id,
                })
                .collect::<Vec<CheckRelationRole>>(),
            None => vec![],
        };
        macro_rules! check {
            //$key 以接口为维度的关键字,外部传入
            //$data 该接口涉及的权限,参考具体结构的权限校验代码
            ($key:literal,$data:expr) => {
                if check_res.name == $key {
                    return self.check(access_env, &$data, Some(&relation.into())).await;
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
        access_env: &AccessCheckEnv<'_>,
        param: &RbacMenuParam<'_>,
    ) -> JsonResult<Vec<RbacMenuStatus>> {
        let mut out = Vec::with_capacity(param.check_res.len());
        for e in param.check_res.iter() {
            out.push(RbacMenuStatus {
                status: self
                    .perm_check(access_env, e)
                    .await
                    .map(|_| true)
                    .unwrap_or(false),
                name: e.name.to_owned(),
            })
        }
        Ok(out)
    }
}
