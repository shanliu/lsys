//权限检查定义
pub mod api;
pub mod rest;

use crate::dao::RbacCheckAccessDepend;
use crate::dao::WebRbac;
use lsys_access::dao::SessionBody;
use lsys_core::RequestEnv;
use lsys_rbac::dao::AccessCheckEnv;
use lsys_rbac::dao::AccessSessionRole;
use lsys_rbac::dao::RbacResult;

impl WebRbac {
    pub async fn check(
        &self,
        req_env: &RequestEnv,
        session_body_opt: Option<&SessionBody>,
        //待检测的权限结构
        //把多个资源跟相关依赖关系一个结构中
        //外部调用检测权限时传入该封装结构
        check: &RbacCheckAccessDepend,
    ) -> RbacResult<()> {
        let mut check_env = AccessCheckEnv {
            req_env: Some(req_env),
            session_role: vec![AccessSessionRole {
                role_key: "system-global",
                user_id: 0,
                app_id: 0,
            }],
            ..Default::default()
        };
        if let Some(session_body) = session_body_opt {
            check_env.login_token_data = Some(session_body.token_data());
            check_env.user_id = session_body.user_id();
            check_env.session_role.push(AccessSessionRole {
                role_key: "system-login",
                user_id: 0,
                app_id: 0,
            });
        }
        self.inner_check(&mut check_env, check).await
    }
    // 统一权限检测
    #[async_recursion::async_recursion]
    async fn inner_check(
        &self,
        check_env: &mut AccessCheckEnv<'_>,
        check: &RbacCheckAccessDepend,
    ) -> RbacResult<()> {
        for pr in check.depends() {
            self.inner_check(check_env, pr.as_ref()).await?
        }
        check.check(&self.rbac_dao.access, check_env).await
    }
}
