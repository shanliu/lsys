use lsys_rbac::dao::{
    AccessRes, RbacAccess, RbacCheck, RbacCheckDepend, RbacResTpl, ResTpl, RoleRelationKey,
    UserRbacResult,
};

pub struct AccessBarCodeView {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessBarCodeView {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[AccessRes::system("global-barcode", &["view"], &[])],
            )
            .await
    }
}
impl RbacResTpl for AccessBarCodeView {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["barcode"],
            user: false,
            key: "global-barcode",
            ops: vec!["view"],
        }]
    }
}

pub struct AccessBarCodeEdit {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessBarCodeEdit {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[AccessRes::system("global-barcode", &["edit"], &[])],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessBarCodeView {
            user_id: self.user_id,
        })]
    }
}
impl RbacResTpl for AccessBarCodeEdit {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["barcode"],
            user: false,
            key: "global-barcode",
            ops: vec!["edit"],
        }]
    }
}