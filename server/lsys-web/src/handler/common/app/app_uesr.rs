use lsys_app::{dao::app::AppDataWhere, model::AppStatus};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::dao::RequestAuthDao;

use crate::handler::access::{
    AccessAppSenderDoMail, AccessAppSenderDoSms, AccessUserAppConfirm, AccessUserAppEdit,
    AccessUserAppView,
};
use crate::{JsonData, JsonResult, PageParam};

#[derive(Debug, Deserialize)]
pub struct AppAddParam {
    user_id: Option<u64>,
    name: String,
    client_id: String,
    domain: Option<String>,
}

pub async fn app_add<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AppAddParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let user_id = param.user_id.unwrap_or(req_auth.user_data().user_id);
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAppEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let app_id = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .innernal_app_add(
            user_id,
            req_auth.user_data().user_id,
            param.name,
            param.client_id,
            param.domain.unwrap_or_default(),
            AppStatus::Init,
            None,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": app_id })))
}

#[derive(Debug, Deserialize)]
pub struct AppEditParam {
    app_id: u64,
    name: String,
    client_id: String,
    domain: Option<String>,
}

pub async fn app_edit<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AppEditParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    if AppStatus::Ok.eq(app.status) && param.client_id != app.client_id {
        return Ok(JsonData::default()
            .set_code(403)
            .set_message("已审核应用不能修改应用id"));
    }

    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAppEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: app.user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .app
        .app_dao
        .app
        .innernal_app_edit(
            &app,
            param.name,
            param.client_id,
            param.domain.unwrap_or_default(),
            req_auth.user_data().user_id,
            None,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct AppResetSecretParam {
    app_id: u64,
}

pub async fn app_reset_secret<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AppResetSecretParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAppView {
                user_id: req_auth.user_data().user_id,
                res_user_id: app.user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;

    let (client_secret, oauth_secret) = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .reset_secret(
            &app,
            &req_auth.user_data().user_id,
            None,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default()
        .set_data(json!({ "secret": client_secret,"oauth_secret":oauth_secret  })))
}

#[derive(Debug, Deserialize)]
pub struct AppViewSecretParam {
    app_id: u64,
}

pub async fn app_view_secret<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AppViewSecretParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAppView {
                user_id: req_auth.user_data().user_id,
                res_user_id: app.user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default()
        .set_data(json!({ "secret": app.client_secret,"oauth_secret":app.oauth_secret })))
}

#[derive(Debug, Deserialize)]
pub struct AppConfrimParam {
    app_id: u64,
}

pub async fn app_confirm<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AppConfrimParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAppConfirm {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let change = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .confirm_app(
            &app,
            req_auth.user_data().user_id,
            None,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "change": change })))
}

#[derive(Debug, Deserialize)]
pub struct AppListParam {
    pub count_num: Option<bool>,
    pub user_id: Option<u64>,
    pub app_id: Option<Vec<u64>>,
    pub status: Option<Vec<i8>>,
    pub client_ids: Option<Vec<String>>,
    pub page: Option<PageParam>,
    pub check_sms: Option<bool>,
    pub check_mail: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ShowAppData {
    pub id: u64,
    pub name: String,
    pub client_id: String,
    pub callback_domain: String,
    pub status: i8,
    pub user_id: u64,
    pub change_time: u64,
    pub confirm_user_id: u64,
    pub confirm_time: u64,
    pub is_sms: bool,
    pub is_mail: bool,
}

pub async fn app_list<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AppListParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let see_user_id = param.user_id;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAppView {
                user_id: req_auth.user_data().user_id,
                res_user_id: see_user_id.unwrap_or(0),
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let status = if let Some(e) = param.status {
        let mut out = Vec::with_capacity(e.len());
        for tmp in e {
            match AppStatus::try_from(tmp) {
                Ok(ts) => out.push(ts),
                Err(err) => return Err(req_dao.fluent_json_data(err)),
            };
        }
        Some(out)
    } else {
        None
    };
    let app_param = AppDataWhere {
        user_id: &see_user_id,
        status: &status,
        client_ids: &param.client_ids,
        app_ids: &param.app_id,
    };
    let appdata = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .app_data(&app_param, &Some(param.page.unwrap_or_default().into()))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let mut out = Vec::with_capacity(appdata.len());
    for tmp in appdata {
        let is_sms = if param.check_sms.unwrap_or(false) {
            req_dao
                .web_dao
                .user
                .rbac_dao
                .rbac
                .check(
                    &AccessAppSenderDoSms {
                        app_id: tmp.id,
                        user_id: tmp.user_id,
                    },
                    None,
                )
                .await
                .map(|_| true)
                .unwrap_or(false)
        } else {
            false
        };
        let is_mail = if param.check_mail.unwrap_or(false) {
            req_dao
                .web_dao
                .user
                .rbac_dao
                .rbac
                .check(
                    &AccessAppSenderDoMail {
                        app_id: tmp.id,
                        user_id: tmp.user_id,
                    },
                    None,
                )
                .await
                .map(|_| true)
                .unwrap_or(false)
        } else {
            false
        };

        out.push(ShowAppData {
            id: tmp.id,
            name: tmp.name,
            client_id: tmp.client_id,
            callback_domain: tmp.callback_domain,
            status: tmp.status,
            user_id: tmp.user_id,
            change_time: tmp.change_time,
            confirm_user_id: tmp.confirm_user_id,
            confirm_time: tmp.confirm_time,
            is_sms,
            is_mail,
        });
    }
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .app
                .app_dao
                .app
                .app_count(&app_param)
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": out,"total":count })))
}
