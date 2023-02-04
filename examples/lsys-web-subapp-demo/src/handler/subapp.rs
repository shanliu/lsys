use lsys_app::model::AppsModel;
use lsys_web::{dao::WebDao, JsonData, JsonResult};
use serde::Deserialize;
use serde_json::json;

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
        .access
        .check(
            app.user_id,
            &[app_dao.app.app_relation_key(app).await],
            &res_data!(DomeRes(app.id)),
        )
        .await?;
    //业务逻辑。。。
    Ok(JsonData::message("dome message").set_data(json!({ "text":param.text })))
}
