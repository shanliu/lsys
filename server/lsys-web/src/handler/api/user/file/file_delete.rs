//用户文件删除接口

use crate::common::{JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::user::CheckUserFileDelete;
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FileDeleteParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub file_id: u64,
}

/// 删除文件
pub async fn file_delete(
    param: &FileDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let user_id = auth_data.user_id();
    let app = super::app_check_get(param.app_id, true, &auth_data, req_dao).await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileDelete {
                res_user_id: user_id,
            },
        )
        .await?;

    req_dao
        .web_dao
        .web_files
        .file_dao
        .delete_file(user_id, app.id, param.file_id, None, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::default())
}
