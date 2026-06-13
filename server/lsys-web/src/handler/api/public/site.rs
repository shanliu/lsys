use crate::common::JsonResult;
use crate::dao::SiteConfig;
use crate::dao::WebDao;
use lsys_setting::dao::NotFoundResult;
use lsys_setting::dao::SettingData;

pub async fn config_data(web_dao: &WebDao) -> JsonResult<SettingData<SiteConfig>> {
    let site_config = web_dao
        .web_setting
        .setting_dao
        .single
        .load::<SiteConfig>(None)
        .await
        .notfound_default()?;
    Ok(site_config)
}
