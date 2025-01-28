use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

use crate::dao::{
    access::api::system::CheckAdminBase, CheckRelationData, CheckResTpl, RbacCheckAccess,
    RbacCheckAccessDepend, RbacCheckResTpl,
};
//这里定义访问权限验证
pub struct CheckAdminSiteSetting {}

#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminSiteSetting {
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
                &[AccessCheckRes::system_empty_data(
                    "global-system",      //资源KEY
                    vec!["site-setting"], //必须验证权限
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckAdminBase {})]
    }
}

impl RbacCheckResTpl for CheckAdminSiteSetting {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-system",
            ops: vec!["site-setting"],
        }]
    }
}
