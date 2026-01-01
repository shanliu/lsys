use crate::common::JsonData;
use crate::common::{JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::user::CheckUserAppSenderSmsConfig;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use crate::dao::access::RbacAccessCheckEnv;

#[derive(Debug, Deserialize)]
pub struct SmserHwConfigListParam {
    #[serde(
        default,
        deserialize_with = "crate::common::deserialize_option_vec_u64"
    )]
    pub ids: Option<Vec<u64>>,
}

pub async fn smser_hw_config_list(
    param: &SmserHwConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppSenderSmsConfig {
                 res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let data = req_dao
        .web_dao
        .app_sender
        .smser
        .hwyun_sender
        .list_config(param.ids.as_deref())
        .await?;

    let row = data
        .into_iter()
        .map(|e| {
            json!({
               "id": e.model().id,
               "name": e.model().name,
               "app_id":e.hide_access_key()
            })
        })
        .collect::<Vec<Value>>();

    Ok(JsonResponse::data(JsonData::body(json!({ "data": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppHwConfigAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub hw_config_id: u64,
    pub name: String,
    pub tpl_key: String,
    pub signature: String,
    pub sender: String,
    pub template_id: String,
    pub template_map: String,
}

pub async fn smser_hw_app_config_add(
    param: &SmserAppHwConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    super::smser_inner_access_check(param.app_id, auth_data.user_id(), req_dao).await?;
    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .hwyun_sender
        .add_app_config(
            &param.name,
            param.app_id,
            param.hw_config_id,
            &param.tpl_key,
            &param.signature,
            &param.sender,
            &param.template_id,
            &param.template_map,
            auth_data.user_id(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": row }))))
}
