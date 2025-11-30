use crate::common::JsonData;
use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonResponse, JsonResult, PageParam, UserAuthQueryDao},
    dao::access::api::system::user::CheckUserAppSenderMailConfig,
};
use lsys_access::dao::AccessSession;
use lsys_app_sender::model::SenderType;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct MailerTplListParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub id: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub tpl_id: Option<String>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}
pub async fn mailer_tpl_body_list(
    param: &MailerTplListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppSenderMailConfig {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_id(param.app_id)
        .await?;
    app.app_status_check()?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(&app, &[crate::handler::APP_FEATURE_MAIL])
        .await?;

    let data = req_dao
        .web_dao
        .app_sender
        .tpl
        .list_data(
            app.id,
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
                    app.id,
                    Some(SenderType::Mailer),
                    param.id,
                    param.tpl_id.as_deref(),
                )
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": data,"total":count }),
    )))
}

#[derive(Debug, Deserialize)]
pub struct MailerTplAddParam {
    pub app_id: u64,
    pub tpl_id: String,
    pub tpl_data: String,
}
pub async fn mailer_tpl_body_add(
    param: &MailerTplAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppSenderMailConfig {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_id(param.app_id)
        .await?;
    app.app_status_check()?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(&app, &[crate::handler::APP_FEATURE_MAIL])
        .await?;

    let id = req_dao
        .web_dao
        .app_sender
        .tpl
        .add(
            app.id,
            SenderType::Mailer,
            param.tpl_id.as_str(),
            &param.tpl_data,
            auth_data.user_id(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": id }))))
}

#[derive(Debug, Deserialize)]
pub struct MailerTplEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
    pub tpl_data: String,
}
pub async fn mailer_tpl_body_edit(
    param: &MailerTplEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let tpl = req_dao.web_dao.app_sender.tpl.find_by_id(&param.id).await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppSenderMailConfig {
                res_user_id: tpl.user_id,
            },
        )
        .await?;

    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_id(tpl.app_id)
        .await?;
    app.app_status_check()?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(&app, &[crate::handler::APP_FEATURE_MAIL])
        .await?;

    req_dao
        .web_dao
        .app_sender
        .tpl
        .edit(
            &tpl,
            &param.tpl_data,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct MailerTplDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
}
pub async fn mailer_tpl_body_del(
    param: &MailerTplDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let data = req_dao.web_dao.app_sender.tpl.find_by_id(&param.id).await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppSenderMailConfig {
                res_user_id: data.user_id,
            },
        )
        .await?;

    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_id(data.app_id)
        .await?;
    app.app_status_check()?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(&app, &[crate::handler::APP_FEATURE_MAIL])
        .await?;

    req_dao
        .web_dao
        .app_sender
        .tpl
        .del(&data, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}
