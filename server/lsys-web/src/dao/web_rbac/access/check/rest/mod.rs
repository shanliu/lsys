//rest接口权限定义
use crate::dao::{CheckRelationData, CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckRestApp {
    pub app_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckRestApp {
    async fn check(
        &self,
        access: &RbacAccess,
        check_env: &AccessCheckEnv<'_>,
        relation: &CheckRelationData,
    ) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &relation.to_session_role(),
                &[AccessCheckRes::system(
                    "rest-app",
                    &self.app_id.to_string(),
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
            key: "global-app",
            ops: vec!["rest"],
        }]
    }
}
