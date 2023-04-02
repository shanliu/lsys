use lsys_rbac::{
    dao::{
        AccessRes, RbacAccess, RbacCheck, RbacCheckDepend, RbacResTpl, ResTpl, UserRbacError,
        UserRbacResult,
    },
    model::RbacRoleResOpRange,
};

pub struct AccessResEdit {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessResEdit {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                &[],
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
            tags: vec![],
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
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                &[],
                &[AccessRes::system("global-rbac-res", &["view"], &[])],
            )
            .await
    }
}
impl RbacResTpl for AccessResView {
    fn tpl_data() -> Vec<ResTpl> {
        vec![ResTpl {
            tags: vec![],
            user: false,
            key: "global-rbac-res",
            ops: vec!["view"],
        }]
    }
}

pub struct AccessUserResEdit {
    pub user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserResEdit {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .check(
                self.user_id,
                &[],
                &[
                    //自己不能管理自己资源,通过系统权限管理
                    AccessRes::system("rbac", &["global-res-view", "global-res-change"], &[]),
                    AccessRes::system("admin", &["main"], &[]),
                ],
            )
            .await
    }
}

pub struct AccessUserResView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserResView {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "rbac",
                        &["res-view"],
                        &[],
                    )],
                    vec![AccessRes::system("rbac", &["global-res-view"], &[])],
                ],
            )
            .await
    }
}

pub struct AccessUserRoleView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserRoleView {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "rbac",
                        &["role-view"],
                        &[],
                    )],
                    vec![AccessRes::system("rbac", &["global-role-view"], &[])],
                ],
            )
            .await
    }
}

pub struct RoleOpCheck {
    pub op_id: u64,
    pub op_user_id: u64,
}

pub struct AccessUserRoleAdd {
    pub user_id: u64,
    pub res_user_id: u64,
    pub op_range: i8,                       //权限操作范围
    pub op_param: Option<Vec<RoleOpCheck>>, //权限操作相关参数
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserRoleAdd {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        if let Some(ref rop) = self.op_param {
            for tmp in rop {
                if self.res_user_id > 0 && tmp.op_user_id != self.res_user_id {
                    return Err(UserRbacError::Check(vec![(
                        "bad-user".to_string(),
                        "can't add other user res to your role".to_string(),
                    )]));
                }
            }
        }
        //添加不同数据需要不同权限
        let mut gres = vec![AccessRes::user(
            self.res_user_id,
            "rbac",
            &["role-change"],
            &[],
        )];
        if RbacRoleResOpRange::AllowAll.eq(self.op_range) {
            gres.push(AccessRes::system("rbac", &["role-allow-res"], &[]));
        } else if RbacRoleResOpRange::DenyAll.eq(self.op_range) {
            gres.push(AccessRes::system("rbac", &["role-deny-res"], &[]));
        }
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    gres,
                    vec![AccessRes::user(
                        self.res_user_id,
                        "rbac",
                        &["global-role-change"],
                        &[],
                    )],
                ],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessUserResView {
            user_id: self.user_id,
            res_user_id: self.res_user_id,
        })]
    }
}

pub struct AccessUserRoleEdit {
    pub user_id: u64,
    pub res_user_id: u64,
    pub op_range: Option<i8>,               //权限操作范围
    pub op_param: Option<Vec<RoleOpCheck>>, //权限操作相关参数
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserRoleEdit {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        if let Some(ref rop) = self.op_param {
            for tmp in rop {
                if self.res_user_id > 0 && tmp.op_user_id != self.res_user_id {
                    return Err(UserRbacError::Check(vec![(
                        "bad-user".to_string(),
                        "can't edit other user res to your role".to_string(),
                    )]));
                }
            }
        }
        //添加不同数据需要不同权限
        let mut gres = vec![AccessRes::user(
            self.res_user_id,
            "rbac",
            &["role-change"],
            &[],
        )];
        if RbacRoleResOpRange::AllowAll.eq(self.op_range.unwrap_or(0)) {
            gres.push(AccessRes::system("rbac", &["role-allow-res"], &[]));
        } else if RbacRoleResOpRange::DenyAll.eq(self.op_range.unwrap_or(0)) {
            gres.push(AccessRes::system("rbac", &["role-deny-res"], &[]));
        }
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    gres,
                    vec![AccessRes::user(
                        self.res_user_id,
                        "rbac",
                        &["global-role-change"],
                        &[],
                    )],
                ],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![Box::new(AccessUserResView {
            user_id: self.user_id,
            res_user_id: self.res_user_id,
        })]
    }
}

pub struct AccessUserRoleViewList {
    pub user_id: u64,
    pub res_user_ids: Vec<u64>,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserRoleViewList {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        if self.res_user_ids.is_empty() {
            Ok(())
        } else {
            let roles = self
                .res_user_ids
                .clone()
                .into_iter()
                .collect::<std::collections::HashSet<u64>>()
                .iter()
                .map(|e| AccessRes::user(*e, "rbac", &["role-view"], &[]))
                .collect::<Vec<lsys_rbac::dao::AccessRes>>();
            access
                .list_check(
                    self.user_id,
                    &[],
                    &[
                        roles,
                        vec![AccessRes::system("rbac", &["global-role-view"], &[])],
                    ],
                )
                .await
        }
    }
}

pub struct AccessUserResAllView {
    pub user_id: u64,
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheck for AccessUserResAllView {
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()> {
        access
            .list_check(
                self.user_id,
                &[],
                &[
                    vec![AccessRes::user(
                        self.res_user_id,
                        "rbac",
                        &["res-view-all"],
                        &[],
                    )],
                    vec![AccessRes::system("rbac", &["global-res-view-all"], &[])],
                ],
            )
            .await
    }
}
