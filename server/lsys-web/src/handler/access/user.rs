use lsys_app::model::AppsModel;
use lsys_rbac::dao::{AccessRes, RbacAccess, RbacCheck, RbacCheckDepend, UserRbacResult};

use crate::handler::access::AccessAdminManage;

use super::app_relation_key;
pub struct AccessSystemLogin {}
#[async_trait::async_trait]
impl RbacCheck for AccessSystemLogin {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(0, &[], &[AccessRes::system("user", &[], &["global-login"])])
            .await
    }
}

pub struct AccessSystemEmailConfirm {}
#[async_trait::async_trait]
impl RbacCheck for AccessSystemEmailConfirm {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                0,
                &[],
                &[AccessRes::system("user", &[], &["global-email-confirm"])],
            )
            .await
    }
}

pub struct AccessSystemMobileConfirm {}
#[async_trait::async_trait]
impl RbacCheck for AccessSystemMobileConfirm {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                0,
                &[],
                &[AccessRes::system("user", &[], &["global-mobile-confirm"])],
            )
            .await
    }
}

pub struct AccessSystemReSetPassword {}
#[async_trait::async_trait]
impl RbacCheck for AccessSystemReSetPassword {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                0,
                &[],
                &[AccessRes::system("user", &[], &["global-reset-confirm"])],
            )
            .await
    }
}

pub struct AccessUserAddressView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserAddressView {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user",
                        &["address-view"],
                        &[],
                    )],
                    vec![AccessRes::system("user", &["global-address-view"], &[])],
                ],
            )
            .await
    }
}

pub struct AccessUserAddressEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserAddressEdit {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user",
                        &["address-edit"],
                        &[],
                    )],
                    vec![AccessRes::system("user", &["global-address-edit"], &[])],
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

pub struct AccessUserExternalEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserExternalEdit {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                &[],
                &[AccessRes::user(
                    self.res_user_id,
                    "user",
                    &["external-change"],
                    &[],
                )],
            )
            .await
    }
}

pub struct AccessUserSetPassword {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserSetPassword {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user",
                        &["set-password"],
                        &[],
                    )],
                    vec![AccessRes::system("user", &["global-set-password"], &[])],
                ],
            )
            .await
    }
}

pub struct AccessUserNameEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserNameEdit {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user",
                        &["name-edit"],
                        &[],
                    )],
                    vec![AccessRes::system("user", &["global-name-edit"], &[])],
                ],
            )
            .await
    }
}

pub struct AccessUserInfoEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserInfoEdit {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user",
                        &["info-edit"],
                        &[],
                    )],
                    vec![AccessRes::system("user", &["global-info-edit"], &[])],
                ],
            )
            .await
    }
}

pub struct AccessUserEmailView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserEmailView {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user",
                        &["email-view"],
                        &[],
                    )],
                    vec![AccessRes::system("user", &["global-email-view"], &[])],
                ],
            )
            .await
    }
}

pub struct AccessUserEmailEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserEmailEdit {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user",
                        &["email-edit"],
                        &[],
                    )],
                    vec![AccessRes::system("user", &["global-email-edit"], &[])],
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

pub struct AccessUserAppView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserAppView {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                &[],
                &[if self.res_user_id == 0 {
                    AccessRes::system("app", &["global-app-view"], &[])
                } else {
                    AccessRes::user(self.res_user_id, "user", &["app-view"], &[])
                }],
            )
            .await
    }
}

pub struct AccessUserAppEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserAppEdit {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                &[],
                &[if self.res_user_id == 0 {
                    AccessRes::system("app", &["global-app-edit"], &[])
                } else {
                    AccessRes::user(self.res_user_id, "user", &["app-edit"], &[])
                }],
            )
            .await
    }
}

pub struct AccessUserMobileView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserMobileView {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user",
                        &["mobile-view"],
                        &[],
                    )],
                    vec![AccessRes::system("user", &["global-mobile-view"], &[])],
                ],
            )
            .await
    }
}

pub struct AccessUserMobileEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserMobileEdit {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "user",
                        &["mobile-edit"],
                        &[],
                    )],
                    vec![AccessRes::system("user", &["global-mobile-edit"], &[])],
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
pub struct AccessAppView {
    pub app: AppsModel,
    pub see_app: AppsModel,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAppView {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.app.user_id,
                &app_relation_key(&self.app),
                &[
                    vec![AccessRes::system(
                        &format!("app-{}", self.app.id),
                        &["global-app-view"],
                        &[],
                    )],
                    vec![AccessRes::user(
                        self.see_app.user_id,
                        &format!("app-{}", self.app.id),
                        &["app-view"],
                        &[],
                    )],
                ],
            )
            .await
    }
}

pub struct AccessAppRbacCheck {
    pub app: AppsModel,
}
#[async_trait::async_trait]
impl RbacCheck for AccessAppRbacCheck {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.app.user_id,
                &[],
                &[AccessRes::system(
                    &format!("app-{}", self.app.id),
                    &["global-rbac-check"],
                    &[],
                )],
            )
            .await
    }
}

pub struct AccessUserAppConfirm {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserAppConfirm {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                &[],
                &[AccessRes::system("app", &["global-app-confirm"], &[])],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessAdminManage {
            user_id: self.user_id,
        })]
    }
}
