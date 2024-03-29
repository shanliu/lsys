use crate::{
    dao::RequestAuthDao,
    handler::access::{AccessAdminMailConfig, AccessAppSenderMailConfig},
    {JsonData, JsonResult},
};
use lsys_app_sender::dao::SmtpConfig;
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;

#[derive(Serialize, Default)]
pub struct ShowSmtpConfig {
    pub id: u64,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub user: String,
    pub email: String,
    pub hide_user: String,
    pub password: String,
    pub hide_password: String,
    pub tls_domain: String,
    pub change_user_id: u64,
    pub change_time: u64,
}

#[derive(Debug, Deserialize)]
pub struct MailerSmtpConfigListParam {
    pub ids: Option<Vec<u64>>,
    pub full_data: Option<bool>,
}

pub async fn mailer_smtp_config_list<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerSmtpConfigListParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let smtp_sender = &req_dao.web_dao.sender_mailer.smtp_sender;
    let row = smtp_sender
        .list_config(&param.ids)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let out = if param.full_data.unwrap_or(false) {
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
                &AccessAdminMailConfig {
                    user_id: req_auth.user_data().user_id,
                },
                None,
            )
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?;
        let data = row
            .into_iter()
            .map(|e| ShowSmtpConfig {
                id: e.model().id,
                name: e.model().name.to_owned(),
                change_user_id: e.model().change_user_id,
                change_time: e.model().change_time,
                host: e.host.clone(),
                port: e.port,
                timeout: e.timeout,
                user: e.user.clone(),
                email: if e.email.is_empty() {
                    e.user.clone()
                } else {
                    e.email.clone()
                },
                hide_user: e.hide_user(),
                password: e.password.clone(),
                hide_password: e.hide_password(),
                tls_domain: e.tls_domain.clone(),
            })
            .collect::<Vec<_>>();
        json!({ "data": data })
    } else {
        let row = row
            .into_iter()
            .map(|e| {
                json!({
                   "id": e.model().id,
                   "name": e.model().name,
                   "user":e.user,
                   "email":e.email,
                })
            })
            .collect::<Vec<Value>>();
        json!({ "data": row })
    };
    Ok(JsonData::data(out))
}

#[derive(Debug, Deserialize)]
pub struct MailerSmtpConfigAddParam {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub email: String,
    pub user: String,
    pub password: String,
    pub tls_domain: String,
    pub branch_limit: Option<u16>,
}

pub async fn mailer_smtp_config_add<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerSmtpConfigAddParam,
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
            &AccessAdminMailConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let smtp_sender = &req_dao.web_dao.sender_mailer.smtp_sender;
    let row = smtp_sender
        .add_config(
            &param.name,
            &SmtpConfig {
                host: param.host,
                port: param.port,
                timeout: param.timeout,
                email: if param.email.is_empty() {
                    param.user.clone()
                } else {
                    param.email
                },
                user: param.user,
                password: param.password,
                tls_domain: param.tls_domain,
                branch_limit: param
                    .branch_limit
                    .map(|e| if e == 0 { 1 } else { e })
                    .unwrap_or(1),
            },
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": row })))
}

#[derive(Debug, Deserialize)]
pub struct MailerSmtpConfigCheckParam {
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub email: String,
    pub user: String,
    pub password: String,
    pub tls_domain: String,
    pub branch_limit: Option<u16>,
}

pub async fn mailer_smtp_config_check<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerSmtpConfigCheckParam,
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
            &AccessAdminMailConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let smtp_sender = &req_dao.web_dao.sender_mailer.smtp_sender;
    smtp_sender
        .check_config(&SmtpConfig {
            host: param.host,
            port: param.port,
            timeout: param.timeout,
            email: if param.email.is_empty() {
                param.user.clone()
            } else {
                param.email
            },
            user: param.user,
            password: param.password,
            tls_domain: param.tls_domain,
            branch_limit: param
                .branch_limit
                .map(|e| if e == 0 { 1 } else { e })
                .unwrap_or(1),
        })
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "status": "ok" })))
}

#[derive(Debug, Deserialize)]
pub struct MailerSmtpConfigEditParam {
    pub id: u64,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub email: String,
    pub user: String,
    pub password: String,
    pub tls_domain: String,
    pub branch_limit: u16,
}

pub async fn mailer_smtp_config_edit<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerSmtpConfigEditParam,
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
            &AccessAdminMailConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let smtp_sender = &req_dao.web_dao.sender_mailer.smtp_sender;
    let row = smtp_sender
        .edit_config(
            &param.id,
            &param.name,
            &SmtpConfig {
                host: param.host,
                port: param.port,
                timeout: param.timeout,
                email: if param.email.is_empty() {
                    param.user.clone()
                } else {
                    param.email
                },
                user: param.user,
                password: param.password,
                tls_domain: param.tls_domain,
                branch_limit: param.branch_limit,
            },
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct MailerSmtpConfigDelParam {
    pub id: u64,
}

pub async fn mailer_smtp_config_del<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerSmtpConfigDelParam,
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
            &AccessAdminMailConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let smtp_sender = &req_dao.web_dao.sender_mailer.smtp_sender;
    let row = smtp_sender
        .del_config(
            &param.id,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct MailerAppSmtpConfigAddParam {
    pub app_id: u64,
    pub user_id: Option<u64>,
    pub smtp_config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub from_email: String,
    pub reply_email: String,
    pub subject_tpl_id: String,
    pub body_tpl_id: String,
}

pub async fn mailer_app_smtp_config_add<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: MailerAppSmtpConfigAddParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let uid = param.user_id.unwrap_or(req_auth.user_data().user_id);

    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderMailConfig {
                user_id: req_auth.user_data().user_id,
                res_user_id: uid,
                app_id: param.app_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let smtp_sender = &req_dao.web_dao.sender_mailer.smtp_sender;
    let row = smtp_sender
        .add_app_config(
            &param.name,
            &param.app_id,
            &param.tpl_id,
            &param.smtp_config_id,
            &param.from_email,
            &param.reply_email,
            &param.subject_tpl_id,
            &param.body_tpl_id,
            &uid,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": row })))
}
