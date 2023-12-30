use lsys_app::model::AppsModel;
use serde::Deserialize;
use serde_json::json;

use crate::dao::WebDao;

use crate::{JsonData, JsonResult};

#[derive(Debug, Deserialize)]
pub struct SubAppViewParam {
    pub client_id: String,
}

pub async fn subapp_view(
    app_dao: &WebDao,
    app: &AppsModel,
    param: SubAppViewParam,
) -> JsonResult<JsonData> {
    // 请求   -> 模块
    //   -> 系统分配appid+请求子应用client_id
    //   -> 返回子应用的appid密钥
    let out_app = app_dao
        .app
        .app_dao
        .sub_app
        .cache()
        .find_sub_secret_by_client_id(&app.id, &param.client_id)
        .await?;

    Ok(JsonData::data(json!({
        "name": out_app.app_name,
        "app_id":out_app.app_id,
        "client_id":out_app.app_client_id,
        "sub_secret": out_app.sub_client_secret,
        "user_id": out_app.user_id,
    })))
}
