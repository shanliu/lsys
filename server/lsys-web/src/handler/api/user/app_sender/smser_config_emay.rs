use crate::common::JsonData;
use crate::common::{JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::user::CheckUserAppSenderSmsConfig;
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
#[derive(Debug, Deserialize)]
pub struct SmserEmayConfigListParam {
    #[serde(
        default,
        deserialize_with = "crate::common::deserialize_option_vec_u64"
    )]
    pub ids: Option<Vec<u64>>,
}

pub async fn smser_emay_config_list(
    param: &SmserEmayConfigListParam,
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

    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .emay_sender
        .list_config(param.ids.as_deref())
        .await?;

    let row = row
        .into_iter()
        .map(|e| {
            json!({
               "id": e.model().id,
               "name": e.model().name,
               "app_id":e.hide_app_id()
            })
        })
        .collect::<Vec<Value>>();

    Ok(JsonResponse::data(JsonData::body(json!({ "data": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppEmayConfigAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub config_id: u64,
    pub name: String,
    pub tpl_key: String,
    pub extended_code: String,
}

pub async fn smser_emay_app_config_add(
    param: &SmserAppEmayConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    super::smser_inner_access_check(param.app_id, auth_data.user_id(), req_dao).await?;
    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .emay_sender
        .add_app_config(
            &param.name,
            param.app_id,
            param.config_id,
            &param.tpl_key,
            &param.extended_code,
            auth_data.user_id(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": row }))))
}
