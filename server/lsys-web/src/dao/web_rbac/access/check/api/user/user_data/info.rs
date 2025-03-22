use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserInfoEdit {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserInfoEdit {
    async fn check(
        &self,
        access: &RbacAccess,
        check_env: &AccessCheckEnv<'_>,
    ) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::system_empty_data(
                    "global-public",
                    vec!["info-edit"],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserInfoEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,data:false,
            key: "global-public",
            ops: vec!["info-edit"],
        }]
    }
}
