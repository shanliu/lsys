// 提供一个方式 用于统一定义验证所需资源跟验证方式
// !!!非必须,可外部自行组织!!!
// 关于收集使用到资源收集,可参考 ./res_tpl.rs
use super::{Rbac, RbacAccess, RoleRelationKey, UserRbacResult};

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
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()>;
}

impl Rbac {
    // 检查实现了RbacCheck trait的授权检查
    #[async_recursion::async_recursion]
    pub async fn check<'t>(
        &self,
        //待检测的权限结构
        //把多个资源跟相关依赖关系一个结构中
        //外部调用检测权限时传入该封装结构
        check: &'t RbacCheckDepend,
        //用户间关系KEY列表
        relation: Option<&'t [RoleRelationKey]>,
    ) -> UserRbacResult<()> {
        for pr in check.depends() {
            self.check(pr.as_ref(), relation).await?
        }
        check
            .check(&self.access, relation.unwrap_or_default())
            .await
    }
}
