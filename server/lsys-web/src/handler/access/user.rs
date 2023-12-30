use lsys_rbac::dao::{
    AccessRes, RbacAccess, RbacCheck, RbacCheckDepend, RbacResTpl, ResTpl, RoleRelationKey,
    UserRbacResult,
};

use crate::handler::access::AccessAdminManage;

pub struct AccessSystemLogin {}
#[async_trait::async_trait]
impl RbacCheck for AccessSystemLogin {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                0,
                relation,
                &[AccessRes::system("global-system", &[], &["login"])],
            )
            .await
    }
}

impl RbacResTpl for AccessSystemLogin {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["system"],
            user: false,
            key: "global-system",
            ops: vec!["login"],
        }]
    }
}

pub struct AccessSystemEmailConfirm {}
#[async_trait::async_trait]
impl RbacCheck for AccessSystemEmailConfirm {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                0,
                relation,
                &[AccessRes::system("global-system", &[], &["email-confirm"])],
            )
            .await
    }
}

impl RbacResTpl for AccessSystemEmailConfirm {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["system"],
            user: false,
            key: "global-system",
            ops: vec!["email-confirm"],
        }]
    }
}

pub struct AccessSystemMobileConfirm {}
#[async_trait::async_trait]
impl RbacCheck for AccessSystemMobileConfirm {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                0,
                relation,
                &[AccessRes::system("global-system", &[], &["mobile-confirm"])],
            )
            .await
    }
}

impl RbacResTpl for AccessSystemMobileConfirm {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["system"],
            user: false,
            key: "global-system",
            ops: vec!["mobile-confirm"],
        }]
    }
}

pub struct AccessSystemReSetPassword {}
#[async_trait::async_trait]
impl RbacCheck for AccessSystemReSetPassword {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                0,
                relation,
                &[AccessRes::system("global-system", &[], &["reset-confirm"])],
            )
            .await
    }
}

impl RbacResTpl for AccessSystemReSetPassword {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["system"],
            user: false,
            key: "global-system",
            ops: vec!["reset-confirm"],
        }]
    }
}

pub struct AccessUserAddressView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserAddressView {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                relation,
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user-address",
                        &["view"],
                        &[],
                    )],
                    vec![AccessRes::system("global-user-address", &["view"], &[])],
                ],
            )
            .await
    }
}

impl RbacResTpl for AccessUserAddressView {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["user"],
                user: true,
                key: "user-address",
                ops: vec!["view"],
            },
            ResTpl {
                tags: vec!["system"],
                user: false,
                key: "global-user-address",
                ops: vec!["view"],
            },
        ]
    }
}

pub struct AccessUserAddressEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserAddressEdit {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                relation,
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user-address",
                        &["edit"],
                        &[],
                    )],
                    vec![AccessRes::system("global-user-address", &["edit"], &[])],
                ],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessUserAddressView {
            user_id: self.user_id,
            res_user_id: self.res_user_id,
        })]
    }
}

impl RbacResTpl for AccessUserAddressEdit {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["user"],
                user: true,
                key: "user-address",
                ops: vec!["edit"],
            },
            ResTpl {
                tags: vec!["system"],
                user: false,
                key: "global-user-address",
                ops: vec!["edit"],
            },
        ]
    }
}

pub struct AccessUserExternalEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserExternalEdit {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[AccessRes::user(
                    self.res_user_id,
                    "user-external",
                    &["change"],
                    &[],
                )],
            )
            .await
    }
}

impl RbacResTpl for AccessUserExternalEdit {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["user"],
            user: true,
            key: "user-external",
            ops: vec!["change"],
        }]
    }
}

pub struct AccessUserSetPassword {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserSetPassword {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                relation,
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user-password",
                        &["set"],
                        &[],
                    )],
                    vec![AccessRes::system("global-user-password", &["set"], &[])],
                ],
            )
            .await
    }
}

impl RbacResTpl for AccessUserSetPassword {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["user"],
                user: true,
                key: "user-password",
                ops: vec!["set"],
            },
            ResTpl {
                tags: vec!["system"],
                user: false,
                key: "global-user-password",
                ops: vec!["set"],
            },
        ]
    }
}

pub struct AccessUserNameEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserNameEdit {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                relation,
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user-name",
                        &["edit"],
                        &[],
                    )],
                    vec![AccessRes::system("global-user-name", &["edit"], &[])],
                ],
            )
            .await
    }
}

impl RbacResTpl for AccessUserNameEdit {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["user"],
                user: true,
                key: "user-name",
                ops: vec!["edit"],
            },
            ResTpl {
                tags: vec!["system"],
                user: false,
                key: "global-user-name",
                ops: vec!["edit"],
            },
        ]
    }
}

