use lsys_rbac::dao::{
    AccessRes, RbacAccess, RbacCheck, RbacRelationTpl, RbacResTpl, ResTpl, RoleRelationKey,
    UserRbacResult,
};

use super::RelationApp;

//这里定义Oauth服务中用到资源
pub struct AccessOauthUserInfo {
    pub user_id: u64,
    pub app_id: u64,
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
                self.user_id,
                &RelationApp {
                    app_id: self.app_id,
                }
                .extend(relation),
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
    pub user_id: u64,
    pub app_id: u64,
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
                self.user_id,
                &RelationApp {
                    app_id: self.app_id,
                }
                .extend(relation),
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
    pub user_id: u64,
    pub app_id: u64,
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
                self.user_id,
                &RelationApp {
                    app_id: self.app_id,
                }
                .extend(relation),
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

pub struct AccessOauthUserAddress {
    pub user_id: u64,
    pub app_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessOauthUserAddress {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                &RelationApp {
                    app_id: self.app_id,
                }
                .extend(relation),
                &[AccessRes::system("global-oauth", &["user-address"], &[])],
            )
            .await
    }
}
impl RbacResTpl for AccessOauthUserAddress {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["oauth", "app"],
            user: false,
            key: "global-oauth",
            ops: vec!["user-address"],
        }]
    }
}
