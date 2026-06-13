use tracing::{debug, info, warn};

use crate::common::{FileError, FileResult};

/// URL 文件信息结构体
#[derive(Debug, Clone)]
pub struct UrlFileInfo {
    /// 文件总大小（字节），None 表示服务器未返回
    pub file_size: Option<u64>,
    /// 是否支持 206 Range 范围请求（可分片下载）
    pub supports_range: bool,
    /// 文件 MIME 类型（可选）
    pub content_type: Option<String>,
}

/// 206 探测的读取字节数
const PROBE_RANGE_BYTES: u64 = 1024;

impl super::FileHelper {
    /// 从 URL 中提取文件名
    pub fn extract_filename_from_url(url: &str) -> String {
        // 去掉查询参数和片段
        let path = url.split('?').next().unwrap_or(url);
        let path = path.split('#').next().unwrap_or(path);
        // 获取最后一个路径段
        if let Some(last_slash) = path.rfind('/') {
            let name = &path[last_slash + 1..];
            if !name.is_empty() {
                return name.to_string();
            }
        }
        String::new()
    }

    /// 获取URL文件信息函数
    ///
    /// 参数: URL
    /// 使用配置中的 download_timeout_secs
    ///
    /// 返回: 文件信息（supports_range 已根据 206 探测结果确定）
    pub async fn get_url_file_info(&self, url: &str) -> FileResult<UrlFileInfo> {
        let timeout_secs = self
            .runtime_setting
            .get_download_timeout_secs()
            .await
            .unwrap_or(60);

        let client = reqwest::Client::new();
        let mut current_url = url.to_string();
        let mut redirect_count = 0;
        const MAX_REDIRECTS: usize = 5;

        info!("get_url_file_info: probing url={}", url);

        loop {
            debug!("get_url_file_info: sending HEAD request to {}", current_url);
            let response = client
                .head(&current_url)
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .send()
                .await
                .map_err(|e| {
                    warn!("get_url_file_info: HEAD request failed: {}", e);
                    FileError::Http(e.to_string())
                })?;

            let status = response.status().as_u16();
            debug!("get_url_file_info: HEAD response status={}", status);

            // 处理重定向 (301, 302)
            if status == 301 || status == 302 {
                if redirect_count >= MAX_REDIRECTS {
                    warn!("get_url_file_info: redirect limit exceeded");
                    return Err(FileError::RedirectLimitExceeded);
                }

                let location = response
                    .headers()
                    .get("location")
                    .and_then(|h| h.to_str().ok())
                    .ok_or_else(|| FileError::Http("Missing location header".to_string()))?;

                info!(
                    "get_url_file_info: redirect {} -> {}",
                    current_url, location
                );
                current_url = location.to_string();
                redirect_count += 1;
                continue;
            }

            // 只接受 200 状态码
            if status != 200 {
                warn!("get_url_file_info: invalid status code {}", status);
                return Err(FileError::InvalidStatusCode(status));
            }

            // 获取文件大小（如果服务器未返回，则为 None）
            let file_size = response
                .headers()
                .get("content-length")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            debug!("get_url_file_info: content-length={:?}", file_size);

            // 判断是否支持下载偏移值（检查 Accept-Ranges 头）
            let accepts_ranges_header = response
                .headers()
                .get("accept-ranges")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_lowercase() != "none")
                .unwrap_or(false);

            debug!(
                "get_url_file_info: accept-ranges header={}",
                accepts_ranges_header
            );

            // 实际发送 206 Range 请求探测是否真正支持分片下载
            let supports_range = if accepts_ranges_header && file_size.is_some_and(|s| s > 0) {
                Self::probe_range_support(
                    &client,
                    &current_url,
                    file_size.unwrap_or(0),
                    timeout_secs,
                )
                .await
            } else {
                false
            };

            info!(
                "get_url_file_info: 206 probe result supports_range={}",
                supports_range
            );

            // 获取内容类型（可选）
            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());

            info!(
                "get_url_file_info: completed file_size={:?}, supports_range={}, content_type={:?}",
                file_size, supports_range, content_type
            );

            return Ok(UrlFileInfo {
                file_size,
                supports_range,
                content_type,
            });
        }
    }

    /// 探测 URL 是否真正支持 206 Range 请求
    ///
    /// 发送一个小范围的 Range 请求, 验证服务器是否返回 206 状态码
    async fn probe_range_support(
        client: &reqwest::Client,
        url: &str,
        file_size: u64,
        timeout_secs: u64,
    ) -> bool {
        // 计算探测范围: 读取前 PROBE_RANGE_BYTES 字节或整个文件(如果较小)
        let end_byte = std::cmp::min(PROBE_RANGE_BYTES, file_size).saturating_sub(1);
        let range_header = format!("bytes=0-{}", end_byte);

        debug!(
            "probe_range_support: sending Range request to {} with header: {}",
            url, range_header
        );

        let response = match client
            .get(url)
            .header("Range", &range_header)
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!("probe_range_support: Range request failed: {}", e);
                return false;
            }
        };

        let status = response.status().as_u16();
        debug!("probe_range_support: Range response status={}", status);

        // 206 Partial Content 表示服务器支持 Range 请求
        if status == 206 {
            // 额外验证: 检查 Content-Range 头是否存在
            let has_content_range = response.headers().get("content-range").is_some();
            if has_content_range {
                info!(
                    "probe_range_support: server supports Range requests (206 with Content-Range)"
                );
                return true;
            } else {
                warn!("probe_range_support: 206 received but no Content-Range header");
                return false;
            }
        }

        // 200 表示服务器忽略了 Range 请求, 返回完整内容
        if status == 200 {
            info!("probe_range_support: server returned 200, Range not supported");
            return false;
        }

        warn!(
            "probe_range_support: unexpected status {} for Range request",
            status
        );
        false
    }
}
