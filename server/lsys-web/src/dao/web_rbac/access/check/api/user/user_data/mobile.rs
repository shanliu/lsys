use crate::dao::{CheckRelationData, CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserMobileBase {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserMobileBase {
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
                    vec!["mobile-base"],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserMobileBase {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-public",
            ops: vec!["mobile-base"],
        }]
    }
}

pub struct CheckUserMobileEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserMobileEdit {
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
                    vec!["mobile-edit"],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserMobileEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            key: "global-public",
            ops: vec!["mobile-edit"],
        }]
    }
}
