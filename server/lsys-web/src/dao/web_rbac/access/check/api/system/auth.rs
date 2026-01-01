use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};
pub struct CheckSystemLogin {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckSystemLogin {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::system_empty_data(
                    "global-public",
                    vec![AccessCheckOp::new("login", false)],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckSystemLogin {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-public",
            ops: vec!["login"],
        }]
    }
}

pub struct CheckSystemRegister {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckSystemRegister {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::system_empty_data(
                    "global-public",
                    vec![AccessCheckOp::new("register", false)],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckSystemRegister {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-public",
            ops: vec!["register"],
        }]
    }
}
