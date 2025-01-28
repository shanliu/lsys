use crate::dao::{
    CheckRelationData, CheckResTpl, RbacCheckAccess, RbacCheckAccessDepend, RbacCheckResTpl,
};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckAdminSenderTplEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminSenderTplEdit {
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
                    "global-system",
                    vec!["tpl-edit"],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckAdminSenderTplView {
            res_user_id: self.res_user_id,
        })]
    }
}
impl RbacCheckResTpl for CheckAdminSenderTplEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![
            CheckResTpl {
                user: false,
                key: "global-system",
                ops: vec!["tpl-edit"],
            },
            CheckResTpl {
                user: true,
                key: "app-sender",
                ops: vec!["tpl-edit"],
            },
        ]
    }
}

pub struct CheckAdminSenderTplView {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminSenderTplView {
    async fn check(
        &self,
        access: &RbacAccess,
        check_env: &AccessCheckEnv<'_>,
        relation: &CheckRelationData,
    ) -> RbacResult<()> {
        if self.res_user_id == 0 {
            access
                .check(
                    check_env,
                    &relation.to_session_role(),
                    &[AccessCheckRes::system_empty_data(
                        "global-system",
                        vec!["tpl-view"],
                    )],
                )
                .await
        } else {
            access
                .list_check(
                    check_env,
                    &relation.to_session_role(),
                    &[
                        &[AccessCheckRes::system_empty_data(
                            "global-system",
                            vec!["tpl-view"],
                        )],
                        &[AccessCheckRes::user_empty_data(
                            self.res_user_id,
                            "app-sender",
                            vec!["tpl-view"],
                        )],
                    ],
                )
                .await
        }
    }
}
impl RbacCheckResTpl for CheckAdminSenderTplView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![
            CheckResTpl {
                user: false,
                key: "global-system",
                ops: vec!["tpl-view"],
            },
            CheckResTpl {
                user: true,
                key: "app-sender",
                ops: vec!["tpl-view"],
            },
        ]
    }
}
