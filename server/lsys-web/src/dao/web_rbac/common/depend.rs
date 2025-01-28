use lsys_rbac::dao::AccessCheckEnv;
use lsys_rbac::dao::RbacAccess;
use lsys_rbac::dao::RbacResult;

use super::CheckRelationData;

// 静态方式定义权限验证
// 以下实现仅用于解决本系统的资源依赖跟关系角色定义问题

// 授权依赖类型
pub type RbacCheckAccessDepend = dyn RbacCheckAccess + std::marker::Sync + std::marker::Send;
// 授权检测trait,使用时统一定义授权
#[async_trait::async_trait]
pub trait RbacCheckAccess {
    //当前授权依赖授权列表
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![]
    }
    // 进行授权当前
    async fn check(
        &self,
        access: &RbacAccess,
        check_env: &AccessCheckEnv<'_>,
        relation: &CheckRelationData,
    ) -> RbacResult<()>;
}
