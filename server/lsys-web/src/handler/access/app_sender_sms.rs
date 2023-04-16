use lsys_app::model::AppsModel;
use lsys_rbac::dao::{
    AccessRes, RbacAccess, RbacCheck, RbacCheckDepend, RbacRelationTpl, RbacResTpl, ResTpl,
    RoleRelationKey, UserRbacResult,
};

use crate::handler::access::AccessAdminManage;

use super::RelationApp;

pub struct AccessAdminAliSmsConfig {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAdminAliSmsConfig {
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
                    &["app-alisms-config"],
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
impl RbacResTpl for AccessAdminAliSmsConfig {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["app", "sms"],
            user: false,
            key: "global-system",
            ops: vec!["app-alisms-config"],
        }]
    }
}

pub struct AccessAppSenderSmsConfig {
    pub app_id: u64,
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAppSenderSmsConfig {
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
                    &[AccessRes::system("global-system", &["app-sms-config"], &[])],
                )
                .await
        } else {
            access
                .list_check(
                    self.user_id,
                    relation,
                    &[
                        vec![AccessRes::system("global-system", &["app-sms-config"], &[])],
                        vec![AccessRes::user(
                            self.res_user_id,
                            "app-sender",
                            &["app-sms-config"],
                            &[],
                        )],
                    ],
                )
                .await
        }
    }
}
impl RbacResTpl for AccessAppSenderSmsConfig {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["sms", "app", "system"],
                user: false,
                key: "global-system",
                ops: vec!["app-sms-config"],
            },
            ResTpl {
                tags: vec!["sms", "app"],
                user: true,
                key: "app-sender",
                ops: vec!["app-sms-config"],
            },
        ]
    }
}

pub struct AccessAppSenderSmsMsg {
    pub user_id: u64,
    pub res_user_id: u64,
    pub app_id: Option<u64>,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAppSenderSmsMsg {
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
                    &[AccessRes::system("global-system", &["app-sms-manage"], &[])],
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
                            "sender-sms",
                            &["app-sms-manage"],
                            &[],
                        )],
                        vec![AccessRes::system("global-system", &["app-sms-manage"], &[])],
                    ],
                )
                .await
        }
    }
}
impl RbacResTpl for AccessAppSenderSmsMsg {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["sms", "app", "system"],
                user: false,
                key: "global-system",
                ops: vec!["app-sms-manage"],
            },
            ResTpl {
                tags: vec!["sms", "app"],
                user: true,
                key: "sender-sms",
                ops: vec!["app-sms-manage"],
            },
        ]
    }
}

pub struct AccessAppSenderDoSms {
    pub app: AppsModel,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAppSenderDoSms {
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
                    &["app-sms-send"],
                    &[],
                )],
            )
            .await
    }
}
impl RbacResTpl for AccessAppSenderDoSms {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["app", "sms"],
            user: false,
            key: "global-app-{appid}",
            ops: vec!["app-sms-send"],
        }]
    }
}
