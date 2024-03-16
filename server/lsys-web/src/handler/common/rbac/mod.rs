mod access;
mod res;
mod role;

use crate::handler::access::{
    AccessAdminChangeLogsView, AccessAdminDocsEdit, AccessAdminMailConfig, AccessAdminManage,
    AccessAdminSetting, AccessAdminSmsConfig, AccessAdminUserFull, AccessAppSenderSmsConfig,
    AccessResView, AccessRoleView, AccessUserAppConfirm,
};
pub use access::*;
use lsys_core::fluent_message;
use lsys_rbac::dao::{RbacDao, RoleRelationKey, UserRbacError, UserRbacResult};
pub use res::*;
pub use role::*;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct RelationParam {
    pub role_key: String,
    pub user_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct RbacAccessParam {
    pub name: String,
    pub data: Value,
    pub relation: Option<Vec<RelationParam>>,
}

pub(crate) async fn access_check(
    rbac_dao: &RbacDao,
    user_id: u64,
    check_res: &RbacAccessParam,
) -> UserRbacResult<()> {
    let relation = match check_res.relation {
        Some(ref e) => e
            .iter()
            .map(|p| RoleRelationKey {
                relation_key: p.role_key.clone(),
                user_id: p.user_id,
            })
            .collect::<Vec<RoleRelationKey>>(),
        None => vec![],
    };
    macro_rules! check {
        //$key 以接口为维度的关键字,外部传入
        //$data 该接口涉及的权限,参考具体结构的权限校验代码
        ($key:literal,$data:expr) => {
            if check_res.name.as_str() == $key {
                return rbac_dao.rbac.check(&$data, Some(&relation)).await;
            }
        };
    }
    check!("admin-sms-config", AccessAdminSmsConfig { user_id });
    check!(
        "admin-sender-config",
        AccessAppSenderSmsConfig {
            user_id,
            app_id: 0,
            res_user_id: 0
        }
    );
    check!("admin-mail-config", AccessAdminMailConfig { user_id });

    check!("admin-app", AccessUserAppConfirm { user_id });
    check!("admin-main", AccessAdminManage { user_id });
    check!("admin-setting", AccessAdminSetting { user_id });
    check!("admin-user", AccessAdminUserFull { user_id });
    check!("admin-logs", AccessAdminChangeLogsView { user_id });
    check!("docs-edit", AccessAdminDocsEdit { user_id });

    let res_user_id = check_res.data.as_u64().unwrap_or(0);
    check!(
        "res-view",
        AccessResView {
            user_id,
            res_user_id
        }
    );
    check!(
        "role-view",
        AccessRoleView {
            user_id,
            res_user_id
        }
    );

    Err(UserRbacError::System(fluent_message!("rbac-unkown-res",{
        "res":&check_res.name
    })))
}
