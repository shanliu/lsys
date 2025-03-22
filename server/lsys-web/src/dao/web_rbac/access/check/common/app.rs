use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

use crate::dao::RbacCheckAccess;
//这里定义访问权限验证
pub struct CheckApp {}

#[async_trait::async_trait]
impl RbacCheckAccess for CheckApp {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::system_empty_data(
                    "global-system", //资源KEY
                    vec!["app"],     //必须验证权限
                )],
            )
            .await
    }
}
