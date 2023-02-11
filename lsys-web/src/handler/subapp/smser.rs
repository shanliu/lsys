use crate::{dao::WebDao, JsonData, JsonResult};
use lsys_app::model::AppsModel;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct SmsSendParam {
    pub base64_image: Vec<u8>,
}
pub async fn sms_send(
    _app_dao: &WebDao,
    _app: &AppsModel,
    _param: SmsSendParam,
) -> JsonResult<JsonData> {
    //TODO 公开发送接口。。。
    // app_dao
    //     .user
    //     .rbac_dao
    //     .rbac
    //     .access
    //     .check(
    //         app.user_id,
    //         &[app_dao.app.app_relation_key(app).await],
    //         &res_data!(AppDecode(app.id)),
    //     )
    //     .await?;
    Ok(JsonData::message("set name succ").set_data(json!({ "pass": 1 })))
}
