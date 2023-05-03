use crate::{
    dao::RequestDao,
    handler::access::{AccessAdminSmtpConfig, AccessAppSenderSmsConfig},
    {JsonData, JsonResult},
};
use lsys_sender::{dao::SmtpConfig, model::SenderSmsAliyunStatus};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct MailerSmtpConfigListParam {
    pub ids: Option<Vec<u64>>,
    pub full_data: Option<bool>,
}

pub async fn mailer_smtp_config_list<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: MailerSmtpConfigListParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let smtp_sender = &req_dao.web_dao.sender_mailer.smtp_sender;
    let row = smtp_sender.list_config(&param.ids).await?;
    let row = if param.full_data.unwrap_or(false) {
        let req_auth = req_dao.user_session.read().await.get_session_data().await?;
        req_dao
            .web_dao
            .user
            .rbac_dao
            .rbac
            .check(
                &AccessAdminSmtpConfig {
                    user_id: req_auth.user_data().user_id,
                },
                None,
            )
            .await?;
        json!({ "data": row })
    } else {
        let row = row
            .into_iter()
            .map(|e| {
                json!({
                   "id": e.id,
                   "name": e.name,
                   "user":e.hide_user,
                   "email":e.email,
                })
            })
            .collect::<Vec<Value>>();
        json!({ "data": row })
    };
    Ok(JsonData::data(row))
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
}

pub async fn mailer_smtp_config_add<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: MailerSmtpConfigAddParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminSmtpConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;
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
            },
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await?;
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
}

pub async fn mailer_smtp_config_check<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: MailerSmtpConfigCheckParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminSmtpConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;
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
        })
        .await?;
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
}

pub async fn mailer_smtp_config_edit<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: MailerSmtpConfigEditParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminSmtpConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;
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
            },
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct MailerSmtpConfigDelParam {
    pub id: u64,
}

pub async fn mailer_smtp_config_del<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: MailerSmtpConfigDelParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminSmtpConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;
    let smtp_sender = &req_dao.web_dao.sender_mailer.smtp_sender;
    let row = smtp_sender
        .del_config(
            &param.id,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct MailerAppSmtpConfigDelParam {
    pub app_config_id: u64,
}

pub async fn mailer_app_smtp_config_del<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: MailerAppSmtpConfigDelParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let smtp_sender = &req_dao.web_dao.sender_mailer.smtp_sender;
    let config = smtp_sender
        .find_app_config_by_id(&param.app_config_id)
        .await?;
    if SenderSmsAliyunStatus::Delete.eq(config.status) {
        return Ok(JsonData::data(json!({ "num": 0 })));
    }

    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderSmsConfig {
                user_id: req_auth.user_data().user_id,
                res_user_id: config.user_id,
                app_id: config.app_id,
            },
            None,
        )
        .await?;

    let row = smtp_sender
        .del_app_config(
            &config,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await?;
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
    pub subject_tpl_id: String,
    pub body_tpl_id: String,
    pub try_num: u16,
}

pub async fn mailer_app_smtp_config_add<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: MailerAppSmtpConfigAddParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let uid = param.user_id.unwrap_or(req_auth.user_data().user_id);

    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderSmsConfig {
                user_id: req_auth.user_data().user_id,
                res_user_id: uid,
                app_id: param.app_id,
            },
            None,
        )
        .await?;

    let smtp_sender = &req_dao.web_dao.sender_mailer.smtp_sender;
    let row = smtp_sender
        .add_app_config(
            &param.name,
            &param.app_id,
            &param.tpl_id,
            &param.smtp_config_id,
            &param.from_email,
            &param.subject_tpl_id,
            &param.body_tpl_id,
            &param.try_num,
            &uid,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": row })))
}

#[derive(Debug, Deserialize)]
pub struct MailerAppSmtpConfigListParam {
    pub user_id: Option<u64>,
    pub id: Option<u64>,
    pub app_id: Option<u64>,
    pub tpl_id: Option<String>,
}

pub async fn mailer_app_smtp_config_list<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: MailerAppSmtpConfigListParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderSmsConfig {
                user_id: req_auth.user_data().user_id,
                res_user_id: param.user_id.unwrap_or(req_auth.user_data().user_id),
                app_id: param.app_id.unwrap_or_default(),
            },
            None,
        )
        .await?;

    let smtp_sender = &req_dao.web_dao.sender_mailer.smtp_sender;
    let row = smtp_sender
        .find_app_config(&param.id, &param.user_id, &param.app_id, &param.tpl_id)
        .await?
        .into_iter()
        .map(|e| {
            json!({
                "config":e.0,
                "user":e.1.hide_user(),
                "name":e.1.model().name,
                "email":e.1.email,
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(json!({ "data": row })))
}
