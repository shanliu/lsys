use crate::dao::{CheckRelationData, CheckResTpl, RbacCheckAccess, RbacCheckResTpl};

use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckAdminBase {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminBase {
    async fn check(
        &self,
        access: &RbacAccess,
        check_env: &AccessCheckEnv<'_>,
        relation: &CheckRelationData,
    ) -> RbacResult<()> {
        access
            .check(
                check_env,
                &relation.to_session_role(),
                &[AccessCheckRes::system_empty_data(
                    "global-system",
                    vec!["main"],
                )],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckAdminBase {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-system",
            ops: vec!["main"],
        }]
    }
}
