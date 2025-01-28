use serde::Deserialize;
use serde_json::json;

use crate::{
    common::{JsonData, JsonResult, UserAuthQueryDao},
    dao::SetPasswordData,
};
use lsys_access::dao::AccessSession;
#[derive(Debug, Deserialize)]
pub struct SetPasswordParam {
    pub old_password: Option<String>,
    pub new_password: String,
}
pub async fn user_set_password(
    param: &SetPasswordParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let pid = req_dao
        .web_dao
        .web_user
        .auth
        .user_set_password(
            &SetPasswordData {
                old_password: param.old_password.as_deref(),
                new_password: &param.new_password,
            },
            &req_auth,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": pid })))
}
