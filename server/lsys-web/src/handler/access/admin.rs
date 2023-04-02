use lsys_rbac::dao::{AccessRes, RbacAccess, RbacCheck, RbacCheckDepend, UserRbacResult};

pub struct AccessAdminManage {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAdminManage {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                &[],
                &[AccessRes::system("admin", &["main"], &[])],
            )
            .await
    }
}

pub struct AccessAdminSetting {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAdminSetting {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                &[],
                &[AccessRes::system("admin", &["setting"], &[])],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessAdminManage {
            user_id: self.user_id,
        })]
    }
}
