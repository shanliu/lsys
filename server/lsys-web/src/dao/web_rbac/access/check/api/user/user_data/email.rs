use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserEmailBase {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserEmailBase {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::system_empty_data(
                    "global-public",
                    vec!["email-base"],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserEmailBase {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-public",
            ops: vec!["email-base"],
        }]
    }
}

pub struct CheckUserEmailEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserEmailEdit {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "global-public",
                    vec!["email-edit"],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserEmailEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            data: false,
            key: "global-public",
            ops: vec!["email-edit"],
        }]
    }
}
