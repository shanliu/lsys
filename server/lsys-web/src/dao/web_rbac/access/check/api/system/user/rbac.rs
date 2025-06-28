use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};

use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckAccessDepend, RbacCheckResTpl};

pub struct CheckUserRbacView {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserRbacView {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::system_empty_data(
                    "global-user",
                    vec![AccessCheckOp::new(
                        "rbac-view",
                        self.res_user_id != check_env.user_id,
                    )],
                )],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserRbacView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-user",
            ops: vec!["rbac-view"],
        }]
    }
}
pub struct CheckUserRbacEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserRbacEdit {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::system_empty_data(
                    "global-user",
                    vec![AccessCheckOp::new(
                        "rbac-edit",
                        self.res_user_id != check_env.user_id,
                    )],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckUserRbacView {
            res_user_id: self.res_user_id,
        })]
    }
}
impl RbacCheckResTpl for CheckUserRbacEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-user",
            ops: vec!["rbac-edit"],
        }]
    }
}
