use lsys_user::dao::AccountPasswordConfig;

use crate::common::JsonData;
use crate::common::JsonResult;
use crate::dao::access::api::system::CheckAdminSiteSetting;
use crate::dao::SiteConfig;
use crate::dao::SiteConfigData;
use lsys_access::dao::AccessSession;
use lsys_setting::dao::NotFoundResult;
use serde::Deserialize;
use serde_json::json;

use crate::common::UserAuthQueryDao;

#[derive(Debug, Deserialize)]
pub struct SiteConfigParam {
    pub site_tips: String,
    pub password_timeout: u64,
    pub disable_old_password: bool,
}

pub async fn site_config_set(
    param: &SiteConfigParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env().await?,
            &CheckAdminSiteSetting {},
            None,
        )
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?; //验证权限
    req_dao
        .web_dao
        .web_setting
        .save_site_setting_data(
            &auth_data,
            &SiteConfigData {
                site_tips: &param.site_tips,
                password_timeout: param.password_timeout,
                disable_old_password: param.disable_old_password,
            },
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

pub async fn site_config_get(req_dao: &UserAuthQueryDao) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env().await?,
            &CheckAdminSiteSetting {},
            None,
        )
        .await?;
    let site_config = req_dao
        .web_dao
        .web_setting
        .setting_dao
        .single
        .load::<SiteConfig>(None)
        .await
        .notfound_default()?;
    let password = req_dao
        .web_dao
        .web_setting
        .setting_dao
        .single
        .load::<AccountPasswordConfig>(None)
        .await
        .notfound_default()?;
    Ok(JsonData::data(json!({
       "config":{
        "site_tips":site_config.site_tips,
        "dis_old_password":password.disable_old_password,
        "timeout":password.timeout,
       }
    })))
}