pub struct AccessUserInfoEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserInfoEdit {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                relation,
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user-info",
                        &["edit"],
                        &[],
                    )],
                    vec![AccessRes::system("global-user-info", &["edit"], &[])],
                ],
            )
            .await
    }
}

impl RbacResTpl for AccessUserInfoEdit {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["user"],
                user: true,
                key: "user-info",
                ops: vec!["edit"],
            },
            ResTpl {
                tags: vec!["system"],
                user: false,
                key: "global-user-info",
                ops: vec!["edit"],
            },
        ]
    }
}

pub struct AccessUserEmailView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserEmailView {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                relation,
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user-email",
                        &["view"],
                        &[],
                    )],
                    vec![AccessRes::system("global-user-email", &["view"], &[])],
                ],
            )
            .await
    }
}

impl RbacResTpl for AccessUserEmailView {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["user"],
                user: true,
                key: "user-email",
                ops: vec!["view"],
            },
            ResTpl {
                tags: vec!["system"],
                user: false,
                key: "global-user-email",
                ops: vec!["view"],
            },
        ]
    }
}

pub struct AccessUserEmailEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserEmailEdit {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                relation,
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user-email",
                        &["edit"],
                        &[],
                    )],
                    vec![AccessRes::system("global-user-email", &["edit"], &[])],
                ],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessUserEmailView {
            user_id: self.user_id,
            res_user_id: self.res_user_id,
        })]
    }
}

impl RbacResTpl for AccessUserEmailEdit {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["user"],
                user: true,
                key: "user-email",
                ops: vec!["edit"],
            },
            ResTpl {
                tags: vec!["system"],
                user: false,
                key: "global-user-email",
                ops: vec!["edit"],
            },
        ]
    }
}

pub struct AccessUserAppView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserAppView {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[if self.res_user_id == 0 {
                    AccessRes::system("global-system", &["app-confirm"], &[])
                } else {
                    AccessRes::user(self.res_user_id, "user-app", &["view"], &[])
                }],
            )
            .await
    }
}

impl RbacResTpl for AccessUserAppView {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["user"],
                user: true,
                key: "user-app",
                ops: vec!["view"],
            },
            ResTpl {
                tags: vec!["system"],
                user: false,
                key: "global-system",
                ops: vec!["app-confirm"],
            },
        ]
    }
}

pub struct AccessUserAppEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserAppEdit {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[if self.res_user_id == 0 {
                    AccessRes::system("global-user-app", &["edit"], &[])
                } else {
                    AccessRes::user(self.res_user_id, "user-app", &["edit"], &[])
                }],
            )
            .await
    }
}

impl RbacResTpl for AccessUserAppEdit {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["user"],
                user: true,
                key: "user-app",
                ops: vec!["edit"],
            },
            ResTpl {
                tags: vec!["system"],
                user: false,
                key: "global-user-app",
                ops: vec!["edit"],
            },
        ]
    }
}

pub struct AccessUserMobileView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserMobileView {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                relation,
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user-mobile",
                        &["view"],
                        &[],
                    )],
                    vec![AccessRes::system("global-user-mobile", &["view"], &[])],
                ],
            )
            .await
    }
}

impl RbacResTpl for AccessUserMobileView {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["user"],
                user: true,
                key: "user-mobile",
                ops: vec!["view"],
            },
            ResTpl {
                tags: vec!["system"],
                user: false,
                key: "global-user-mobile",
                ops: vec!["view"],
            },
        ]
    }
}

pub struct AccessUserMobileEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserMobileEdit {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                relation,
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user-mobile",
                        &["edit"],
                        &[],
                    )],
                    vec![AccessRes::system("global-user-mobile", &["edit"], &[])],
                ],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessUserMobileView {
            user_id: self.user_id,
            res_user_id: self.res_user_id,
        })]
    }
}

impl RbacResTpl for AccessUserMobileEdit {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["user"],
                user: true,
                key: "user-mobile",
                ops: vec!["edit"],
            },
            ResTpl {
                tags: vec!["system"],
                user: false,
                key: "global-user-mobile",
                ops: vec!["edit"],
            },
        ]
    }
}

pub struct AccessUserAppConfirm {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserAppConfirm {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[AccessRes::system("global-system", &["app-confirm"], &[])],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessAdminManage {
            user_id: self.user_id,
        })]
    }
}

impl RbacResTpl for AccessUserAppConfirm {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["system"],
            user: false,
            key: "global-system",
            ops: vec!["app-confirm"],
        }]
    }
}

pub struct AccessUserAppStatus {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserAppStatus {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[AccessRes::system("global-system", &["app-status"], &[])],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessAdminManage {
            user_id: self.user_id,
        })]
    }
}

impl RbacResTpl for AccessUserAppStatus {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["system"],
            user: false,
            key: "global-system",
            ops: vec!["app-status"],
        }]
    }
}
