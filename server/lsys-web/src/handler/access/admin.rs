use lsys_rbac::dao::{
    AccessRes, RbacAccess, RbacCheck, RbacCheckDepend, RbacResTpl, ResTpl, RoleRelationKey,
    UserRbacResult,
};

pub struct AccessAdminManage {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAdminManage {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[AccessRes::system("global-system", &["main"], &[])],
            )
            .await
    }
}
impl RbacResTpl for AccessAdminManage {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec![],
            user: false,
            key: "global-system",
            ops: vec!["main"],
        }]
    }
}

pub struct AccessAdminSetting {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAdminSetting {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[AccessRes::system("global-system", &["setting"], &[])],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessAdminManage {
            user_id: self.user_id,
        })]
    }
}
impl RbacResTpl for AccessAdminSetting {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec![],
            user: false,
            key: "global-system",
            ops: vec!["setting"],
        }]
    }
}
