use lsys_app::model::AppsModel;
use lsys_rbac::dao::{AccessRes, RbacAccess, RbacCheck, UserRbacResult};

use super::app_relation_key;
//这里定义Oauth服务中用到资源
pub struct AccessOauthUserInfo {
    pub app: AppsModel,
}
#[async_trait::async_trait]
impl RbacCheck for AccessOauthUserInfo {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.app.user_id,
                &app_relation_key(&self.app),
                &[AccessRes::system("app", &[], &["oauth-user-info"])],
            )
            .await
    }
}

pub struct AccessOauthUserEmail {
    pub app: AppsModel,
}
#[async_trait::async_trait]
impl RbacCheck for AccessOauthUserEmail {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.app.user_id,
                &app_relation_key(&self.app),
                &[AccessRes::system("app", &["oauth-user-email"], &[])],
            )
            .await
    }
}
pub struct AccessOauthUserMobile {
    pub app: AppsModel,
}
#[async_trait::async_trait]
impl RbacCheck for AccessOauthUserMobile {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.app.user_id,
                &app_relation_key(&self.app),
                &[AccessRes::system("app", &["oauth-user-mobile"], &[])],
            )
            .await
    }
}
