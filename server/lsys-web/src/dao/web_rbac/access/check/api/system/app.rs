use crate::dao::{
    access::check::api::system::CheckAdminBase, CheckResTpl, RbacCheckAccess,
    RbacCheckAccessDepend, RbacCheckResTpl,
};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckAdminApp {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminApp {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::system_empty_data(
                    "global-system",
                    vec!["view-app"],
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
            data: false,
            key: "global-system",
            ops: vec!["edit-app"],
        }]
    }
}
