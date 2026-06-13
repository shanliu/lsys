use crate::common::{JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileView;
use lsys_access::dao::AccessSession;
use lsys_file::dao::UnifiedFileStream;
use lsys_file::model::{FileModel, FileRefModel};
use lsys_user::dao::UserAuthData;
use serde::Deserialize;

/// 应用文件访问结果（URL 或流）
pub enum AppFileAccessData {
    /// 公开文件：返回 URL 用于重定向
    Url(String),
    /// 私有文件：返回流用于内容输出
    Stream {
        ref_model: Box<FileRefModel>,
        file_model: Box<FileModel>,
        stream: UnifiedFileStream,
    },
}

/// 应用文件访问参数
#[derive(Debug, Deserialize)]
pub struct AppFileAccessParam {
    pub key: String,
}

/// 应用文件访问接口（自动判断公开/私有）
///
/// - 公开文件：返回 URL（无需认证）
/// - 私有文件：验证权限后返回流
pub async fn app_file_access<'a, F, Fut>(
    param: &AppFileAccessParam,
    offset: u64,
    req_dao: &RequestDao,
    private_auth: F,
    web_dao: &WebDao,
) -> JsonResult<AppFileAccessData>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = JsonResult<&'a UserAuthQueryDao>>,
{
    // 解码 key 获取 ref_id
    let ref_id = web_dao
        .web_file.file_dao
        .file_key_encoder()
        .decode(&param.key)?;

    // 获取 file_ref 信息
    let ref_model = web_dao
        .web_file.file_dao
        .cache()
        .find_file_ref_by_id(ref_id)
        .await?;

    // 检查文件状态
    if ref_model.status != 1 {
        return Err(lsys_file::common::FileError::Param(
            lsys_core::fluent_message!("file-unavailable"),
        )
        .into());
    }

    // 获取 file 信息
    let file_model = web_dao
        .web_file.file_dao
        .cache()
        .find_file_by_id(ref_model.file_id)
        .await?;

    // 检查文件是否为私有
    let is_private = web_dao
        .web_file.file_dao
        .cache()
        .is_private(&file_model)
        .await?;

    if is_private {
        // 私有文件：验证权限后返回流
        
        // 调用认证回调（设置 login token 并返回 auth_dao）
        let auth_dao = private_auth().await?;
        
        // 获取用户认证信息
        let auth_data: UserAuthData = auth_dao
            .user_session
            .read()
            .await
            .get_session_data()
            .await?;

        // 验证app权限（使用 ref_model 中的 app_id）
        super::app_check_get(ref_model.app_id, false, &auth_data, req_dao, web_dao).await?;

        // 验证用户对文件资源的访问权限
        web_dao
            .web_rbac
            .check(
                &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
                &CheckUserFileView {
                    res_user_id: ref_model.user_id,
                },
            )
            .await?;

        // 读取文件流
        let stream = web_dao
            .web_file.file_dao
            .cache()
            .read_file_stream(&file_model, offset, None)
            .await?;

        Ok(AppFileAccessData::Stream {
            ref_model: Box::new(ref_model),
            file_model: Box::new(file_model),
            stream,
        })
    } else {
        // 公开文件：生成 URL 并返回（无需认证）
        let url = web_dao
            .web_file.file_dao
            .cache()
            .get_file_url(&file_model)
            .await?
            .ok_or_else(|| {
                lsys_file::common::FileError::Param(lsys_core::fluent_message!(
                    "file-url-not-available"
                ))
            })?;

        Ok(AppFileAccessData::Url(url))
    }
}

/// 公开文件分享访问（仅支持公开文件，无需认证）
///
/// - 公开文件：返回 URL
/// - 私有文件：返回 Err
pub async fn app_file_share(
    key: &str,
    web_dao: &WebDao,
) -> JsonResult<String> {
    let ref_id = web_dao
        .web_file.file_dao
        .file_key_encoder()
        .decode(key)?;

    let ref_model = web_dao
        .web_file.file_dao
        .cache()
        .find_file_ref_by_id(ref_id)
        .await?;

    if ref_model.status != 1 {
        return Err(lsys_file::common::FileError::Param(
            lsys_core::fluent_message!("file-unavailable"),
        )
        .into());
    }

    let file_model = web_dao
        .web_file.file_dao
        .cache()
        .find_file_by_id(ref_model.file_id)
        .await?;

    let is_private = web_dao
        .web_file.file_dao
        .cache()
        .is_private(&file_model)
        .await?;

    if is_private {
        return Err(lsys_file::common::FileError::Param(
            lsys_core::fluent_message!("file-access-denied"),
        )
        .into());
    }

    let url = web_dao
        .web_file.file_dao
        .cache()
        .get_file_url(&file_model)
        .await?
        .ok_or_else(|| {
            lsys_file::common::FileError::Param(lsys_core::fluent_message!(
                "file-url-not-available"
            ))
        })?;

    Ok(url)
}
