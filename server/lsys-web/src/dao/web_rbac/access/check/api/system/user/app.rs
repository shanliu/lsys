use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckAccessDepend, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserAppView {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAppView {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::system_empty_data(
                    "global-user",
                    vec![AccessCheckOp::new(
                        "view-app",
                        self.res_user_id != check_env.user_id,
                    )],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserAppView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-user",
            ops: vec!["view-app"],
        }]
    }
}

pub struct CheckUserAppEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAppEdit {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::system_empty_data(
                    "global-user",
                    vec![AccessCheckOp::new(
                        "edit-app",
                        self.res_user_id != check_env.user_id,
                    )],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckUserAppView {
            res_user_id: self.res_user_id,
        })]
    }
}

impl RbacCheckResTpl for CheckUserAppEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-user",
            ops: vec!["edit-app"],
        }]
    }
}
