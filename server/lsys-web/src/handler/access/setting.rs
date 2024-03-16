use lsys_rbac::dao::{
    AccessRes, RbacAccess, RbacCheck, RbacResTpl, ResTpl, RoleRelationKey, UserRbacResult,
};

//这里定义访问权限验证
pub struct AccessSiteSetting {
    pub user_id: u64,
}

#[async_trait::async_trait]
impl RbacCheck for AccessSiteSetting {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id, //资源访问用户
                relation,     //资源关系
                &[AccessRes::system(
                    "global-system",   //资源KEY
                    &["site-setting"], //必须验证权限
                    &[],               //可选验证权限
                )],
            )
            .await
    }
}

impl RbacResTpl for AccessSiteSetting {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["system"],
            user: false,
            key: "global-system",
            ops: vec!["site-setting"],
        }]
    }
}
