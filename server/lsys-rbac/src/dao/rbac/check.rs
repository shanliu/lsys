// 提供一个方式 用于统一定义验证所需资源跟验证方式
// !!!非必须,可外部自行组织!!!
// 关于收集使用到资源收集,可参考 ./res_tpl.rs
use super::{Rbac, RbacAccess, UserRbacResult};

// 授权依赖类型
pub type RbacCheckDepend = dyn RbacCheck + std::marker::Sync + std::marker::Send;
// 授权检测trait,使用时统一定义授权
#[async_trait::async_trait]
pub trait RbacCheck {
    //当前授权依赖授权列表
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![]
    }
    // 进行授权当前
    async fn check(&self, access: &RbacAccess) -> UserRbacResult<()>;
}

impl Rbac {
    // 检查实现了RbacCheck trait的授权检查
    #[async_recursion::async_recursion]
    pub async fn check(&self, check: &RbacCheckDepend) -> UserRbacResult<()> {
        for pr in check.depends() {
            self.check(pr.as_ref()).await?
        }
        check.check(&self.access).await
    }
}
