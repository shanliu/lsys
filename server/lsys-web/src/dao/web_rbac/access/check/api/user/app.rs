use crate::dao::{
    CheckRelationData, CheckResTpl, RbacCheckAccess, RbacCheckAccessDepend, RbacCheckResTpl,
};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserAppView {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAppView {
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
                    "user-app",
                    vec!["view-app"],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserAppView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-user",
            ops: vec!["view-app"],
        }]
    }
}

pub struct CheckUserAppEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAppEdit {
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
                    "user-app",
                    vec!["edit-app"],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckUserAppView {
            res_user_id: self.res_user_id,
        })]
    }
}

impl RbacCheckResTpl for CheckUserAppEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-user",
            ops: vec!["edit-app"],
        }]
    }
}
