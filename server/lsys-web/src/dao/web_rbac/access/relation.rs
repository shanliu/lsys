//可用会话角色定义
use crate::access_relation_tpl;

use crate::dao::{CheckRelationData, CheckRelationRole, CheckRelationTpl, RbacCheckRelationTpl};

pub struct TestRelation {
    pub level: i8,
}
impl RbacCheckRelationTpl for TestRelation {
    fn relation_data(&self) -> CheckRelationData {
        vec![CheckRelationRole {
            role_key: format!("app-{}", self.level),
            user_id: 0,
        }]
        .into()
    }
    fn tpl_data() -> Vec<CheckRelationTpl> {
        vec![CheckRelationTpl {
            key: "app-{appid}",
            user: false,
        }]
    }
}

pub fn relation_tpls() -> Vec<CheckRelationTpl> {
    access_relation_tpl!(TestRelation)
}
