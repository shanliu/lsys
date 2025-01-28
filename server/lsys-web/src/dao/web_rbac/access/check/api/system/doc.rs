use crate::dao::{
    access::api::system::CheckAdminBase, CheckRelationData, CheckResTpl, RbacCheckAccess,
    RbacCheckAccessDepend, RbacCheckResTpl,
};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckAdminDocs {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminDocs {
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
                    vec!["docs"],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckAdminBase {})]
    }
}
impl RbacCheckResTpl for CheckAdminDocs {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-system",
            ops: vec!["edit-docs"],
        }]
    }
}
