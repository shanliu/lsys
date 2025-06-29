//rest接口权限定义
use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckRestApp {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckRestApp {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env, //资源访问用户
                &[
                    &[AccessCheckRes::system(
                        "global-app",
                        &self.res_user_id.to_string(),
                        vec![AccessCheckOp::new(
                            "rest",
                            check_env.user_id != self.res_user_id,
                        )],
                    )],
                    &[AccessCheckRes::system_empty_data(
                        "global-app",
                        vec![AccessCheckOp::new(
                            "rest",
                            check_env.user_id != self.res_user_id,
                        )],
                    )],
                ],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckRestApp {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-app",
            ops: vec!["rest"],
        }]
    }
}
