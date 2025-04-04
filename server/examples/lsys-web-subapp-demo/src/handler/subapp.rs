use lsys_app::model::AppModel;
use lsys_web::{
    common::{JsonData, JsonResult, RequestDao},
    dao::access::rest::CheckRestApp,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct DemoParam {
    pub text: String,
}
pub async fn demo_handler(
    param: &DemoParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    //全局启用app验证
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckRestApp {})
        .await?;
    //是否启用功能验证
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(app, &["my-app-feature"])
        //request_exter_feature ->featuer_data[my-app-feature]
        .await?;
    //业务逻辑。。。
    Ok(JsonData::data(json!({ "text":param.text,"app_id":app.id })))
}
