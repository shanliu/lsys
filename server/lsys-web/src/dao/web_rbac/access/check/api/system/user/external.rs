use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserExternalEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserExternalEdit {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env, //资源访问用户
                &[
                    &[AccessCheckRes::system_empty_data(
                        "global-user",
                        vec![AccessCheckOp::new("external-edit", false)],
                    )],
                    &[AccessCheckRes::system(
                        "global-user",
                        &self.res_user_id.to_string(),
                        vec![AccessCheckOp::new(
                            "external-edit",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                ],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserExternalEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![
            CheckResTpl {
                user: false,
                data: false,
                key: "global-user",
                ops: vec!["external-edit"],
            },
            CheckResTpl {
                user: false,
                data: true,
                key: "global-user",
                ops: vec!["external-edit"],
            },
        ]
    }
}
