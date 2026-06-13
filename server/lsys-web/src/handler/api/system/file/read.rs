use crate::common::{JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use lsys_access::dao::AccessSession;
use lsys_file::dao::UnifiedFileStream;
use lsys_file::model::{FileModel, FileRefModel};
use lsys_user::dao::UserAuthData;
use serde::Deserialize;

/// 管理员文件访问结果（URL 或流）
pub enum AdminFileAccessData {
    /// 公开文件：返回 URL 用于重定向
    Url(String),
    /// 私有文件：返回流用于内容输出
    Stream {
        ref_model: Box<FileRefModel>,
        file_model: Box<FileModel>,
        stream: UnifiedFileStream,
    },
}

/// 管理员文件访问参数
#[derive(Debug, Deserialize)]
pub struct AdminFileAccessParam {
    pub key: String,
}

/// 管理员文件访问接口（自动判断公开/私有）
///
/// - 公开文件：返回 URL（需要管理员认证）
/// - 私有文件：验证管理员权限后返回流
pub async fn admin_file_access<'a, F, Fut>(
    param: &AdminFileAccessParam,
    offset: u64,
    req_dao: &RequestDao,
    auth_callback: F,
    web_dao: &WebDao,
) -> JsonResult<AdminFileAccessData>
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


        // 管理员访问文件（公开/私有都需要认证）
        // 调用认证回调（设置 login token 并返回 auth_dao）
        let auth_dao = auth_callback().await?;
        
    

        // 获取用户认证信息
        let auth_data: UserAuthData = auth_dao
            .user_session
            .read()
            .await
            .get_session_data()
            .await?;

         // 验证管理员文件管理权限
        web_dao
            .web_rbac
            .check(
                &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
                &CheckAdminFileManage {},
            )
            .await?;

        // 私有文件：返回流
        let stream = web_dao
            .web_file.file_dao
            .cache()
            .read_file_stream(&file_model, offset, None)
            .await?;

        Ok(AdminFileAccessData::Stream {
            ref_model: Box::new(ref_model),
            file_model: Box::new(file_model),
            stream,
        })
    } else {
        // 公开文件：返回 URL
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

        Ok(AdminFileAccessData::Url(url))
    }
}
