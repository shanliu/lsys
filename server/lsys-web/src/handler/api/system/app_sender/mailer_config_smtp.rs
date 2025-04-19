use lsys_access::dao::AccessSession;
use lsys_app_sender::dao::SmtpConfig;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::dao::access::api::system::CheckAdminMailConfig;
use crate::dao::access::api::system::CheckAdminMailMgr;

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
}

pub async fn mailer_smtp_config_list(
    param: &MailerSmtpConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminMailConfig {})
        .await?;
    let row = req_dao
        .web_dao
        .app_sender
        .mailer
        .smtp_sender
        .list_config(param.ids.as_deref())
        .await?;
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

    Ok(JsonResponse::data(JsonData::body(json!({ "data": data }))))
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

pub async fn mailer_smtp_config_add(
    param: &MailerSmtpConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminMailConfig {})
        .await?;

    let row = req_dao
        .web_dao
        .app_sender
        .mailer
        .smtp_sender
        .add_config(
            &param.name,
            &SmtpConfig {
                host: param.host.to_owned(),
                port: param.port,
                timeout: param.timeout,
                email: if param.email.is_empty() {
                    param.user.clone()
                } else {
                    param.email.to_owned()
                },
                user: param.user.to_owned(),
                password: param.password.to_owned(),
                tls_domain: param.tls_domain.to_owned(),
                branch_limit: param
                    .branch_limit
                    .map(|e| if e == 0 { 1 } else { e })
                    .unwrap_or(1),
            },
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": row }))))
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

pub async fn mailer_smtp_config_check(
    param: &MailerSmtpConfigCheckParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminMailConfig {})
        .await?;

    req_dao
        .web_dao
        .app_sender
        .mailer
        .smtp_sender
        .check_config(&SmtpConfig {
            host: param.host.to_owned(),
            port: param.port,
            timeout: param.timeout,
            email: if param.email.is_empty() {
                param.user.clone()
            } else {
                param.email.to_owned()
            },
            user: param.user.to_owned(),
            password: param.password.to_owned(),
            tls_domain: param.tls_domain.to_owned(),
            branch_limit: 0,
        })
        .await?;
    Ok(JsonResponse::data(JsonData::body(
        json!({ "status": "ok" }),
    )))
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

pub async fn mailer_smtp_config_edit(
    param: &MailerSmtpConfigEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminMailConfig {})
        .await?;

    let row = req_dao
        .web_dao
        .app_sender
        .mailer
        .smtp_sender
        .edit_config(
            param.id,
            &param.name,
            &SmtpConfig {
                host: param.host.to_owned(),
                port: param.port,
                timeout: param.timeout,
                email: if param.email.is_empty() {
                    param.user.clone()
                } else {
                    param.email.to_owned()
                },
                user: param.user.to_owned(),
                password: param.password.to_owned(),
                tls_domain: param.tls_domain.to_owned(),
                branch_limit: param.branch_limit,
            },
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "num": row }))))
}

#[derive(Debug, Deserialize)]
pub struct MailerSmtpConfigDelParam {
    pub id: u64,
}

pub async fn mailer_smtp_config_del(
    param: &MailerSmtpConfigDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminMailConfig {})
        .await?;

    let row = req_dao
        .web_dao
        .app_sender
        .mailer
        .smtp_sender
        .del_config(param.id, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "num": row }))))
}

#[derive(Debug, Deserialize)]
pub struct MailerAppSmtpConfigAddParam {
    pub smtp_config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub from_email: String,
    pub reply_email: String,
    pub subject_tpl_id: String,
    pub body_tpl_id: String,
}

pub async fn mailer_tpl_config_smtp_add(
    param: &MailerAppSmtpConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminMailMgr {})
        .await?;

    let row = req_dao
        .web_dao
        .app_sender
        .mailer
        .smtp_sender
        .add_app_config(
            &param.name,
            0,
            &param.tpl_id,
            param.smtp_config_id,
            &param.from_email,
            &param.reply_email,
            &param.subject_tpl_id,
            &param.body_tpl_id,
            auth_data.user_id(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": row }))))
}
