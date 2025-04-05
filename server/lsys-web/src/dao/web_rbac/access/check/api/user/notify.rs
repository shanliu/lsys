use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserNotifyView {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserNotifyView {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "global-user",
                    vec!["view-notify"],
                )],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserNotifyView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-user",
            ops: vec!["view-notify"],
        }]
    }
}
