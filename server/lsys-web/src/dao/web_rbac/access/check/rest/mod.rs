//rest接口权限定义
use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckRestApp {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckRestApp {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                //所有接口默认授权访问
                //屏蔽某应用直接关对应应用权限
                //如果需要关整站接口，rbac添加禁止授权
                &[&[AccessCheckRes::system_empty_data(
                    "global-app",
                    vec![AccessCheckOp::new("rest", false)],
                )]],
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
