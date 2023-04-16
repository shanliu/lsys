use lsys_app::model::AppsModel;
use lsys_rbac::dao::{AccessRes, RbacAccess, RbacCheck, RoleRelationKey, UserRbacResult};
use lsys_web::{dao::WebDao, JsonData, JsonResult};
use serde::Deserialize;
use serde_json::json;

//这里定义访问权限验证
pub struct DomeAccess {
    pub app: AppsModel,
}

#[async_trait::async_trait]
impl RbacCheck for DomeAccess {
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        _relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()> {
        access
            .check(
                self.app.user_id,                                           //资源访问用户
                &[RoleRelationKey::system(format!("app-{}", self.app.id))], //资源关系
                &[AccessRes::system(
                    &format!("app-{}", self.app.id), //资源KEY
                    &["global-dome-auth"],           //必须验证权限
                    &[],                             //可选验证权限
                )],
            )
            .await
    }
}

#[derive(Debug, Deserialize)]
pub struct DemoParam {
    pub text: String,
}
pub async fn demo_handler(
    app_dao: &WebDao,
    app: &AppsModel,
    param: DemoParam,
) -> JsonResult<JsonData> {
    //验证权限
    app_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &DomeAccess {
                app: app.to_owned(),
            },
            None,
        )
        .await?;
    //业务逻辑。。。
    Ok(JsonData::data(json!({ "text":param.text })))
}
