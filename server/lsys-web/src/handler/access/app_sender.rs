use lsys_rbac::dao::{
    AccessRes, RbacAccess, RbacCheck, RbacCheckDepend, RbacResTpl, ResTpl, RoleRelationKey,
    UserRbacResult,
};

use crate::handler::access::AccessAdminManage;

pub struct AccessAdminSenderTplEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAdminSenderTplEdit {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        if self.res_user_id == 0 {
            access
                .check(
                    self.user_id,
                    relation,
                    &[AccessRes::system("global-system", &["tpl-edit"], &[])],
                )
                .await
        } else {
            access
                .list_check(
                    self.user_id,
                    relation,
                    &[
                        vec![AccessRes::system("global-system", &["tpl-edit"], &[])],
                        vec![AccessRes::user(
                            self.res_user_id,
                            "app-sender",
                            &["tpl-edit"],
                            &[],
                        )],
                    ],
                )
                .await
        }
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![
            Box::new(AccessAdminManage {
                user_id: self.user_id,
            }),
            Box::new(AccessAdminSenderTplView {
                user_id: self.user_id,
                res_user_id: self.res_user_id,
            }),
        ]
    }
}
impl RbacResTpl for AccessAdminSenderTplEdit {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["app", "tpl"],
                user: false,
                key: "global-system",
                ops: vec!["tpl-edit"],
            },
            ResTpl {
                tags: vec!["app", "tpl"],
                user: true,
                key: "app-sender",
                ops: vec!["tpl-edit"],
            },
        ]
    }
}

pub struct AccessAdminSenderTplView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAdminSenderTplView {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        if self.res_user_id == 0 {
            access
                .check(
                    self.user_id,
                    relation,
                    &[AccessRes::system("global-system", &["tpl-view"], &[])],
                )
                .await
        } else {
            access
                .list_check(
                    self.user_id,
                    relation,
                    &[
                        vec![AccessRes::system("global-system", &["tpl-view"], &[])],
                        vec![AccessRes::user(
                            self.res_user_id,
                            "app-sender",
                            &["tpl-view"],
                            &[],
                        )],
                    ],
                )
                .await
        }
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessAdminManage {
            user_id: self.user_id,
        })]
    }
}
impl RbacResTpl for AccessAdminSenderTplView {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["app", "tpl"],
                user: false,
                key: "global-system",
                ops: vec!["tpl-view"],
            },
            ResTpl {
                tags: vec!["app", "tpl"],
                user: true,
                key: "app-sender",
                ops: vec!["tpl-view"],
            },
        ]
    }
}
