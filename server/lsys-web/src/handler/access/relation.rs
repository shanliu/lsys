use lsys_app::model::AppsModel;
use lsys_rbac::dao::{RbacRelationTpl, RelationTpl, RoleRelationKey};

//app 关系
pub struct RelationApp<'t> {
    pub app: &'t AppsModel,
}
impl<'t> RbacRelationTpl for RelationApp<'t> {
    fn relation_data(&self) -> Vec<RoleRelationKey> {
        vec![RoleRelationKey::system(format!("app-{}", self.app.id))]
    }
    fn tpl_data() -> Vec<RelationTpl> {
        vec![RelationTpl {
            key: "app-{appid}",
            user: false,
        }]
    }
}
