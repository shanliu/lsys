//权限检查定义
pub mod api;
pub mod rest;

use crate::dao::RbacCheckAccessDepend;
use crate::dao::WebRbac;
use lsys_access::dao::SessionBody;
use lsys_access::model::UserModel;
use lsys_core::RequestEnv;
use lsys_rbac::dao::AccessCheckEnv;
use lsys_rbac::dao::AccessSessionRole;
use lsys_rbac::dao::RbacResult;
use std::ops::Deref;

pub struct RbacAccessCheckEnv<'t> {
    check_env: AccessCheckEnv<'t>,
}

impl<'t> Deref for RbacAccessCheckEnv<'t> {
    type Target = AccessCheckEnv<'t>;
    fn deref(&self) -> &Self::Target {
        &self.check_env
    }
}

impl<'t> RbacAccessCheckEnv<'t> {
    pub fn any(req_env: &'t RequestEnv) -> Self {
        Self {
            check_env: AccessCheckEnv {
                user_req_env: Some(req_env),
                session_role: vec![AccessSessionRole {
                    role_key: "system-global",
                    user_id: 0,
                    app_id: 0,
                }],
                ..Default::default()
            },
        }
    }
    pub fn user(user: &UserModel, req_env: &'t RequestEnv) -> Self {
        Self {
            check_env: AccessCheckEnv {
                user_req_env: Some(req_env),
                session_role: vec![
                    AccessSessionRole {
                        role_key: "system-global",
                        user_id: 0,
                        app_id: 0,
                    },
                    AccessSessionRole {
                        role_key: "system-login",
                        user_id: 0,
                        app_id: 0,
                    },
                ],
                user_id: user.id,
                user_app_id: user.app_id,
                user_login_token: None,
            },
        }
    }
    pub fn session_body(session_body: &'t SessionBody, req_env: &'t RequestEnv) -> Self {
        let check_env = AccessCheckEnv {
            user_req_env: Some(req_env),
            session_role: vec![
                AccessSessionRole {
                    role_key: "system-global",
                    user_id: 0,
                    app_id: 0,
                },
                AccessSessionRole {
                    role_key: "system-login",
                    user_id: 0,
                    app_id: 0,
                },
            ],
            user_id: session_body.user_id(),
            user_app_id: session_body.session().user_app_id,
            user_login_token: Some(session_body.token_data()),
        };
        Self { check_env }
    }
}

impl WebRbac {
    // pub async fn check(
    //     &self,

    //     //待检测的权限结构
    //     //把多个资源跟相关依赖关系一个结构中
    //     //外部调用检测权限时传入该封装结构
    //     check_dep: &RbacCheckAccessDepend,
    // ) -> RbacResult<()> {
    //     // let mut check_env = AccessCheckEnv {
    //     //     user_req_env: Some(req_env),
    //     //     session_role: vec![AccessSessionRole {
    //     //         role_key: "system-global",
    //     //         user_id: 0,
    //     //         app_id: 0,
    //     //     }],
    //     //     ..Default::default()
    //     // };
    //     // if let Some(session_body) = session_body_opt {
    //     //     check_env.user_login_token = Some(session_body.token_data());
    //     //     check_env.user_id = session_body.user_id();
    //     //     check_env.user_app_id = session_body.session().user_app_id;
    //     //     check_env.session_role.push(AccessSessionRole {
    //     //         role_key: "system-login",
    //     //         user_id: 0,
    //     //         app_id: 0,
    //     //     });
    //     // }
    //     self.inner_check(check_env, check_dep).await
    // }
    // 统一权限检测
    #[async_recursion::async_recursion]
    pub async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        check_dep: &RbacCheckAccessDepend,
    ) -> RbacResult<()> {
        if check_env.user_app_id == 0 && check_env.user_id > 0 && self.is_root(check_env.user_id) {
            return Ok(());
        }
        for pr in check_dep.depends() {
            self.check(check_env, pr.as_ref()).await?
        }
        check_dep.check(&self.rbac_dao.access, check_env).await
    }
}
