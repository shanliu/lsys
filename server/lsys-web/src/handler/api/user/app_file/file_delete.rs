//用户文件删除接口

use crate::common::{JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileDelete;
use lsys_access::dao::AccessSession;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FileDeleteParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub file_user_id: u64,
}

/// 删除文件
pub async fn file_delete(
    param: &FileDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let user_id = auth_data.user_id();

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

    if file_user.user_id != user_id {
        return Err(
            lsys_file::dao::FileError::Param(lsys_core::fluent_message!("file-not-found")).into(),
        );
    }

    let _app = super::app_check_get(file_user.app_id, true, &auth_data, req_dao).await?;

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
