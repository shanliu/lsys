use lsys_rbac::dao::{
    AccessRes, RbacAccess, RbacCheck, RbacResTpl, ResTpl, RoleRelationKey, UserRbacResult,
};

pub struct AccessSubAppRbacCheck {
    pub user_id: u64,
    pub app_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessSubAppRbacCheck {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[AccessRes::system(
                    //系统控制指定用户APP是否有校验权限功能
                    &format!("global-app-{}", self.app_id),
                    &["access-check"],
                    &[],
                )],
            )
            .await
    }
}

impl RbacResTpl for AccessSubAppRbacCheck {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["system"],
            user: false,
            key: "global-app-{appid}",
            ops: vec!["access-check"],
        }]
    }
}
