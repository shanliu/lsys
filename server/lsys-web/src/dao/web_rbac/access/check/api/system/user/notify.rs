use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserNotifyView {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserNotifyView {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[
                    &[AccessCheckRes::system_empty_data(
                        "global-user",
                        vec![AccessCheckOp::new(
                            "view-notify",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                    &[AccessCheckRes::system(
                        "global-user",
                        &self.res_user_id.to_string(),
                        vec![AccessCheckOp::new(
                            "view-notify",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                ],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserNotifyView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![
            CheckResTpl {
                user: false,
                data: false,
                key: "global-user",
                ops: vec!["view-notify"],
            },
            CheckResTpl {
                user: false,
                data: true,
                key: "global-user",
                ops: vec!["view-notify"],
            },
        ]
    }
}
