use std::pin::Pin;
use std::time::Duration;

use chrono::Utc;
use futures_util::StreamExt;
use hmac::{Hmac, KeyInit, Mac};
use reqwest::{Client, Method, header};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::common::{FileResult, OssObjectMeta, OssProvider, OssProviderConfig, OssResult, UploadFileInfo};
use crate::model::FileOssModel;

type HmacSha1 = Hmac<Sha1>;

const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// 腾讯云 COS provider 类型标识
pub const PROVIDER_TYPE: &str = "tencent-cos";

// ==================== 配置结构 ====================

/// 腾讯云 COS 配置（存储在 lsys-setting 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TencentCosConfig {
    /// e.g. "cos.ap-guangzhou.myqcloud.com"
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    /// HTTP 请求超时（秒），默认 30
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

impl OssProviderConfig for TencentCosConfig {
    fn build_provider(
        self,
    ) -> Pin<Box<dyn std::future::Future<Output = FileResult<Box<dyn OssProvider>>> + Send>> {
        Box::pin(async move {
            let timeout = Duration::from_secs(self.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS));
            let client = Client::builder().timeout(timeout).build().map_err(|e| {
                crate::common::FileError::System(lsys_core::fluent_message!("tencent-cos-error", e))
            })?;
            Ok(Box::new(TencentCosProvider {
                config: self,
                client,
            }) as Box<dyn OssProvider>)
        })
    }
}

// ==================== Provider 实现 ====================

pub struct TencentCosProvider {
    config: TencentCosConfig,
    client: Client,
}

impl TencentCosProvider {
    fn sign(&self, method: &Method, uri: &str) -> crate::common::FileResult<String> {
        let now = Utc::now().timestamp();
        let expire = now + 3600; // 1 hour expiration
        let q_key_time = format!("{};{}", now, expire);
        let q_sign_time = format!("{};{}", now, expire);
        let q_sign_algorithm = "sha1";

        let mut mac1 =
            HmacSha1::new_from_slice(self.config.secret_key.as_bytes()).map_err(|_| {
                crate::common::FileError::System(lsys_core::fluent_message!(
                    "tencent-cos-error",
                    "Invalid HMAC key length"
                ))
            })?;
        mac1.update(q_key_time.as_bytes());
        let sign_key = hex::encode(mac1.finalize().into_bytes());

        let http_method = method.as_str().to_lowercase();
        let http_uri = if !uri.starts_with('/') {
            format!("/{}", uri)
        } else {
            uri.to_string()
        };
        let http_parameters = "";
        let http_headers = "host";

        let http_string = format!(
            "{}\n{}\n{}\n{}\n",
            http_method, http_uri, http_parameters, http_headers
        );
        let mut hasher = Sha1::new();
        hasher.update(http_string.as_bytes());
        let sha1_http_string = hex::encode(hasher.finalize());

        let string_to_sign = format!(
            "{}\n{}\n{}\n",
            q_sign_algorithm, q_sign_time, sha1_http_string
        );

        let mut mac2 = HmacSha1::new_from_slice(sign_key.as_bytes()).map_err(|_| {
            crate::common::FileError::System(lsys_core::fluent_message!(
                "tencent-cos-error",
                "Invalid HMAC key length"
            ))
        })?;
        mac2.update(string_to_sign.as_bytes());
        let q_signature = hex::encode(mac2.finalize().into_bytes());

        Ok(format!(
            "q-sign-algorithm={}&q-ak={}&q-sign-time={}&q-key-time={}&q-header-list=host&q-url-param-list=&q-signature={}",
            q_sign_algorithm, self.config.access_key, q_sign_time, q_key_time, q_signature
        ))
    }
}

impl OssProvider for TencentCosProvider {
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
            let signature = self.sign(&Method::GET, &object_key)?;
            let url = format!(
                "https://{}.{}/{}",
                self.config.bucket, self.config.endpoint, object_key
            );
            let host_header = format!("{}.{}", self.config.bucket, self.config.endpoint);

            let mut request = self
                .client
                .get(&url)
                .header(header::HOST, host_header)
                .header(header::AUTHORIZATION, signature);

            // 腾讯云 COS 支持 Range 请求
            if let Some(start) = offset {
                let range_value = if let Some(len) = length {
                    format!("bytes={}-{}", start, start + len - 1)
                } else {
                    format!("bytes={}-", start)
                };
                request = request.header(header::RANGE, range_value);
            }

            let response = request.send().await.map_err(|e| {
                crate::common::FileError::System(lsys_core::fluent_message!("tencent-cos-error", e))
            })?;

            // 支持 200 (完整响应) 和 206 (部分内容)
            let status = response.status();
            if !status.is_success() && status.as_u16() != 206 {
                return Err(crate::common::FileError::System(
                    lsys_core::fluent_message!(
                        "tencent-cos-error",
                        format!("download failed with status: {}", status)
                    ),
                ));
            }

            let byte_stream = response.bytes_stream();

