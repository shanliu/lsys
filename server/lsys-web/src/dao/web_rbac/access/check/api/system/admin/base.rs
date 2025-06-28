use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};

use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckAdminBase {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminBase {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::system_empty_data(
                    "global-system",
                    vec![AccessCheckOp::new("main", true)],
                )],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckAdminBase {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-system",
            ops: vec!["main"],
        }]
    }
}
