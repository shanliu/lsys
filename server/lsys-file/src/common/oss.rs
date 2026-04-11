use super::FileResult;
use crate::model::FileOssModel;

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

/// 上传本地文件到 OSS 时所需的文件元数据
#[derive(Debug, Clone)]
pub struct UploadFileInfo<'a> {
    pub file_name: &'a str,
    pub file_md5: &'a str,
    pub file_size: u64,
    pub content_type: &'a str,
}

/// OSS 云服务 trait: 各实现(阿里云/腾讯云等)需要实现此 trait
pub trait OssProvider: Send + Sync {
    /// provider 类型标识，如 "aliyun-oss", "aws-s3", "tencent-cos"
    ///
    /// 静态方法，不需要实例即可调用（通过具体类型）。
    /// 不可通过 `dyn OssProvider` 调用，运行时可用模块常量替代。
    fn provider_type() -> &'static str
    where
        Self: Sized;

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
        file_info: &UploadFileInfo<'_>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FileResult<OssResult>> + Send + '_>>;

    /// 删除 OSS 对象
    fn delete_object(
        &self,
        file_oss: &FileOssModel,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FileResult<()>> + Send + '_>>;
}

/// OSS 配置 trait: 每种 provider 的配置结构需实现此 trait
///
/// `build_provider(self)` 消费配置本身，直接移入 Provider 内部，
/// 避免逐字段 clone。调用方若需保留配置副本请提前 `.clone()`。
///
/// 各 provider 的 Config 结构放在 `oss/` 对应文件中，
/// 与 OssProvider 实现紧挨着，新增厂商只需新增一个文件。
///
/// OssProvider 构建结果的 Future 类型别名
pub type OssProviderFuture =
    std::pin::Pin<Box<dyn std::future::Future<Output = FileResult<Box<dyn OssProvider>>> + Send>>;

pub trait OssProviderConfig: Send {
    /// 消费配置，构造 OssProvider 实例
    fn build_provider(self) -> OssProviderFuture;
}
