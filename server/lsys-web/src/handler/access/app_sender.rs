use lsys_app::model::AppsModel;
use lsys_rbac::dao::{AccessRes, RbacAccess, RbacCheck, RbacCheckDepend, UserRbacResult};

use crate::handler::access::AccessAdminManage;

use super::app_relation_key;

pub struct AccessAdminAliSmsConfig {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAdminAliSmsConfig {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                &[],
                &[AccessRes::system("admin", &["alisms-config"], &[])],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessAdminManage {
            user_id: self.user_id,
        })]
    }
}
//     Some(vec![Self::AdminManage]),
// )),

pub struct AccessAppSender {
    pub app_id: u64,
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAppSender {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    vec![AccessRes::system(
                        &format!("app-{}", self.app_id),
                        &["global-app-sender-config"],
                        &[],
                    )],
                    vec![AccessRes::user(
                        self.res_user_id,
                        &format!("app-{}", self.app_id),
                        &["app-sender-config"],
                        &[],
                    )],
                ],
            )
            .await
    }
}

pub struct AccessAppSenderSms {
    pub user_id: u64,
    pub res_user_id: Option<u64>,
    pub app_id: Option<u64>,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAppSenderSms {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                &[],
                &[
                    // AccessRes::(["app", access_op!(["send-sms", true])])
                ],
            )
            .await
    }
}

pub struct AccessAppSenderDoSms {
    pub app: AppsModel,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAppSenderDoSms {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.app.user_id,
                &app_relation_key(&self.app),
                &[AccessRes::system("app", &["send-sms"], &[])],
            )
            .await
    }
}
