use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{RbacAccess, RbacResult};

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
                &[&[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "global-user",
                    vec!["app-mail-config"],
                )]],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserAppSenderMailConfig {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            data: false,
            key: "global-user",
            ops: vec!["app-mail-config"],
        }]
    }
}

pub struct CheckUserAppSenderMailMsg {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAppSenderMailMsg {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[&[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "global-user",
                    vec!["app-mail-veiw"],
                )]],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserAppSenderMailMsg {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            data: false,
            key: "global-user",
            ops: vec!["app-mail-manage"],
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
                &[&[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "global-user",
                    vec!["app-mail-send"],
                )]],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserAppSenderMailSend {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            data: false,
            key: "global-user",
            ops: vec!["app-mail-send"],
        }]
    }
}
