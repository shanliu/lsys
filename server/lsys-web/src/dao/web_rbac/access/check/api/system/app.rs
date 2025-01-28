use crate::dao::{
    access::check::api::system::CheckAdminBase, CheckRelationData, CheckResTpl, RbacCheckAccess,
    RbacCheckAccessDepend, RbacCheckResTpl,
};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckAdminApp {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminApp {
    async fn check(
        &self,
        access: &RbacAccess,
        check_env: &AccessCheckEnv<'_>,
        relation: &CheckRelationData,
    ) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &relation.to_session_role(),
                &[AccessCheckRes::system_empty_data(
                    "global-system",
                    vec!["edit-app"],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckAdminBase {})]
    }
}

impl RbacCheckResTpl for CheckAdminApp {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-system",
            ops: vec!["edit-app"],
        }]
    }
}
