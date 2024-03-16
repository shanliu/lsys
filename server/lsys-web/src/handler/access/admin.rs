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

pub struct AccessAdminUserBase {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAdminUserBase {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                relation,
                &[vec![
                    AccessRes::system("global-system", &["main"], &[]),
                    AccessRes::system(&format!("global-user-{}", self.user_id), &["base"], &[]),
                ]],
            )
            .await
    }
}
impl RbacResTpl for AccessAdminUserBase {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["system", "user-list"],
            user: false,
            key: "global-user-{user_id}",
            ops: vec!["base"],
        }]
    }
}

pub struct AccessAdminUserFull {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAdminUserFull {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                relation,
                &[vec![AccessRes::system(
                    &format!("global-user-{}", self.user_id),
                    &["full"],
                    &[],
                )]],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessAdminUserBase {
            user_id: self.user_id,
        })]
    }
}
impl RbacResTpl for AccessAdminUserFull {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["system", "user-list"],
            user: false,
            key: "global-user-{user_id}",
            ops: vec!["full"],
        }]
    }
}

pub struct AccessAdminChangeLogsView {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAdminChangeLogsView {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[AccessRes::system("global-system", &["see-change-log"], &[])],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessAdminManage {
            user_id: self.user_id,
        })]
    }
}
impl RbacResTpl for AccessAdminChangeLogsView {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["system", "change log"],
            user: false,
            key: "global-system",
            ops: vec!["see-change-log"],
        }]
    }
}

pub struct AccessAdminDocsEdit {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAdminDocsEdit {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[AccessRes::system("global-system", &["edit-docs"], &[])],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessAdminManage {
            user_id: self.user_id,
        })]
    }
}
impl RbacResTpl for AccessAdminDocsEdit {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["system", "docs"],
            user: false,
            key: "global-system",
            ops: vec!["edit-docs"],
        }]
    }
}
