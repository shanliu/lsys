use crate::dao::{CheckRelationData, CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};
pub struct CheckSystemLogin {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckSystemLogin {
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
                    vec!["login"],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckSystemLogin {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-public",
            ops: vec!["login"],
        }]
    }
}

pub struct CheckSystemRegister {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckSystemRegister {
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
                    vec!["register"],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckSystemRegister {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-public",
            ops: vec!["register"],
        }]
    }
}
