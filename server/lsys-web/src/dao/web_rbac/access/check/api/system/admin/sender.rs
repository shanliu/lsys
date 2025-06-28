use crate::dao::{
    access::api::system::admin::CheckAdminBase, CheckResTpl, RbacCheckAccess,
    RbacCheckAccessDepend, RbacCheckResTpl,
};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckAdminSmsConfig {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminSmsConfig {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::system_empty_data(
                    "global-system",
                    vec![AccessCheckOp::new("app-sms-config", true)],
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
            data: false,
            key: "global-system",
            ops: vec!["app-sms-config"],
        }]
    }
}

pub struct CheckAdminSmsMgr {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminSmsMgr {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::system_empty_data(
                    "global-system",
                    vec![AccessCheckOp::new("app-sms-mgr", true)],
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
            data: false,
            key: "global-system",
            ops: vec!["app-sms-mgr"],
        }]
    }
}

pub struct CheckAdminMailConfig {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminMailConfig {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::system_empty_data(
                    "global-system",
                    vec![AccessCheckOp::new("app-mail-config", true)],
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
            data: false,
            key: "global-system",
            ops: vec!["app-mail-config"],
        }]
    }
}

pub struct CheckAdminMailMgr {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminMailMgr {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::system_empty_data(
                    "global-system",
                    vec![AccessCheckOp::new("app-mail-mgr", true)],
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
            data: false,
            key: "global-system",
            ops: vec!["app-mail-mgr"],
        }]
    }
}
