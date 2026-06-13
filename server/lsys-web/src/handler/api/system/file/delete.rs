use crate::common::{JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use lsys_access::dao::AccessSession;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AdminFileDeleteParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub file_ref_id: u64,
}

/// 管理员删除文件
pub async fn admin_file_delete(
    param: &AdminFileDeleteParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let file_user = web_dao
        .web_file.file_dao
        .helper()
        .find_file_ref_by_id(param.file_ref_id)
        .await?
        .ok_or_else(|| {
            lsys_file::dao::FileError::Param(lsys_core::fluent_message!("file-not-found"))
        })?;

    let file = web_dao
        .web_file.file_dao
        .helper()
        .find_file_by_id(file_user.file_id)
        .await?
        .ok_or_else(|| {
            lsys_file::dao::FileError::Param(lsys_core::fluent_message!("file-not-found"))
        })?;

    let ctx = web_dao
        .web_file.file_dao
        .file_ops()
        .create_context(&file_user)
        .with_file(&file)?;
    web_dao
        .web_file.file_dao
        .file_ops()
        .delete_file(ctx, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::default())
}
