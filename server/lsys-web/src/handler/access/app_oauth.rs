use lsys_app::model::AppsModel;
use lsys_rbac::dao::{
    AccessRes, RbacAccess, RbacCheck, RbacRelationTpl, RbacResTpl, ResTpl, RoleRelationKey,
    UserRbacResult,
};

use super::RelationApp;

//这里定义Oauth服务中用到资源
pub struct AccessOauthUserInfo {
    pub app: AppsModel,
}
#[async_trait::async_trait]
impl RbacCheck for AccessOauthUserInfo {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.app.user_id,
                &RelationApp { app: &self.app }.extend(relation),
                &[AccessRes::system("global-oauth", &[], &["user-info"])],
            )
            .await
    }
}
impl RbacResTpl for AccessOauthUserInfo {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["oauth", "app"],
            user: false,
            key: "global-oauth",
            ops: vec!["user-info"],
        }]
    }
}

pub struct AccessOauthUserEmail {
    pub app: AppsModel,
}
#[async_trait::async_trait]
impl RbacCheck for AccessOauthUserEmail {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.app.user_id,
                &RelationApp { app: &self.app }.extend(relation),
                &[AccessRes::system("global-oauth", &["user-email"], &[])],
            )
            .await
    }
}
impl RbacResTpl for AccessOauthUserEmail {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["oauth", "app"],
            user: false,
            key: "global-oauth",
            ops: vec!["user-email"],
        }]
    }
}

pub struct AccessOauthUserMobile {
    pub app: AppsModel,
}
#[async_trait::async_trait]
impl RbacCheck for AccessOauthUserMobile {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.app.user_id,
                &RelationApp { app: &self.app }.extend(relation),
                &[AccessRes::system("global-oauth", &["user-mobile"], &[])],
            )
            .await
    }
}
impl RbacResTpl for AccessOauthUserMobile {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["oauth", "app"],
            user: false,
            key: "global-oauth",
            ops: vec!["user-mobile"],
        }]
    }
}
