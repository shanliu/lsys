use crate::{
    dao::RequestDao,
    {JsonData, JsonResult},
};
use lsys_sender::model::{SenderAliyunConfigStatus, SenderSmsAliyunStatus};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct SmserAliConfigListParam {
    pub ids: Option<Vec<u64>>,
}

pub async fn smser_ali_config_list<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: SmserAliConfigListParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(
            req_auth.user_data().user_id,
            &[],
            &res_data!(AdminAliSmsConfig),
        )
        .await?;
    let alisender = &req_dao.web_dao.smser.aliyun_sender;
    let row = alisender.list_config(param.ids.as_deref()).await?;
    Ok(JsonData::data(json!({ "data": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAliConfigAddParam {
    pub name: String,
    pub access_id: String,
    pub access_secret: String,
}

pub async fn smser_ali_config_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserAliConfigAddParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(
            req_auth.user_data().user_id,
            &[],
            &res_data!(AdminAliSmsConfig),
        )
        .await?;
    let alisender = &req_dao.web_dao.smser.aliyun_sender;
    let row = alisender
        .add_config(
            &param.name,
            &param.access_id,
            &param.access_secret,
            &req_auth.user_data().user_id,
        )
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAliConfigEditParam {
    pub id: u64,
    pub name: String,
    pub access_id: String,
    pub access_secret: String,
}

pub async fn smser_ali_config_edit<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: SmserAliConfigEditParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(
            req_auth.user_data().user_id,
            &[],
            &res_data!(AdminAliSmsConfig),
        )
        .await?;
    let alisender = &req_dao.web_dao.smser.aliyun_sender;
    let config = alisender.find_config_by_id(&param.id).await?;
    let row = alisender
        .edit_config(
            &config,
            &param.name,
            &param.access_id,
            &param.access_secret,
            &req_auth.user_data().user_id,
        )
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAliConfigDelParam {
    pub id: u64,
}

pub async fn smser_ali_config_del<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserAliConfigDelParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(
            req_auth.user_data().user_id,
            &[],
            &res_data!(AdminAliSmsConfig),
        )
        .await?;
    let alisender = &req_dao.web_dao.smser.aliyun_sender;
    let config = alisender.find_config_by_id(&param.id).await?;
    if SenderAliyunConfigStatus::Delete.eq(config.status) {
        return Ok(JsonData::data(json!({ "num": 0 })));
    }
    let row = alisender.del_config(&config).await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppAliConfigDelParam {
    pub app_config_id: u64,
}

pub async fn smser_app_ali_config_del<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: SmserAppAliConfigDelParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let alisender = &req_dao.web_dao.smser.aliyun_sender;
    let config = alisender
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
        .access
        .check(
            req_auth.user_data().user_id,
            &[],
            &res_data!(AppSender(config.app_id, req_auth.user_data().user_id)),
        )
        .await?;
    let row = alisender
        .del_app_config(&config, &req_auth.user_data().user_id)
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppAliConfigAddParam {
    pub app_id: u64,
    pub ali_config_id: u64,
    pub sms_tpl: String,
    pub aliyun_sms_tpl: String,
    pub aliyun_sign_name: String,
    pub try_num: u16,
}

pub async fn smser_app_ali_config_add<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: SmserAppAliConfigAddParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(
            req_auth.user_data().user_id,
            &[],
            &res_data!(AppSender(param.app_id, req_auth.user_data().user_id)),
        )
        .await?;
    let alisender = &req_dao.web_dao.smser.aliyun_sender;
    let config = alisender.find_config_by_id(&param.ali_config_id).await?;
    let row = alisender
        .add_app_config(
            &param.app_id,
            &config,
            &param.sms_tpl,
            &param.aliyun_sms_tpl,
            &param.aliyun_sign_name,
            &param.try_num,
            &req_auth.user_data().user_id,
        )
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}
