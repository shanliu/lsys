use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckOp, RbacAccess, RbacResult};

use lsys_rbac::dao::AccessCheckEnv;
use lsys_rbac::dao::AccessCheckRes;

pub struct CheckUserAppSenderMailConfig {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAppSenderMailConfig {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[&[AccessCheckRes::system_empty_data(
                    "global-user",
                    vec![AccessCheckOp::new(
                        "app-mail-config",
                        self.res_user_id != check_env.user_id,
                    )],
                )]],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserAppSenderMailConfig {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-user",
            ops: vec!["app-mail-config"],
        }]
    }
}

pub struct CheckUserAppSenderMailView {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAppSenderMailView {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[&[AccessCheckRes::system_empty_data(
                    "global-user",
                    vec![AccessCheckOp::new(
                        "app-mail-veiw",
                        self.res_user_id != check_env.user_id,
                    )],
                )]],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserAppSenderMailView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-user",
            ops: vec!["app-mail-view"],
        }]
    }
}

pub struct CheckUserAppSenderMailSend {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAppSenderMailSend {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[&[AccessCheckRes::system_empty_data(
                    "global-user",
                    vec![AccessCheckOp::new(
                        "app-mail-send",
                        self.res_user_id != check_env.user_id,
                    )],
                )]],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserAppSenderMailSend {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-user",
            ops: vec!["app-mail-send"],
        }]
    }
}
