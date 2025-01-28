use crate::common::{JsonData, JsonResult, UserAuthQueryDao};

use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Debug, Deserialize)]
pub struct MailerSmtpConfigListParam {
    pub ids: Option<Vec<u64>>,
}

pub async fn mailer_smtp_config_list(
    param: &MailerSmtpConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let row = req_dao
        .web_dao
        .app_sender
        .mailer
        .smtp_sender
        .list_config(param.ids.as_deref())
        .await?;
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
    Ok(JsonData::data(json!({ "data": row })))
}

//系统应用邮件配置

#[derive(Debug, Deserialize)]
pub struct MailerAppSmtpConfigAddParam {
    pub app_id: u64,
    pub smtp_config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub from_email: String,
    pub reply_email: String,
    pub subject_tpl_id: String,
    pub body_tpl_id: String,
}

pub async fn mailer_app_smtp_config_add(
    param: &MailerAppSmtpConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    let row = req_dao
        .web_dao
        .app_sender
        .mailer
        .smtp_sender
        .add_app_config(
            &param.name,
            param.app_id,
            &param.tpl_id,
            param.smtp_config_id,
            &param.from_email,
            &param.reply_email,
            &param.subject_tpl_id,
            &param.body_tpl_id,
            req_auth.user_id(),
            req_auth.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": row })))
}
