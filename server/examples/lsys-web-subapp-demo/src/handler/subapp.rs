use lsys_app::model::AppModel;
use lsys_web::{
    common::{JsonData, JsonResponse, JsonResult, RequestDao},
    dao::access::{rest::CheckRestApp, RbacAccessCheckEnv},
};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct DemoParam {
    pub text: String,
}

pub async fn demo_api1(
    param: &DemoParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    //全局启用app验证
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
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
    Ok(JsonResponse::data(JsonData::body(
        json!({ "text":param.text,"app_id":app.id }),
    )))
}
