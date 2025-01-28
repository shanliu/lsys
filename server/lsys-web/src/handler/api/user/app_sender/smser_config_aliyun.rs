use crate::common::{JsonData, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::user::CheckAppSenderSmsConfig;
use lsys_access::dao::AccessSession;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
#[derive(Debug, Deserialize)]
pub struct SmserAliConfigListParam {
    pub ids: Option<Vec<u64>>,
}

#[derive(Serialize)]
pub struct ShowAliYunConfig {
    pub id: u64,
    pub name: String,
    pub region: String,
    pub access_id: String,
    pub hide_access_id: String,
    pub access_secret: String,
    pub change_user_id: u64,
    pub change_time: u64,
    pub limit: u16,
    pub callback_url: String,
    pub callback_key: String,
}

pub async fn smser_ali_config_list(
    param: &SmserAliConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .aliyun_sender
        .list_config(param.ids.as_deref())
        .await?;

    let row = row
        .into_iter()
        .map(|e| {
            json!({
                "id": e.model().id,
                "name": e.model().name,
                "app_id":e.hide_access_id()
            })
        })
        .collect::<Vec<Value>>();

    Ok(JsonData::data(json!({
        "data":row
    })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppAliConfigAddParam {
    pub app_id: u64,
    pub ali_config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub aliyun_sms_tpl: String,
    pub aliyun_sign_name: String,
}

pub async fn smser_ali_app_config_add(
    param: &SmserAppAliConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env().await?,
            &CheckAppSenderSmsConfig {
                res_user_id: auth_data.user_id(),
            },
            None,
        )
        .await?;

    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .aliyun_sender
        .add_app_config(
            &param.name,
            param.app_id,
            param.ali_config_id,
            &param.tpl_id,
            &param.aliyun_sms_tpl,
            &param.aliyun_sign_name,
            auth_data.user_id(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": row })))
}
