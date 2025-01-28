use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

use crate::dao::{
    CheckRelationData, CheckResTpl, RbacCheckAccess, RbacCheckAccessDepend, RbacCheckResTpl,
};

pub struct CheckUserRbacView {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserRbacView {
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
                    "global-rbac-res",
                    vec!["edit"],
                )],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserRbacView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-rbac-res",
            ops: vec!["edit"],
        }]
    }
}
pub struct CheckUserRbacEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserRbacEdit {
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
                    "global-rbac-res",
                    vec!["edit"],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckUserRbacView {
            res_user_id: self.res_user_id,
        })]
    }
}
impl RbacCheckResTpl for CheckUserRbacEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-rbac-res",
            ops: vec!["edit"],
        }]
    }
}
