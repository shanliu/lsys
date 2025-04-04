//rest接口权限定义
use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckRestApp {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckRestApp {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::system_empty_data(
                    "global-app",
                    vec!["rest"],
                )],
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
