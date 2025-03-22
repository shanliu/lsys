use crate::dao::{
    access::api::system::CheckAdminBase, CheckResTpl, RbacCheckAccess, RbacCheckAccessDepend,
    RbacCheckResTpl,
};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckAdminUserManage {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminUserManage {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::system_empty_data(
                    "global-system",
                    vec!["manage-user"],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckAdminBase {})]
    }
}

impl RbacCheckResTpl for CheckAdminUserManage {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data:false,
            key: "global-system",
            ops: vec!["manage-user"],
        }]
    }
}

pub struct CheckAdminChangeLogsView {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminChangeLogsView {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::system_empty_data(
                    "global-system",
                    vec!["see-change-log"],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckAdminBase {})]
    }
}
impl RbacCheckResTpl for CheckAdminChangeLogsView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data:false,
            key: "global-system",
            ops: vec!["see-change-log"],
        }]
    }
}
