use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::AccessCheckEnv;
use lsys_rbac::dao::AccessCheckOp;
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
                &[
                    &[AccessCheckRes::system_empty_data(
                        "global-user",
                        vec![AccessCheckOp::new(
                            "app-sms-config",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                    &[AccessCheckRes::system(
                        "global-user",
                        &self.res_user_id.to_string(),
                        vec![AccessCheckOp::new(
                            "app-sms-config",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                ],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserAppSenderSmsConfig {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![
            CheckResTpl {
                user: false,
                data: false,
                key: "global-user",
                ops: vec!["app-sms-config"],
            },
            CheckResTpl {
                user: false,
                data: true,
                key: "global-user",
                ops: vec!["app-sms-config"],
            },
        ]
    }
}

pub struct CheckUserAppSenderSmsView {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAppSenderSmsView {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[
                    &[AccessCheckRes::system_empty_data(
                        "global-user",
                        vec![AccessCheckOp::new(
                            "app-sms-view",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                    &[AccessCheckRes::system(
                        "global-user",
                        &self.res_user_id.to_string(),
                        vec![AccessCheckOp::new(
                            "app-sms-view",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                ],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserAppSenderSmsView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![
            CheckResTpl {
                user: false,
                data: false,
                key: "global-user",
                ops: vec!["app-sms-view"],
            },
            CheckResTpl {
                user: false,
                data: true,
                key: "global-user",
                ops: vec!["app-sms-view"],
            },
        ]
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
                &[&[AccessCheckRes::system_empty_data(
                    "sender-sms",
                    vec![AccessCheckOp::new(
                        "app-sms-send",
                        self.res_user_id != check_env.user_id,
                    )],
                )]],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserAppSenderSmsSend {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-user",
            ops: vec!["app-sms-send"],
        }]
    }
}
