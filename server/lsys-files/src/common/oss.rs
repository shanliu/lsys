use super::FileResult;
use crate::model::{FileModel, FileOssModel};

/// OSS 结果参数
#[derive(Debug, Clone)]
pub struct OssResult {
    pub file_md5: String,
    pub object_key: String,
    pub bucket: String,
    pub object_url: String,
    pub content_type: Option<String>,
    pub file_size: Option<u64>,
    pub modify_time: Option<u64>,
    pub file_name: Option<String>,
    pub region: Option<String>,
    pub local_file_id: Option<u64>,
    pub source_url: Option<String>,
}

/// OSS 云服务 trait: 各实现(阿里云/腾讯云等)需要实现此 trait
pub trait OssProvider: Send + Sync {
    /// 从 OSS 下载文件到本地路径
    fn download_to_local(
        &self,
        file_oss: &FileOssModel,
        local_path: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FileResult<()>> + Send + '_>>;

    /// 上传本地文件到 OSS, 返回 OssResult
    fn upload_from_local(
        &self,
        local_path: &str,
        file: &FileModel,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FileResult<OssResult>> + Send + '_>>;

    /// 删除 OSS 对象
    fn delete_object(
        &self,
        file_oss: &FileOssModel,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FileResult<()>> + Send + '_>>;
}
