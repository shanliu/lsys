use crate::dao::{CheckRelationData, CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserExternalEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserExternalEdit {
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
                &[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "global-public",
                    vec!["external-edit"],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserExternalEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            key: "global-public",
            ops: vec!["external-edit"],
        }]
    }
}
