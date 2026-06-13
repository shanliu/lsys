//! 服务间认证验证接口
use crate::common::{JsonData, JsonResponse, JsonResult, RequestAuthDao};
use lsys_access::dao::AccessSession;
use serde::Serialize;
use serde_json::json;

/// login Token返回的用户信息
#[derive(Debug, Serialize)]
pub struct VerifyResult {
    pub user_id: u64,
    pub app_id: u64,
    pub nickname: String,
    pub userdata: String,
    pub username: String,
}

/// login Token并返回用户信息
///
/// 该接口用于服务间调用验证login Token的有效性，
/// 并返回Token对应的用户基本信息
pub async fn verify<T, D, S>(req_dao: &RequestAuthDao<T, D, S>) -> JsonResult<JsonResponse>
where
    T: lsys_access::dao::AccessSessionToken,
    D: lsys_access::dao::AccessSessionData,
    S: AccessSession<T, D>,
{
    // 获取会话数据
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let session_body = auth_data.session_body();
    let user_id = session_body.user_id();
    let user = session_body.user();

    Ok(JsonResponse::data(JsonData::body(json!({
        "user_id": user_id,
        "app_id": user.app_id,
        "nickname": user.user_nickname,
        "userdata": user.user_data,
        "username": user.user_account,
    }))))
}
