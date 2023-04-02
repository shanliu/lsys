use lsys_app::model::AppsModel;
use serde::Deserialize;
use serde_json::json;

use crate::dao::WebDao;

use crate::handler::access::AccessAppView;
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
    //   -> 系统分配appid
    //   -> 系统[访问用户+查询指定appid组成的关系key,检查权限[资源id:global-app-access-{appid}]]
    //   -> 返回查询appid密钥
    let out_app = app_dao
        .app
        .app_dao
        .app
        .cache()
        .find_by_client_id(&param.client_id)
        .await?;
    app_dao
        .user
        .rbac_dao
        .rbac
        .check(&AccessAppView {
            app: app.clone(),
            see_app: out_app.clone(),
        })
        .await?;

    Ok(JsonData::message("app data").set_data(json!({
        "name": out_app.name,
        "client_id":out_app.client_id,
        "client_secret": out_app.client_secret,
        "user_id": out_app.user_id,
        "status": out_app.status,
    })))
}
