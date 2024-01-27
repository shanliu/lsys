use lsys_core::fluent_message;
use lsys_rbac::{
    dao::{
        AccessRes, RbacAccess, RbacCheck, RbacCheckDepend, RbacResTpl, ResTpl, RoleRelationKey,
        UserRbacError, UserRbacResult,
    },
    model::RbacRoleResOpRange,
};

pub struct AccessResEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessResEdit {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                relation,
                &[AccessRes::system("global-rbac-res", &["edit"], &[])],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessResView {
            user_id: self.user_id,
            res_user_id: self.res_user_id,
        })]
    }
}
impl RbacResTpl for AccessResEdit {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec!["rbac", "res"],
            user: false,
            key: "global-rbac-res",
            ops: vec!["edit"],
        }]
    }
}

pub struct AccessResView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessResView {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        if self.res_user_id > 0 {
            access
                .list_check(
                    self.user_id,
                    relation,
                    &[
                        vec![AccessRes::user(
                            self.res_user_id,
                            "rbac-res",
                            &["view"],
                            &[],
                        )],
                        vec![AccessRes::system("global-rbac-res", &["view"], &[])],
                    ],
                )
                .await
        } else {
            access
                .check(
                    self.user_id,
                    relation,
                    &[AccessRes::system("global-rbac-res", &["view"], &[])],
                )
                .await
        }
    }
}
impl RbacResTpl for AccessResView {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["rbac", "res"],
                user: true,
                key: "rbac-res",
                ops: vec!["view"],
            },
            ResTpl {
                tags: vec!["rbac", "res"],
                user: false,
                key: "global-rbac-res",
                ops: vec!["view"],
            },
        ]
    }
}

pub struct AccessRoleView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessRoleView {
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
                        "rbac-role",
                        &["view"],
                        &[],
                    )],
                    vec![AccessRes::system("global-rbac-role", &["view"], &[])],
                ],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessResView {
            user_id: self.user_id,
            res_user_id: self.res_user_id,
        })]
    }
}
impl RbacResTpl for AccessRoleView {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["rbac", "role"],
                user: true,
                key: "rbac-role",
                ops: vec!["view"],
            },
            ResTpl {
                tags: vec!["rbac", "role"],
                user: false,
                key: "global-rbac-role",
                ops: vec!["view"],
            },
        ]
    }
}

pub struct RoleOpCheck {
    pub op_id: u64,
    pub op_user_id: u64,
}

pub struct AccessRoleEdit {
    pub user_id: u64,
    pub res_user_id: u64,                   //当为0时表示系统资源
    pub op_range: Option<i8>,               //权限操作范围
    pub op_param: Option<Vec<RoleOpCheck>>, //权限操作相关参数
}
#[async_trait::async_trait]
impl RbacCheck for AccessRoleEdit {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        if let Some(ref rop) = self.op_param {
            for tmp in rop {
                if self.res_user_id > 0 && tmp.op_user_id != self.res_user_id {
                    return Err(UserRbacError::Check(vec![(
                        "bad-user".to_string(),
                        fluent_message!("rbac-edit-role-bad-user"),
                    )])); //"can't edit other user res to your role".to_string(),
                }
            }
        }
        //添加不同数据需要不同权限
        let mut gres = vec![AccessRes::user(
            self.res_user_id,
            "rbac-role",
            &["change"],
            &[],
        )];
        //操作范围为全局,必须有全局权限
        if RbacRoleResOpRange::AllowAll.eq(self.op_range.unwrap_or(0)) {
            gres.push(AccessRes::system("global-rbac-role", &["allow-res"], &[]));
        } else if RbacRoleResOpRange::DenyAll.eq(self.op_range.unwrap_or(0)) {
            gres.push(AccessRes::system("global-rbac-role", &["deny-res"], &[]));
        }
        access
            .list_check(
                self.user_id,
                relation,
                &[
                    gres,
                    vec![AccessRes::system("global-rbac-role", &["change-all"], &[])],
                ],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessResView {
            user_id: self.user_id,
            res_user_id: self.res_user_id,
        })]
    }
}
impl RbacResTpl for AccessRoleEdit {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["rbac", "role"],
                user: true,
                key: "rbac-role",
                ops: vec!["change"],
            },
            ResTpl {
                tags: vec!["rbac", "role"],
                user: false,
                key: "global-rbac-role",
                ops: vec!["change-all", "allow-res", "deny-res"],
            },
        ]
    }
}

pub struct AccessRoleViewList {
    pub user_id: u64,
    pub res_user_ids: Vec<u64>,
}
#[async_trait::async_trait]
impl RbacCheck for AccessRoleViewList {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        if self.res_user_ids.is_empty() {
            Ok(())
        } else {
            let roles = self
                .res_user_ids
                .clone()
                .into_iter()
                .collect::<std::collections::HashSet<u64>>()
                .iter()
                .map(|e| AccessRes::user(*e, "rbac-role", &["view"], &[]))
                .collect::<Vec<lsys_rbac::dao::AccessRes>>();
            access
                .list_check(
                    self.user_id,
                    relation,
                    &[
                        roles,
                        vec![AccessRes::system("global-rbac-role", &["view"], &[])],
                    ],
                )
                .await
        }
    }
}
impl RbacResTpl for AccessRoleViewList {
    fn tpl_data() -> Vec<ResTpl> {
        vec![
            ResTpl {
                tags: vec!["rbac", "role"],
                user: true,
                key: "rbac-role",
                ops: vec!["view"],
            },
            ResTpl {
                tags: vec!["rbac", "role"],
                user: false,
                key: "global-rbac-role",
                ops: vec!["view"],
            },
        ]
    }
}
