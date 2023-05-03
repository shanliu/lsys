use lsys_user::dao::{
    account::user_password::UserPasswordConfig,
    auth::{SessionData, SessionTokenData, UserSession},
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    dao::{RequestDao, SiteConfig, WebDao},
    handler::access::AccessSiteSetting,
    JsonData, JsonResult,
};
use lsys_setting::dao::{NotFoundDefault, SettingKey};

#[derive(Debug, Deserialize)]
pub struct SiteConfigParam {
    pub site_tips: String,
    pub password_timeout: u64,
    pub disable_old_password: bool,
}

pub async fn site_config_set<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SiteConfigParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    //验证权限
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessSiteSetting {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;
    let mut transaction = req_dao.web_dao.db.begin().await?;
    if let Err(e) = req_dao
        .web_dao
        .setting
        .single
        .save::<UserPasswordConfig>(
            &None,
            UserPasswordConfig::key(),
            &UserPasswordConfig {
                timeout: param.password_timeout,
                disable_old_password: param.disable_old_password,
            },
            &req_auth.user_data().user_id,
            Some(&mut transaction),
            None,
        )
        .await
    {
        transaction.rollback().await?;
        return Err(e.into());
    };
    if let Err(e) = req_dao
        .web_dao
        .setting
        .single
        .save::<SiteConfig>(
            &None,
            SiteConfig::key(),
            &SiteConfig {
                site_tips: param.site_tips,
            },
            &req_auth.user_data().user_id,
            Some(&mut transaction),
            None,
        )
        .await
    {
        transaction.rollback().await?;
        return Err(e.into());
    };
    transaction.commit().await?;
    Ok(JsonData::message("ok"))
}

pub async fn site_config_get<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let site_config = &*req_dao
        .web_dao
        .setting
        .single
        .load::<SiteConfig>(&None)
        .await
        .notfound_default()?;
    let password = &*req_dao
        .web_dao
        .setting
        .single
        .load::<UserPasswordConfig>(&None)
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

pub async fn site_config_info(web_dao: &WebDao) -> JsonResult<JsonData> {
    let site_config = &*web_dao
        .setting
        .single
        .load::<SiteConfig>(&None)
        .await
        .notfound_default()?;
    Ok(JsonData::data(json!({ "data": {
        "site_tips":site_config.site_tips
    }})))
}
