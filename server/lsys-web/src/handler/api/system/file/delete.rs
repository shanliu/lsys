use crate::common::{JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use lsys_access::dao::AccessSession;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AdminFileDeleteParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub file_user_id: u64,
}

/// 管理员删除文件
pub async fn admin_file_delete(
    param: &AdminFileDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let file_user = req_dao
        .web_dao
        .web_files
        .file_dao
        .helper()
        .find_file_user_by_id(param.file_user_id)
        .await?
        .ok_or_else(|| {
            lsys_file::dao::FileError::Param(lsys_core::fluent_message!("file-not-found"))
        })?;

    let file = req_dao
        .web_dao
        .web_files
        .file_dao
        .helper()
        .find_file_by_id(file_user.file_id)
        .await?
        .ok_or_else(|| {
            lsys_file::dao::FileError::Param(lsys_core::fluent_message!("file-not-found"))
        })?;

    let ctx = req_dao
        .web_dao
        .web_files
        .file_dao
        .create_context(&file_user)
        .with_file(&file)?;
    req_dao
        .web_dao
        .web_files
        .file_dao
        .delete_file(ctx, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::default())
}
