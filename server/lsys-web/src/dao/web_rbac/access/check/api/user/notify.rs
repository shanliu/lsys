use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckNotifyView {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckNotifyView {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        // let mut check_copy = check_env.to_owned();
        // check_copy
        //     .session_role
        //     .push(crate::dao::access::AccessSessionRole {
        //         role_key: "global-notify",
        //         user_id: self.res_user_id,
        //     });
        access
            .check(
                check_env,
                &[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "global-notify",
                    vec!["view"],
                )],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckNotifyView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-notify",
            ops: vec!["view"],
        }]
    }
}
