use std::pin::Pin;
use std::time::Duration;

use reqwest::{Client, Method, header};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use chrono::Utc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use crate::common::{FileResult, OssProvider, OssProviderConfig, OssResult, UploadFileInfo};
use crate::model::FileOssModel;

type HmacSha1 = Hmac<Sha1>;

const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// 阿里云 OSS provider 类型标识
pub const PROVIDER_TYPE: &str = "aliyun-oss";

// ==================== 配置结构 ====================

/// 阿里云 OSS 配置（存储在 lsys-setting 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliyunOssConfig {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    /// HTTP 请求超时（秒），默认 30
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

impl OssProviderConfig for AliyunOssConfig {
    fn build_provider(
        self,
    ) -> Pin<Box<dyn std::future::Future<Output = FileResult<Box<dyn OssProvider>>> + Send>>
    {
        Box::pin(async move {
            let timeout = Duration::from_secs(self.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS));
            let client = Client::builder()
                .timeout(timeout)
                .build()
                .map_err(|e| crate::common::FileError::System(lsys_core::fluent_message!("aliyun-oss-error", e)))?;
            Ok(Box::new(AliyunOssProvider { config: self, client }) as Box<dyn OssProvider>)
        })
    }
}

// ==================== Provider 实现 ====================

pub struct AliyunOssProvider {
    config: AliyunOssConfig,
    client: Client,
}

impl AliyunOssProvider {
    fn gmt_date() -> String {
        Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    }

    fn sign(&self, method: &Method, date: &str, content_type: &str, object_key: &str) -> String {
        let string_to_sign = format!(
            "{}\n\n{}\n{}\n/{}/{}",
            method.as_str(),
            content_type,
            date,
            self.config.bucket,
            object_key
        );

        let mut mac = HmacSha1::new_from_slice(self.config.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(string_to_sign.as_bytes());
        let result = mac.finalize();
        BASE64.encode(result.into_bytes())
    }
}

impl OssProvider for AliyunOssProvider {
    fn provider_type() -> &'static str {
        PROVIDER_TYPE
    }

    fn download_to_local(
        &self,
        file_oss: &FileOssModel,
        local_path: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = FileResult<()>> + Send + '_>> {
        let local_path = local_path.to_string();
        let object_key = file_oss.object_key.clone();
        Box::pin(async move {
            let date = Self::gmt_date();
            let signature = self.sign(&Method::GET, &date, "", &object_key);
            let auth_header = format!("OSS {}:{}", self.config.access_key, signature);
            let url = format!("https://{}.{}/{}", self.config.bucket, self.config.endpoint, object_key);

            let response = self.client.get(&url)
                .header(header::DATE, date)
                .header(header::AUTHORIZATION, auth_header)
                .send()
                .await
                .map_err(|e| crate::common::FileError::System(lsys_core::fluent_message!("aliyun-oss-error", e)))?;

            if !response.status().is_success() {
                return Err(crate::common::FileError::System(lsys_core::fluent_message!("aliyun-oss-error", format!("download failed with status: {}", response.status()))));
            }

            let mut file = File::create(&local_path)
                .await
                .map_err(|e| crate::common::FileError::System(lsys_core::fluent_message!("file-io-error", e)))?;

            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let bytes = chunk.map_err(|e| crate::common::FileError::System(lsys_core::fluent_message!("aliyun-oss-error", e)))?;
                file.write_all(&bytes).await
                    .map_err(|e| crate::common::FileError::System(lsys_core::fluent_message!("file-io-error", e)))?;
            }

            file.flush().await
                .map_err(|e| crate::common::FileError::System(lsys_core::fluent_message!("file-io-error", e)))?;

            Ok(())
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
            let file = File::open(&local_path).await
                .map_err(|e| crate::common::FileError::System(lsys_core::fluent_message!("file-io-error", e)))?;
            let stream = reqwest::Body::wrap_stream(tokio_util::io::ReaderStream::new(file));

            let date = Self::gmt_date();
            let ext = crate::common::extract_extension(Some(&file_name));
            let object_key = format!(
                "{}/{}{}",
                chrono::Local::now().format("%Y/%m/%d"),
                file_md5,
                if ext.is_empty() { String::new() } else { format!(".{}", ext) }
            );

            let signature = self.sign(&Method::PUT, &date, &content_type, &object_key);
            let auth_header = format!("OSS {}:{}", self.config.access_key, signature);
            let url = format!("https://{}.{}/{}", self.config.bucket, self.config.endpoint, object_key);

            let response = self.client.put(&url)
                .header(header::DATE, date)
                .header(header::CONTENT_TYPE, &content_type)
                .header(header::AUTHORIZATION, auth_header)
                .body(stream)
                .send()
                .await
                .map_err(|e| crate::common::FileError::System(lsys_core::fluent_message!("aliyun-oss-error", e)))?;

            if !response.status().is_success() {
                return Err(crate::common::FileError::System(lsys_core::fluent_message!("aliyun-oss-error", format!("upload failed with status: {}", response.status()))));
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
                local_file_id: None,
                source_url: None,
            })
        })
    }

    fn delete_object(
        &self,
        file_oss: &FileOssModel,
    ) -> Pin<Box<dyn std::future::Future<Output = FileResult<()>> + Send + '_>> {
        let object_key = file_oss.object_key.clone();
        Box::pin(async move {
            let date = Self::gmt_date();
            let signature = self.sign(&Method::DELETE, &date, "", &object_key);
            let auth_header = format!("OSS {}:{}", self.config.access_key, signature);
            let url = format!("https://{}.{}/{}", self.config.bucket, self.config.endpoint, object_key);

            let response = self.client.delete(&url)
                .header(header::DATE, date)
                .header(header::AUTHORIZATION, auth_header)
                .send()
                .await
                .map_err(|e| crate::common::FileError::System(lsys_core::fluent_message!("aliyun-oss-error", e)))?;

            if !response.status().is_success() {
                return Err(crate::common::FileError::System(lsys_core::fluent_message!("aliyun-oss-error", format!("delete failed with status: {}", response.status()))));
            }

            Ok(())
        })
    }
}
