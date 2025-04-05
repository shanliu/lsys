use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserExternalEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserExternalEdit {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "global-user",
                    vec!["external-edit"],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserExternalEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            data: false,
            key: "global-user",
            ops: vec!["external-edit"],
        }]
    }
}
