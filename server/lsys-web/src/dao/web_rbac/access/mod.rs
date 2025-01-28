mod check;
mod relation;
mod res;

pub use check::*;
pub use relation::*;
pub use res::*;

mod perm;
use crate::dao::CheckRelationData;
use crate::dao::RbacCheckAccessDepend;
use crate::dao::WebRbac;
use lsys_rbac::dao::AccessCheckEnv;
use lsys_rbac::dao::RbacResult;
pub use perm::*;

impl WebRbac {
    // 统一权限检测
    #[async_recursion::async_recursion]
    pub async fn check<'t>(
        &self,
        check_env: &'t AccessCheckEnv<'t>,
        //待检测的权限结构
        //把多个资源跟相关依赖关系一个结构中
        //外部调用检测权限时传入该封装结构
        check: &'t RbacCheckAccessDepend,
        //用户间关系KEY列表
        relation: Option<&'t CheckRelationData>,
    ) -> RbacResult<()> {
        for pr in check.depends() {
            self.check(check_env, pr.as_ref(), relation).await?
        }
        let def = CheckRelationData::default();
        let relation_tmp = match relation {
            Some(t) => t,
            None => &def,
        };
        check
            .check(&self.rbac_dao.access, check_env, relation_tmp)
            .await
    }
}
