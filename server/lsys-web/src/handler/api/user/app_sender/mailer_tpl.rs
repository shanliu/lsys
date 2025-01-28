use lsys_access::dao::AccessSession;
use lsys_app_sender::model::SenderType;
use serde::Deserialize;
use serde_json::json;

use crate::common::{JsonData, JsonResult, PageParam, UserAuthQueryDao};

#[derive(Debug, Deserialize)]
pub struct MailerTplListParam {
    pub user_id: Option<u64>,
    pub sender_type: Option<i8>,
    pub id: Option<u64>,
    pub tpl_id: Option<String>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}
pub async fn mailer_tpl_body_list(
    param: &MailerTplListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    let sender_type = match param.sender_type {
        Some(e) => Some(SenderType::try_from(e)?),
        None => None,
    };
    let data = req_dao
        .web_dao
        .app_sender
        .tpl
        .list_data(
            param.user_id.unwrap_or(req_auth.user_id()),
            sender_type,
            param.id,
            param.tpl_id.as_deref(),
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .app_sender
                .tpl
                .list_count(
                    param.user_id.unwrap_or(req_auth.user_id()),
                    sender_type,
                    param.id,
                    param.tpl_id.as_deref(),
                )
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": data,"total":count })))
}

#[derive(Debug, Deserialize)]
pub struct MailerTplAddParam {
    pub tpl_id: String,
    pub tpl_data: String,
    pub sender_type: i8,
}
pub async fn mailer_tpl_body_add(
    param: &MailerTplAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    let sender_type = SenderType::try_from(param.sender_type)?;
    let id = req_dao
        .web_dao
        .app_sender
        .tpl
        .add(
            sender_type,
            param.tpl_id.as_str(),
            &param.tpl_data,
            req_auth.user_id(),
            req_auth.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct MailerTplEditParam {
    pub id: u64,
    pub tpl_data: String,
}
pub async fn mailer_tpl_body_edit(
    param: &MailerTplEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let tpl = req_dao.web_dao.app_sender.tpl.find_by_id(&param.id).await?;

    req_dao
        .web_dao
        .app_sender
        .tpl
        .edit(
            &tpl,
            &param.tpl_data,
            req_auth.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct MailerTplDelParam {
    pub id: u64,
}
pub async fn mailer_tpl_body_del(
    param: &MailerTplDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let res = req_dao.web_dao.app_sender.tpl.find_by_id(&param.id).await;
    let data = match res {
        Ok(d) => d,
        Err(e) => return Err(e.into()),
    };

    req_dao
        .web_dao
        .app_sender
        .tpl
        .del(&data, req_auth.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}
