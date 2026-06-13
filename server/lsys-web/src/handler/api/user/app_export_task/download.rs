use crate::common::{JsonError, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileView;
use lsys_access::dao::AccessSession;
use lsys_core::fluent_message;
use lsys_file_manager::dao::FileManagerError;
use serde::Deserialize;

/// 用户端导出任务文件下载参数
#[derive(Debug, Deserialize)]
pub struct ExportDownloadParam {
    /// 任务 ID
    pub task_id: u64,
}

/// 用户端导出任务文件下载响应
pub struct ExportDownloadResponse {
    /// 文件读取迭代器
    pub iter: lsys_file::dao::FileReadIterator,
    /// 文件总大小
    pub file_size: u64,
    /// 文件名
    pub file_name: String,
    /// 内容类型
    pub content_type: String,
}

/// 用户端导出任务文件下载
///
/// 支持断点续传，通过 offset 参数指定读取起始位置。
pub async fn app_export_download(
    param: ExportDownloadParam,
    offset: u64,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<ExportDownloadResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    // 查询任务记录
    let task = web_dao
        .web_export.export_task
        .find_by_id(param.task_id)
        .await?
        .ok_or_else(|| JsonError::Message(fluent_message!("export-task-not-found")))?;

    // 使用任务记录中的 user_id 进行权限校验
    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileView {
                res_user_id: task.user_id,
            },
        )
        .await?;

    // 从 task 中获取 app_id 并校验权限
    super::app_check_get(task.app_id, false, &auth_data, req_dao, web_dao).await?;

    // 读取导出文件
    let (iter, file_model) = web_dao
        .web_export.export_task
        .read_export_file(&task, offset)
        .await
        .map_err(|e| match e {
            FileManagerError::Message(msg) => JsonError::Message(msg),
            _ => JsonError::Message(fluent_message!("export-download-error", "Unknown error")),
        })?;

    Ok(ExportDownloadResponse {
        iter,
        file_size: file_model.file_size,
        file_name: file_model.origin_name,
        content_type: file_model.content_type,
    })
}