            let stream = async_stream::try_stream! {
                tokio::pin!(byte_stream);
                let mut downloaded: u64 = 0;
                while let Some(bytes) = byte_stream.next().await {
                    let bytes = bytes.map_err(|e| {
                        crate::common::FileError::System(
                            lsys_core::fluent_message!("tencent-cos-error", e)
                        )
                    })?;
                    let chunk_size = bytes.len() as u64;
                    downloaded += chunk_size;
                    yield crate::common::OssDownloadChunk {
                        data: bytes,
                        offset: offset.map(|o| o + downloaded - chunk_size),
                        downloaded,
                    };
                }
            };

            // 腾讯云 COS 支持 Range 请求
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
            let file = tokio::fs::File::open(&local_path).await.map_err(|e| {
                crate::common::FileError::System(lsys_core::fluent_message!("file-io-error", e))
            })?;
            let stream = reqwest::Body::wrap_stream(tokio_util::io::ReaderStream::new(file));

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

            let signature = self.sign(&Method::PUT, &object_key)?;
            let url = format!(
                "https://{}.{}/{}",
                self.config.bucket, self.config.endpoint, object_key
            );
            let host_header = format!("{}.{}", self.config.bucket, self.config.endpoint);

            let response = self
                .client
                .put(&url)
                .header(header::HOST, host_header)
                .header(header::CONTENT_TYPE, &content_type)
                .header(header::AUTHORIZATION, signature)
                .body(stream)
                .send()
                .await
                .map_err(|e| {
                    crate::common::FileError::System(lsys_core::fluent_message!(
                        "tencent-cos-error",
                        e
                    ))
                })?;

            if !response.status().is_success() {
                return Err(crate::common::FileError::System(
                    lsys_core::fluent_message!(
                        "tencent-cos-error",
                        format!("upload failed with status: {}", response.status())
                    ),
                ));
            }

            Ok(OssResult {
                file_md5,
                object_key: object_key.clone(),
                bucket: self.config.bucket.clone(),
                object_url: url,
                content_type: Some(content_type),
                file_size: Some(file_size),
                modify_time: Some(chrono::Utc::now().timestamp() as u64),
                file_name: Some(file_name),
                region: Some(self.config.endpoint.clone()),
            })
        })
    }

    fn delete_object(
        &self,
        file_oss: &FileOssModel,
    ) -> Pin<Box<dyn std::future::Future<Output = FileResult<()>> + Send + '_>> {
        let object_key = file_oss.object_key.clone();
        Box::pin(async move {
            let signature = self.sign(&Method::DELETE, &object_key)?;
            let url = format!(
                "https://{}.{}/{}",
                self.config.bucket, self.config.endpoint, object_key
            );
            let host_header = format!("{}.{}", self.config.bucket, self.config.endpoint);

            let response = self
                .client
                .delete(&url)
                .header(header::HOST, host_header)
                .header(header::AUTHORIZATION, signature)
                .send()
                .await
                .map_err(|e| {
                    crate::common::FileError::System(lsys_core::fluent_message!(
                        "tencent-cos-error",
                        e
                    ))
                })?;

            if !response.status().is_success() {
                return Err(crate::common::FileError::System(
                    lsys_core::fluent_message!(
                        "tencent-cos-error",
                        format!("delete failed with status: {}", response.status())
                    ),
                ));
            }

            Ok(())
        })
    }

    fn object_meta(
        &self,
        file_oss: &FileOssModel,
    ) -> Pin<Box<dyn std::future::Future<Output = FileResult<OssObjectMeta>> + Send + '_>> {
        let object_key = file_oss.object_key.clone();
        Box::pin(async move {
            let signature = self.sign(&Method::HEAD, &object_key)?;
            let url = format!(
                "https://{}.{}/{}",
                self.config.bucket, self.config.endpoint, object_key
            );
            let host_header = format!("{}.{}", self.config.bucket, self.config.endpoint);

            let response = self
                .client
                .head(&url)
                .header(header::HOST, host_header)
                .header(header::AUTHORIZATION, signature)
                .send()
                .await
                .map_err(|e| {
                    crate::common::FileError::System(lsys_core::fluent_message!(
                        "tencent-cos-error",
                        e
                    ))
                })?;

            let status = response.status();
            if status.as_u16() == 404 {
                return Ok(OssObjectMeta {
                    exists: false,
                    file_size: None,
                    content_md5: None,
                    content_type: None,
                    last_modified: None,
                });
            }
            if !status.is_success() {
                return Err(crate::common::FileError::System(
                    lsys_core::fluent_message!(
                        "tencent-cos-error",
                        format!("head object failed with status: {}", status)
                    ),
                ));
            }

            let headers = response.headers();
            let file_size = headers
                .get(header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());
            // ETag = "hexmd5"；分片上传时含 '-'，不代表文件 MD5
            let content_md5 = headers
                .get(header::ETAG)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim_matches('"').to_string())
                .filter(|s| !s.contains('-'));
            let content_type = headers
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let last_modified = headers
                .get(header::LAST_MODIFIED)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| chrono::DateTime::parse_from_rfc2822(s).ok())
                .map(|dt| dt.timestamp() as u64);

            Ok(OssObjectMeta {
                exists: true,
                file_size,
                content_md5,
                content_type,
                last_modified,
            })        })
    }
}