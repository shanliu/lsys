use super::FileResult;
use crate::model::FileOssModel;
use bytes::Bytes;
use futures_util::Stream;

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
}

/// 上传本地文件到 OSS 时所需的文件元数据
#[derive(Debug, Clone)]
pub struct UploadFileInfo<'a> {
    pub file_name: &'a str,
    pub file_md5: &'a str,
    pub file_size: u64,
    pub content_type: &'a str,
}

/// OSS 下载流式数据块
#[derive(Debug, Clone)]
pub struct OssDownloadChunk {
    /// 数据块内容
    pub data: Bytes,
    /// 当前块在文件中的偏移（如果支持）
    pub offset: Option<u64>,
    /// 已下载的总字节数
    pub downloaded: u64,
}

/// OSS 流式下载器
///
/// 用于边下载边读取 OSS 文件内容。
/// 可以直接输出到用户端，也可以写入本地文件。
pub type OssDownloadStream =
    std::pin::Pin<Box<dyn Stream<Item = FileResult<OssDownloadChunk>> + Send>>;

/// OSS 下载结果
///
/// 表示 OSS provider 是否支持偏移读取
pub enum OssDownloadResult {
    /// 支持偏移读取，返回从指定位置开始的流
    RangeSupported(OssDownloadStream),
    /// 不支持偏移读取，返回完整文件流（忽略 offset/length 参数）
    FullStreamOnly(OssDownloadStream),
}

/// OSS 对象元数据（来自 HEAD 请求）
#[derive(Debug, Clone)]
pub struct OssObjectMeta {
    /// 对象是否存在
    pub exists: bool,
    /// 文件大小（字节）
    pub file_size: Option<u64>,
    /// 内容 MD5（仅简单上传时可用；分片上传的 ETag 含 '-'，不代表文件 MD5，此时为 None）
    pub content_md5: Option<String>,
    /// 内容类型
    pub content_type: Option<String>,
    /// 最后修改时间（Unix 时间戳，秒）
    pub last_modified: Option<u64>,
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

    /// 流式下载 OSS 文件
    ///
    /// 返回一个异步流，可以边下载边处理数据。
    ///
    /// # 使用场景
    /// 1. 直接输出到用户端（HTTP 响应流）
    /// 2. 保存到本地文件（配合 tokio::fs::File 写入）
    /// 3. 边下载边处理（如计算 MD5、转码等）
    ///
    /// # 参数
    /// - `file_oss`: OSS 文件模型
    /// - `offset`: 起始偏移（字节），None 表示从头开始
    /// - `length`: 读取长度（字节），None 表示读取到文件末尾
    ///
    /// # 返回
    /// 返回 `OssDownloadResult` 枚举：
    /// - `RangeSupported`: 支持偏移读取，返回从指定位置开始的流
    /// - `FullStreamOnly`: 不支持偏移读取，返回完整文件流（忽略 offset/length）
    ///
    /// # 示例 1: 直接输出到用户端
    /// ```rust,ignore
    /// use futures_util::StreamExt;
    ///
    /// let result = provider.download_stream(&file_oss, None, None).await?;
    /// let mut stream = match result {
    ///     OssDownloadResult::RangeSupported(s) => s,
    ///     OssDownloadResult::FullStreamOnly(s) => s,
    /// };
    /// while let Some(result) = stream.next().await {
    ///     let chunk = result?;
    ///     response.write_all(&chunk.data).await?;
    /// }
    /// ```
    ///
    /// # 示例 2: 保存到本地文件
    /// ```rust,ignore
    /// use futures_util::StreamExt;
    /// use tokio::fs::File;
    /// use tokio::io::AsyncWriteExt;
    ///
    /// let result = provider.download_stream(&file_oss, None, None).await?;
    /// let mut stream = match result {
    ///     OssDownloadResult::RangeSupported(s) => s,
    ///     OssDownloadResult::FullStreamOnly(s) => s,
    /// };
    /// let mut file = File::create("/tmp/downloaded.dat").await?;
    ///
    /// while let Some(result) = stream.next().await {
    ///     let chunk = result?;
    ///     file.write_all(&chunk.data).await?;
    /// }
    /// file.flush().await?;
    /// ```
    ///
    /// # 示例 3: 处理不支持 Range 的 OSS
    /// ```rust,ignore
    /// use futures_util::StreamExt;
    ///
    /// let offset = 1024;
    /// let length = Some(2048);
    ///
    /// let result = provider.download_stream(&file_oss, Some(offset), length).await?;
    /// match result {
    ///     OssDownloadResult::RangeSupported(mut stream) => {
    ///         // 直接使用偏移下载的流
    ///         while let Some(result) = stream.next().await {
    ///             let chunk = result?;
    ///             // 处理数据...
    ///         }
    ///     }
    ///     OssDownloadResult::FullStreamOnly(mut stream) => {
    ///         // 需要在客户端跳过前面的数据
    ///         let mut skipped = 0u64;
    ///         while let Some(result) = stream.next().await {
    ///             let chunk = result?;
    ///             if skipped < offset {
    ///                 let skip_in_chunk = std::cmp::min(chunk.data.len() as u64, offset - skipped);
    ///                 skipped += skip_in_chunk;
    ///                 // 处理剩余数据...
    ///             } else {
    ///                 // 处理数据...
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    fn download_stream(
        &self,
        file_oss: &FileOssModel,
        offset: Option<u64>,
        length: Option<u64>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = FileResult<OssDownloadResult>> + Send + '_>,
    >;

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

    /// 获取 OSS 对象元数据（HEAD 请求）
    ///
    /// - 对象不存在时返回 `OssObjectMeta { exists: false, .. }`，不返回 Err。
    /// - `content_md5` 仅在简单上传时可用（ETag = MD5 hex）；
    ///   分片上传时 ETag 格式为 `"hexmd5-partcount"`，此字段设为 `None`。
    fn object_meta(
        &self,
        file_oss: &FileOssModel,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FileResult<OssObjectMeta>> + Send + '_>>;
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
