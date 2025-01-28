use crate::dao::{CheckRelationData, CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserInfoEdit {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserInfoEdit {
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
            user: true,
            key: "global-public",
            ops: vec!["info-edit"],
        }]
    }
}
