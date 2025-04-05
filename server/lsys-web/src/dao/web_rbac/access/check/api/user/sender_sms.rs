use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::AccessCheckEnv;
use lsys_rbac::dao::AccessCheckRes;
use lsys_rbac::dao::RbacAccess;
use lsys_rbac::dao::RbacResult;
pub struct CheckUserAppSenderSmsConfig {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAppSenderSmsConfig {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[&[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "global-user",
                    vec!["app-sms-config"],
                )]],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserAppSenderSmsConfig {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            data: false,
            key: "global-user",
            ops: vec!["app-sms-config"],
        }]
    }
}

pub struct CheckUserAppSenderSmsMsg {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAppSenderSmsMsg {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[&[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "global-user",
                    vec!["app-sms-manage"],
                )]],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserAppSenderSmsMsg {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            data: false,
            key: "global-user",
            ops: vec!["app-sms-manage"],
        }]
    }
}

pub struct CheckUserAppSenderSmsSend {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAppSenderSmsSend {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[&[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "sender-sms",
                    vec!["app-sms-send"],
                )]],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserAppSenderSmsSend {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            data: false,
            key: "global-user",
            ops: vec!["app-sms-send"],
        }]
    }
}
