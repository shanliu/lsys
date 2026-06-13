use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use lsys_access::dao::AccessSession;
use lsys_file::model::FileUserStatus;
use serde::Deserialize;
use serde_json::json;

// ==================== 更新过期时间 ====================

#[derive(Debug, Deserialize)]
pub struct FileUpdateExpireTimeParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub file_ref_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub expire_time: u64,
}

/// 用户更新文件过期时间
pub async fn file_update_expire_time(
    param: &FileUpdateExpireTimeParam,
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
    let user_id = auth_data.user_id();
    super::app_check_get(param.app_id, true, &auth_data, req_dao, web_dao).await?;

    let file_dao = &web_dao.web_file.file_dao;

    // 验证文件属于当前用户和应用
    let file_user = file_dao
        .helper()
        .find_file_ref_by_id(param.file_ref_id)
        .await?
        .filter(|r| {
            r.user_id == user_id
                && r.app_id == param.app_id
                && r.status == FileUserStatus::Normal as i8
        })
        .ok_or_else(|| {
            crate::common::JsonError::Message(lsys_core::fluent_message!("file-not-found"))
        })?;

    let rows = file_dao
        .file_ops()
        .update_expire_time(
            &file_user,
            param.expire_time,
            user_id,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "updated": rows > 0,
        "rows_affected": rows,
    }))))
}

// ==================== 文件拷贝 ====================

#[derive(Debug, Deserialize)]
pub struct FileCopyParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub file_ref_id: u64,
    pub storage_type: String, // 必传
}

/// 用户拷贝文件（在同一用户和应用下）
pub async fn file_copy(
    param: &FileCopyParam,
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
    let user_id = auth_data.user_id();
    super::app_check_get(param.app_id, true, &auth_data, req_dao, web_dao).await?;

    let file_dao = &web_dao.web_file.file_dao;

    // 验证文件属于当前用户和应用
    let file_user = file_dao
        .helper()
        .find_file_ref_by_id(param.file_ref_id)
        .await?
        .filter(|r| {
            r.user_id == user_id
                && r.app_id == param.app_id
                && r.status == FileUserStatus::Normal as i8
        })
        .ok_or_else(|| {
            crate::common::JsonError::Message(lsys_core::fluent_message!("file-not-found"))
        })?;

    // 获取源文件信息
    let file = file_dao
        .helper()
        .find_file_by_id(file_user.file_id)
        .await?
        .ok_or_else(|| {
            crate::common::JsonError::Message(lsys_core::fluent_message!("file-not-found"))
        })?;

    // 检查文件状态
    if !lsys_file::model::FileStatus::Normal.eq(file.status) {
        return Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("file-status-must-be-normal"),
        ));
    }

    let storage_type = param.storage_type.as_str();
    let ctx = file_dao.file_ops().create_context(&file_user);

    let (new_file, new_file_user) = if file.is_local() {
        // 源文件为本地文件：storage_type 可以是任意类型
        if lsys_file::model::FileModel::is_local_key(storage_type) {
            // 目标为本地类型
            if file.storage_type == storage_type {
                // 目标为同类型：使用引用模式（不拷贝文件）
                let ctx_with_file = ctx.with_file(&file)?;
                file_dao
                    .local_file_copy(
                        ctx_with_file,
                        storage_type,
                        lsys_file::dao::LocalFileCopyMode::Ref,
                        Some(&req_dao.req_env),
                    )
                    .await?
            } else {
                // 目标为本地非同类型：使用转换
                let ctx_with_file = ctx.with_file(&file)?;
                file_dao
                    .local_file_convert(ctx_with_file, storage_type, Some(&req_dao.req_env))
                    .await?
            }
        } else {
            // 目标为非本地类型（OSS）：上传到 OSS
            let ctx_with_file = ctx.with_file(&file)?;
            file_dao
                .sync_local_to_oss(ctx_with_file, storage_type, Some(&req_dao.req_env))
                .await?
        }
    } else {
        // 源文件为非本地文件（OSS）：storage_type 只能是本地3种类型
        if !lsys_file::model::FileModel::is_local_key(storage_type) {
            return Err(crate::common::JsonError::Message(
                lsys_core::fluent_message!(
                    "file-oss-to-oss-not-supported",
                    {"storage_type": storage_type}
                ),
            ));
        }
        // 同步到本地
        let ctx_with_file = ctx.with_file(&file)?;
        file_dao
            .sync_oss_to_local(ctx_with_file, storage_type, Some(&req_dao.req_env))
            .await?
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "file_id": new_file.id,
        "file_ref_id": new_file_user.id,
        "storage_type": new_file.storage_type,
        "file_name": new_file.origin_name,
        "file_size": new_file.file_size,
    }))))
}

