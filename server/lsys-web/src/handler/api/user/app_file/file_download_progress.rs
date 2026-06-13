// 文件下载进度查询接口

use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileView;
use lsys_access::dao::AccessSession;
use lsys_file::dao::FileProgressInfo;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;

#[derive(Debug, Deserialize)]
pub struct FileDownloadProgressParam {
    /// 要查询进度的 file_ref_id 列表（最多 50 个）
    pub file_ref_ids: Vec<String>,
}

/// 批量查询文件下载进度
///
/// 传入 file_ref_id 列表，服务端通过记录查出真实 file_id 并校验权限，
/// 返回各文件当前已下载字节、总大小、百分比、下载速度。
/// 对尚未开始下载或进度已清理的文件，结果中不包含该条目。
/// 轮询此接口（建议 1s 间隔）即可实时展示下载速度。
pub async fn file_download_progress(
    param: &FileDownloadProgressParam,
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

    // 解析并限制数量
    let ref_ids: Vec<u64> = param
        .file_ref_ids
        .iter()
        .take(50)
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();

    if ref_ids.is_empty() {
        return Ok(JsonResponse::data(JsonData::body(json!({ "items": [] }))));
    }

    // 批量查 file_ref 记录（走缓存，一次 IN 查询回填）
    let file_dao = &web_dao.web_file.file_dao;
    let ref_map = file_dao.cache().find_file_refs_by_ids(&ref_ids).await?;

    // 收集有效记录，同时要求所有记录必须属于同一 app_id 和 user_id
    let mut ref_to_file: Vec<(u64, u64)> = Vec::with_capacity(ref_ids.len()); // (file_ref_id, file_id)
    let mut common_app_id: Option<u64> = None;
    let mut common_user_id: Option<u64> = None;
    for ref_id in &ref_ids {
        let file_ref = match ref_map.get(ref_id) {
            Some(r) => r,
            None => continue, // 不存在直接跳过
        };
        // 强制要求所有记录属于同一 app_id 和 user_id，防止跨 app/user 混查
        match (common_app_id, common_user_id) {
            (None, _) => {
                common_app_id = Some(file_ref.app_id);
                common_user_id = Some(file_ref.user_id);
            }
            (Some(app_id), Some(user_id))
                if app_id != file_ref.app_id || user_id != file_ref.user_id =>
            {
                return Err(crate::common::JsonError::Message(
                    lsys_core::fluent_message!("param-error"),
                ));
            }
            _ => {}
        }
        ref_to_file.push((*ref_id, file_ref.file_id));
    }

    if ref_to_file.is_empty() {
        return Ok(JsonResponse::data(JsonData::body(json!({ "items": [] }))));
    }

    // 统一做一次权限校验（app_id / user_id 均来自数据库记录，不信任参数）
    let app_id = common_app_id.ok_or_else(|| crate::common::JsonError::Message(lsys_core::fluent_message!("param-error")))?;
    let user_id = common_user_id.ok_or_else(|| crate::common::JsonError::Message(lsys_core::fluent_message!("param-error")))?;
    super::app_check_get(app_id, false, &auth_data, req_dao, web_dao).await?;
    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileView {
                res_user_id: user_id,
            },
        )
        .await?;

    // 用内部 file_id 批量查进度
    let file_ids: Vec<u64> = ref_to_file.iter().map(|(_, fid)| *fid).collect();
    let progress_map = web_dao
        .web_file.file_dao
        .progress_tracker()
        .get_progress_batch(&file_ids)
        .await;

    // 按请求顺序组装结果，对外暴露 file_ref_id，不暴露内部 file_id 和分片细节
    let items: Vec<serde_json::Value> = ref_to_file
        .iter()
        .filter_map(|(ref_id, file_id)| {
            progress_map.get(file_id).map(|info| {
                json!({
                    "file_ref_id":      ref_id.to_string(),
                    "total_downloaded": info.total_downloaded,
                    "total_size":       info.total_size,
                    "percent":          info.percent,
                    "speed_bps":        info.speed_bps,
                })
            })
        })
        .collect();

    Ok(JsonResponse::data(JsonData::body(
        json!({ "items": items }),
    )))
}

// ──────────────────────────────────────────
// SSE 订阅接口
// ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct FileDownloadProgressSseParam {
    /// 要订阅进度的 file_ref_id 列表（最多 50 个）
    pub ref_ids: Vec<u64>,
}

/// 订阅文件下载进度实时推送（SSE）
///
/// 权限校验与 `file_download_progress` 完全一致：
/// - 所有 file_ref_id 必须属于同一 app_id + user_id
/// - 当前用户需通过 app 访问校验及文件查看 RBAC 校验
///
/// 校验通过后返回 `mpsc::Receiver<FileProgressInfo>`，调用方负责将其转为 SSE 流。
/// - 每条消息包含 `file_id`、进度数据和 `status` 字段
/// - 所有订阅文件均到达终态（`Completed` / `Failed`）或 Receiver drop 时，流自动结束
pub async fn file_download_progress_sse(
    param: &FileDownloadProgressSseParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<mpsc::Receiver<FileProgressInfo>> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;


    if param.ref_ids.is_empty() {
        let (_, rx) = mpsc::channel::<FileProgressInfo>(1);
        return Ok(rx);
    }

    let file_dao = &web_dao.web_file.file_dao;
    let ref_map = file_dao.cache().find_file_refs_by_ids(&param.ref_ids).await?;

    let mut ref_to_file: Vec<(u64, u64)> = Vec::with_capacity(param.ref_ids.len());
    let mut common_app_id: Option<u64> = None;
    let mut common_user_id: Option<u64> = None;
    for ref_id in &param.ref_ids {
        let file_ref = match ref_map.get(ref_id) {
            Some(r) => r,
            None => continue,
        };
        match (common_app_id, common_user_id) {
            (None, _) => {
                common_app_id = Some(file_ref.app_id);
                common_user_id = Some(file_ref.user_id);
            }
            (Some(app_id), Some(user_id))
                if app_id != file_ref.app_id || user_id != file_ref.user_id =>
            {
                return Err(crate::common::JsonError::Message(
                    lsys_core::fluent_message!("param-error"),
                ));
            }
            _ => {}
        }
        ref_to_file.push((*ref_id, file_ref.file_id));
    }

    if ref_to_file.is_empty() {
        let (_, rx) = mpsc::channel::<FileProgressInfo>(1);
        return Ok(rx);
    }

    let app_id = common_app_id.ok_or_else(|| crate::common::JsonError::Message(lsys_core::fluent_message!("param-error")))?;
    let user_id = common_user_id.ok_or_else(|| crate::common::JsonError::Message(lsys_core::fluent_message!("param-error")))?;
    super::app_check_get(app_id, false, &auth_data, req_dao, web_dao).await?;
    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileView {
                res_user_id: user_id,
            },
        )
        .await?;
	drop(auth_data);
    let file_ids: Vec<u64> = ref_to_file.iter().map(|(_, fid)| *fid).collect();
    let rx = web_dao
        .web_file.file_dao
        .progress_tracker()
        .subscribe_progress_batch(&file_ids)
        .await?;

    Ok(rx)
}
