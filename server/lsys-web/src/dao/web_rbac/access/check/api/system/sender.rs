use crate::dao::{
    access::api::system::CheckAdminBase, CheckRelationData, CheckResTpl, RbacCheckAccess,
    RbacCheckAccessDepend, RbacCheckResTpl,
};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckAdminSmsConfig {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminSmsConfig {
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
                    vec!["app-sms-config"],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckAdminBase {})]
    }
}
impl RbacCheckResTpl for CheckAdminSmsConfig {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-system",
            ops: vec!["app-sms-config"],
        }]
    }
}

pub struct CheckAdminSmsMgr {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminSmsMgr {
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
                    vec!["app-sms-cancel"],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckAdminBase {})]
    }
}
impl RbacCheckResTpl for CheckAdminSmsMgr {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-system",
            ops: vec!["app-sms-cancel"],
        }]
    }
}

pub struct CheckAdminMailConfig {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminMailConfig {
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
                    vec!["app-mail-config"],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckAdminBase {})]
    }
}
impl RbacCheckResTpl for CheckAdminMailConfig {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-system",
            ops: vec!["app-mail-config"],
        }]
    }
}

pub struct CheckAdminMailMgr {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminMailMgr {
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
                    vec!["app-mail-cancel"],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckAdminBase {})]
    }
}
impl RbacCheckResTpl for CheckAdminMailMgr {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-system",
            ops: vec!["app-mail-cancel"],
        }]
    }
}
