use lsys_rbac::dao::{RbacRelationTpl, RelationTpl, RoleRelationKey};

//app 关系
pub struct RelationApp {
    pub app_id: u64,
}
impl RbacRelationTpl for RelationApp {
    fn relation_data(&self) -> Vec<RoleRelationKey> {
        vec![RoleRelationKey::system(format!("app-{}", self.app_id))]
    }
    fn tpl_data() -> Vec<RelationTpl> {
        vec![RelationTpl {
            key: "app-{appid}",
            user: false,
        }]
    }
}
