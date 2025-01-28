
//RBAC中角色相关实现
use super::RbacRole;

pub struct RbacRoleCache<'t> {
    pub(crate) role: &'t RbacRole,
}

impl RbacRole {
    pub fn cache(&self) -> RbacRoleCache<'_> {
        RbacRoleCache { role: self }
    }
}
