use crate::dao::{
    access::api::system::admin::CheckAdminBase, CheckResTpl, RbacCheckAccess,
    RbacCheckAccessDepend, RbacCheckResTpl,
};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckAdminDocs {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminDocs {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::system_empty_data(
                    "global-system",
                    vec![AccessCheckOp::new("view-docs", true)],
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
            data: false,
            key: "global-system",
            ops: vec!["edit-docs"],
        }]
    }
}
