use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{RbacAccess, RbacResult};

use lsys_rbac::dao::AccessCheckEnv;
use lsys_rbac::dao::AccessCheckRes;

pub struct CheckAppSenderMailConfig {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAppSenderMailConfig {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[
                    &[AccessCheckRes::system_empty_data(
                        "global-system",
                        vec!["app-mail-config"],
                    )],
                    &[AccessCheckRes::user_empty_data(
                        self.res_user_id,
                        "app-sender",
                        vec!["app-mail-config"],
                    )],
                ],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckAppSenderMailConfig {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![
            CheckResTpl {
                user: false,
                data: false,
                key: "global-system",
                ops: vec!["app-mail-config"],
            },
            CheckResTpl {
                user: true,
                data: false,
                key: "app-sender",
                ops: vec!["app-mail-config"],
            },
        ]
    }
}

pub struct CheckAppSenderMailMsg {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAppSenderMailMsg {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[
                    &[AccessCheckRes::user_empty_data(
                        self.res_user_id,
                        "app-sender",
                        vec!["app-mail-veiw"],
                    )],
                    &[AccessCheckRes::system_empty_data(
                        "global-system",
                        vec!["app-mail-manage"],
                    )],
                ],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckAppSenderMailMsg {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![
            CheckResTpl {
                user: false,
                data: false,
                key: "global-system",
                ops: vec!["app-mail-manage"],
            },
            CheckResTpl {
                user: true,
                data: false,
                key: "app-sender",
                ops: vec!["app-mail-manage"],
            },
        ]
    }
}

pub struct CheckAppSenderMailSend {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAppSenderMailSend {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[
                    &[AccessCheckRes::user_empty_data(
                        self.res_user_id,
                        "app-sender",
                        vec!["app-mail-send"],
                    )],
                    &[AccessCheckRes::system_empty_data(
                        "global-system",
                        vec!["app-mail-send"],
                    )],
                ],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckAppSenderMailSend {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![
            CheckResTpl {
                user: false,
                data: false,
                key: "global-system",
                ops: vec!["app-mail-send"],
            },
            CheckResTpl {
                user: true,
                data: false,
                key: "app-sender",
                ops: vec!["app-mail-send"],
            },
        ]
    }
}
