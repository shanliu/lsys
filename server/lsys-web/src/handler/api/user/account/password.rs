use crate::common::JsonData;
use crate::{
    common::{JsonResponse, JsonResult, UserAuthQueryDao},
    dao::SetPasswordData,
};
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct SetPasswordParam {
    pub old_password: Option<String>,
    pub new_password: String,
}
pub async fn set_password(
    param: &SetPasswordParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let pid = req_dao
        .web_dao
        .web_user
        .auth
        .user_set_password(
            &SetPasswordData {
                old_password: param.old_password.as_deref(),
                new_password: &param.new_password,
            },
            &auth_data,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": pid }))))
}
