use lsys_access::dao::AccessSession;
use lsys_app_sender::{dao::SenderError, model::SenderType};
use serde::Deserialize;
use serde_json::json;

use crate::{
    common::{JsonData, JsonResult, PageParam, UserAuthQueryDao},
    dao::access::api::system::CheckAdminMailMgr,
};

#[derive(Debug, Deserialize)]
pub struct MailerTplListParam {
    pub id: Option<u64>,
    pub tpl_id: Option<String>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}
pub async fn mailer_tpl_body_list(
    param: &MailerTplListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminMailMgr {}, None)
        .await?;

    let data = req_dao
        .web_dao
        .app_sender
        .tpl
        .list_data(
            0,
            Some(SenderType::Mailer),
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
                    0,
                    Some(SenderType::Mailer),
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
}
pub async fn mailer_tpl_body_add(
    param: &MailerTplAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminMailMgr {}, None)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let id = req_dao
        .web_dao
        .app_sender
        .tpl
        .add(
            SenderType::Mailer,
            &param.tpl_id,
            &param.tpl_data,
            0,
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
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminMailMgr {}, None)
        .await?;
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
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminMailMgr {}, None)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let res = req_dao.web_dao.app_sender.tpl.find_by_id(&param.id).await;
    let data = match res {
        Ok(d) => d,
        Err(SenderError::Sqlx(sqlx::Error::RowNotFound)) => {
            return Ok(JsonData::message("not find"))
        }
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
