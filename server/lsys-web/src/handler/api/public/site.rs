use crate::common::JsonData;
use crate::common::JsonResult;
use crate::common::RequestDao;
use crate::dao::SiteConfig;
use lsys_setting::dao::NotFoundResult;
use serde_json::json;
pub async fn config_info(req_dao: &RequestDao) -> JsonResult<JsonData> {
    let site_config = req_dao
        .web_dao
        .web_setting
        .setting_dao
        .single
        .load::<SiteConfig>(None)
        .await
        .notfound_default()?;
    Ok(JsonData::data(json!({ "data": {
        "site_tips":site_config.site_tips
    }})))
}
