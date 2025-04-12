use crate::common::JsonResult;
use crate::common::RequestDao;
use crate::dao::SiteConfig;
use lsys_setting::dao::NotFoundResult;
use lsys_setting::dao::SettingData;

pub async fn config_data(req_dao: &RequestDao) -> JsonResult<SettingData<SiteConfig>> {
    let site_config = req_dao
        .web_dao
        .web_setting
        .setting_dao
        .single
        .load::<SiteConfig>(None)
        .await
        .notfound_default()?;
    Ok(site_config)
}
