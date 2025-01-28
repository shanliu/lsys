use lsys_app::model::AppModel;
use serde::Deserialize;
use serde_json::json;

use crate::common::RequestDao;

use crate::common::{JsonData, JsonResult};
use crate::dao::access::rest::CheckRestApp;

#[derive(Debug, Deserialize)]
pub struct SubAppViewParam {
    pub client_id: String,
}
//@todo 待定....
pub async fn subapp_view(
    param: &SubAppViewParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env(),
            &CheckRestApp { app_id: app.id },
            None,
        )
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .inner_feature_sub_app_check(app)
        .await?;

    let out_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_sub_app_by_client_id(app, &param.client_id)
        .await?;

    Ok(JsonData::data(json!({
        "name": out_app.name,
        "client_id":out_app.client_id,
        "sub_secret": out_app.client_secret,
        "user_id": out_app.user_id,
    })))
}
