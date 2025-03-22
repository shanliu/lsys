use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::AccessCheckEnv;
use lsys_rbac::dao::AccessCheckRes;
use lsys_rbac::dao::RbacAccess;
use lsys_rbac::dao::RbacResult;
pub struct CheckAppSenderSmsConfig {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAppSenderSmsConfig {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[
                    &[AccessCheckRes::system_empty_data(
                        "global-system",
                        vec!["app-sms-config"],
                    )],
                    &[AccessCheckRes::user_empty_data(
                        self.res_user_id,
                        "app-sender",
                        vec!["app-sms-config"],
                    )],
                ],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckAppSenderSmsConfig {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![
            CheckResTpl {
                user: false, data: false,
                key: "global-system",
                ops: vec!["app-sms-config"],
            },
            CheckResTpl {
                user: true,
                data: false,
                key: "app-sender",
                ops: vec!["app-sms-config"],
            },
        ]
    }
}

pub struct CheckAppSenderSmsMsg {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAppSenderSmsMsg {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[
                    &[AccessCheckRes::user_empty_data(
                        self.res_user_id,
                        "sender-sms",
                        vec!["app-sms-manage"],
                    )],
                    &[AccessCheckRes::system_empty_data(
                        "global-system",
                        vec!["app-sms-manage"],
                    )],
                ],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckAppSenderSmsMsg {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![
            CheckResTpl {
                user: false, data: false,
                key: "global-system",
                ops: vec!["app-sms-manage"],
            },
            CheckResTpl {
                user: true,
                data: false,
                key: "app-sender",
                ops: vec!["app-sms-manage"],
            },
        ]
    }
}

pub struct CheckAppSenderSmsSend {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAppSenderSmsSend {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[
                    &[AccessCheckRes::user_empty_data(
                        self.res_user_id,
                        "sender-sms",
                        vec!["app-sms-send"],
                    )],
                    &[AccessCheckRes::system_empty_data(
                        "global-system",
                        vec!["app-sms-send"],
                    )],
                ],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckAppSenderSmsSend {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![
            CheckResTpl {
                user: false, data: false,
                key: "global-system",
                ops: vec!["app-sms-send"],
            },
            CheckResTpl {
                user: true,
                data: false,
                key: "app-sender",
                ops: vec!["app-sms-send"],
            },
        ]
    }
}
