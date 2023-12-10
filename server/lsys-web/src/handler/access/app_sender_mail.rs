use lsys_app::model::AppsModel;
use lsys_rbac::dao::{
    AccessRes, RbacAccess, RbacCheck, RbacCheckDepend, RbacRelationTpl, RbacResTpl, ResTpl,
    RoleRelationKey, UserRbacResult,
};

use crate::handler::access::AccessAdminManage;

use super::RelationApp;

pub struct AccessAdminMailConfig {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAdminMailConfig {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[AccessRes::system(
                    "global-system",
                    &["app-smtp-config"],
                    &[],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessAdminManage {
            user_id: self.user_id,
        })]
    }
}
impl RbacResTpl for AccessAdminMailConfig {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["app", "mail"],
            user: false,
            key: "global-system",
            ops: vec!["app-smtp-config"],
        }]
    }
}

pub struct AccessAppSenderMailConfig {
    pub app_id: u64,
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAppSenderMailConfig {
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
                    &[AccessRes::system(
                        "global-system",
                        &["app-mail-config"],
                        &[],
                    )],
                )
                .await
        } else {
            access
                .list_check(
                    self.user_id,
                    relation,
                    &[
                        vec![AccessRes::system(
                            "global-system",
                            &["app-mail-config"],
                            &[],
                        )],
                        vec![AccessRes::user(
                            self.res_user_id,
                            "app-sender",
                            &["app-mail-config"],
                            &[],
                        )],
                    ],
                )
                .await
        }
    }
}
impl RbacResTpl for AccessAppSenderMailConfig {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["mail", "app", "system"],
                user: false,
                key: "global-system",
                ops: vec!["app-mail-config"],
            },
            ResTpl {
                tags: vec!["mail", "app"],
                user: true,
                key: "app-sender",
                ops: vec!["app-mail-config"],
            },
        ]
    }
}

pub struct AccessAppSenderMailMsg {
    pub user_id: u64,
    pub res_user_id: u64,
    pub app_id: Option<u64>,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAppSenderMailMsg {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        let res_user_id = self.res_user_id;
        if res_user_id == 0 {
            //系统短信查看
            access
                .check(
                    self.user_id,
                    relation,
                    &[AccessRes::system(
                        "global-system",
                        &["app-mail-manage"],
                        &[],
                    )],
                )
                .await
        } else {
            access
                .list_check(
                    self.user_id,
                    relation,
                    &[
                        vec![AccessRes::user(
                            res_user_id,
                            "app-sender",
                            &["app-mail-veiw"],
                            &[],
                        )],
                        vec![AccessRes::system(
                            "global-system",
                            &["app-mail-manage"],
                            &[],
                        )],
                    ],
                )
                .await
        }
    }
}
impl RbacResTpl for AccessAppSenderMailMsg {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["mail", "app", "system"],
                user: false,
                key: "global-system",
                ops: vec!["app-mail-manage"],
            },
            ResTpl {
                tags: vec!["mail", "app"],
                user: true,
                key: "app-sender",
                ops: vec!["app-mail-manage"],
            },
        ]
    }
}

pub struct AccessAppSenderDoMail {
    pub app: AppsModel,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAppSenderDoMail {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.app.user_id,
                &RelationApp { app: &self.app }.extend(relation),
                &[AccessRes::system(
                    &format!("global-app-{}", self.app.id),
                    &["app-mail-send"],
                    &[],
                )],
            )
            .await
    }
}
impl RbacResTpl for AccessAppSenderDoMail {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["app", "mail"],
            user: false,
            key: "global-app-{appid}",
            ops: vec!["app-mail-send"],
        }]
    }
}
