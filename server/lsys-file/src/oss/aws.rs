use std::pin::Pin;

use aws_sdk_s3::Client;
use serde::{Deserialize, Serialize};

use crate::common::{FileResult, OssObjectMeta, OssProvider, OssProviderConfig, OssResult, UploadFileInfo};
use crate::model::FileOssModel;

// ==================== 配置结构 ====================

/// AWS S3 provider 类型标识
pub const PROVIDER_TYPE: &str = "aws-s3";

/// AWS S3 配置（存储在 lsys-setting 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsOssConfig {
    /// 自定义 endpoint（MinIO / R2 等兼容 S3 协议的服务）
    #[serde(default)]
    pub endpoint: Option<String>,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    /// HTTP 请求超时（秒），默认 30
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

impl OssProviderConfig for AwsOssConfig {
    fn build_provider(
        self,
    ) -> Pin<Box<dyn std::future::Future<Output = FileResult<Box<dyn OssProvider>>> + Send>> {
        Box::pin(async move {
            let creds = aws_sdk_s3::config::Credentials::new(
                self.access_key.clone(),
                self.secret_key.clone(),
                None,
                None,
                "AwsOssProvider",
            );

            let timeout = std::time::Duration::from_secs(self.timeout_secs.unwrap_or(30));
            let timeout_cfg = aws_config::timeout::TimeoutConfig::builder()
                .operation_timeout(timeout)
                .connect_timeout(timeout)
                .build();

            let mut config_builder = aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(aws_config::Region::new(self.region.clone()))
                .credentials_provider(creds)
                .timeout_config(timeout_cfg);

            if let Some(ref ep) = self.endpoint {
                config_builder = config_builder.endpoint_url(ep);
            }

            let sdk_config = config_builder.load().await;
            let client = Client::new(&sdk_config);

            Ok(Box::new(AwsOssProvider {
                config: self,
                client,
            }) as Box<dyn OssProvider>)
        })
    }
}

// ==================== Provider 实现 ====================

pub struct AwsOssProvider {
    config: AwsOssConfig,
    client: Client,
}

impl OssProvider for AwsOssProvider {
    fn provider_type() -> &'static str {
        PROVIDER_TYPE
    }

    fn download_stream(
        &self,
        file_oss: &FileOssModel,
        offset: Option<u64>,
        length: Option<u64>,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = FileResult<crate::common::OssDownloadResult>>
                + Send
                + '_,
        >,
    > {
        let object_key = file_oss.object_key.clone();
        Box::pin(async move {
            let mut request = self
                .client
                .get_object()
                .bucket(&self.config.bucket)
                .key(&object_key);

            // AWS S3 支持 Range 请求
            if let Some(start) = offset {
                let range_value = if let Some(len) = length {
                    format!("bytes={}-{}", start, start + len - 1)
                } else {
                    format!("bytes={}-", start)
                };
                request = request.range(range_value);
            }

            let mut resp = request.send().await.map_err(|e| {
                crate::common::FileError::System(lsys_core::fluent_message!("aws-s3-error", e))
            })?;

            let stream = async_stream::try_stream! {
                let mut downloaded: u64 = 0;
                while let Some(bytes) = resp.body.try_next().await.map_err(|e| {
                    crate::common::FileError::System(lsys_core::fluent_message!("aws-s3-error", e))
                })? {
                    let chunk_size = bytes.len() as u64;
                    downloaded += chunk_size;
                    yield crate::common::OssDownloadChunk {
                        data: bytes,
                        offset: offset.map(|o| o + downloaded - chunk_size),
                        downloaded,
                    };
                }
            };

            // AWS S3 支持 Range 请求
            Ok(crate::common::OssDownloadResult::RangeSupported(
                Box::pin(stream) as crate::common::OssDownloadStream,
            ))
        })
    }

    fn upload_from_local(
        &self,
        local_path: &str,
        file_info: &UploadFileInfo<'_>,
    ) -> Pin<Box<dyn std::future::Future<Output = FileResult<OssResult>> + Send + '_>> {
        let local_path = local_path.to_string();
        let file_name = file_info.file_name.to_string();
        let file_md5 = file_info.file_md5.to_string();
        let file_size = file_info.file_size;
        let content_type = file_info.content_type.to_string();
        Box::pin(async move {
            let body = aws_sdk_s3::primitives::ByteStream::from_path(&local_path)
                .await
                .map_err(|e| {
                    crate::common::FileError::System(lsys_core::fluent_message!("file-io-error", e))
                })?;

            let ext = crate::common::extract_extension(Some(&file_name));
            let object_key = format!(
                "{}/{}{}",
                chrono::Local::now().format("%Y%m%d"),
                file_md5,
                if ext.is_empty() {
                    String::new()
                } else {
                    format!(".{}", ext)
                }
            );

            self.client
                .put_object()
                .bucket(&self.config.bucket)
                .key(&object_key)
                .content_type(&content_type)
                .body(body)
                .send()
                .await
                .map_err(|e| {
                    crate::common::FileError::System(lsys_core::fluent_message!("aws-s3-error", e))
                })?;

            let object_url = format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                self.config.bucket, self.config.region, object_key
            );

            Ok(OssResult {
                file_md5,
                object_key,
                bucket: self.config.bucket.clone(),
                object_url,
                content_type: Some(content_type),
                file_size: Some(file_size),
                modify_time: Some(chrono::Utc::now().timestamp() as u64),
                file_name: Some(file_name),
                region: Some(self.config.region.clone()),
            })
        })
    }

    fn delete_object(
        &self,
        file_oss: &FileOssModel,
    ) -> Pin<Box<dyn std::future::Future<Output = FileResult<()>> + Send + '_>> {
        let object_key = file_oss.object_key.clone();
        Box::pin(async move {
            self.client
                .delete_object()
                .bucket(&self.config.bucket)
                .key(&object_key)
                .send()
                .await
                .map_err(|e| {
                    crate::common::FileError::System(lsys_core::fluent_message!("aws-s3-error", e))
                })?;

            Ok(())
        })
    }

    fn object_meta(
        &self,
        file_oss: &FileOssModel,
    ) -> Pin<Box<dyn std::future::Future<Output = FileResult<OssObjectMeta>> + Send + '_>> {
        let object_key = file_oss.object_key.clone();
        Box::pin(async move {
            let result = self
                .client
                .head_object()
                .bucket(&self.config.bucket)
                .key(&object_key)
                .send()
                .await;

            let output = match result {
                Ok(o) => o,
                Err(e) => {
                    let not_found = e
                        .as_service_error()
                        .map(|se| se.is_not_found())
                        .unwrap_or(false);
                    if not_found {
                        return Ok(OssObjectMeta {
                            exists: false,
                            file_size: None,
                            content_md5: None,
                            content_type: None,
                            last_modified: None,
                        });
                    }
                    return Err(crate::common::FileError::System(
                        lsys_core::fluent_message!("aws-s3-error", e),
                    ));
                }
            };

            let file_size = output.content_length().map(|v| v as u64);
            // ETag = "hexmd5"；分片上传时含 '-'，不代表文件 MD5
            let content_md5 = output
                .e_tag()
                .map(|s| s.trim_matches('"').to_string())
                .filter(|s| !s.contains('-'));
            let content_type = output.content_type().map(|s| s.to_string());
            let last_modified = output
                .last_modified()
                .and_then(|dt| dt.to_millis().ok())
                .map(|ms| (ms as u64) / 1000);

            Ok(OssObjectMeta {
                exists: true,
                file_size,
                content_md5,
                content_type,
                last_modified,
            })
        })
    }
}
