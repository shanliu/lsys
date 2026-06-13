use std::sync::Arc;

use lsys_setting::dao::{
    SettingDecode, SettingEncode, SettingKey, SingleSetting, SingleSettingData,
};
use serde::{Deserialize, Serialize};

use crate::common::{FileError, FileResult};

// ==================== 配置数据结构 ====================

/// 文件服务运行时配置（存储在数据库中，可动态调整）
///
/// 这些配置可以在运行时通过 API 修改，无需重启服务。
/// 适用于需要根据实际运行环境动态调整的参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRuntimeSettingData {
    /// 本地公开文件 URL 前缀
    ///
    /// 支持配置 CDN 地址，例如：
    /// - 本地路径: "/files/" 或 "http://localhost/files/"
    /// - CDN 地址: "https://cdn.example.com/files/"
    ///
    /// 注意：URL 前缀应该以 "/" 结尾
    #[serde(default = "default_url_prefix")]
    pub local_public_url_prefix: String,

    /// 最大下载并发数
    ///
    /// 控制同时进行的文件下载任务数量。
    /// 可以根据服务器性能和网络带宽动态调整。
    #[serde(default = "default_max_concurrency")]
    pub max_download_concurrency: usize,

    /// 下载超时时间（秒）
    ///
    /// 单个文件下载的最大允许时间。
    /// 可以根据网络环境和文件大小调整。
    #[serde(default = "default_timeout")]
    pub download_timeout_secs: u64,

    /// 上传文件最大大小（字节）
    ///
    /// 值为 0 表示不限制。
    #[serde(default = "default_upload_max_file_size")]
    pub upload_max_file_size: u64,
}

fn default_url_prefix() -> String {
    "/files/".to_string()
}

fn default_max_concurrency() -> usize {
    10
}

fn default_timeout() -> u64 {
    300
}

fn default_upload_max_file_size() -> u64 {
    0
}

impl Default for FileRuntimeSettingData {
    fn default() -> Self {
        Self {
            local_public_url_prefix: default_url_prefix(),
            max_download_concurrency: default_max_concurrency(),
            download_timeout_secs: default_timeout(),
            upload_max_file_size: default_upload_max_file_size(),
        }
    }
}

impl SettingKey for FileRuntimeSettingData {
    fn key<'t>() -> &'t str {
        "file-runtime-config"
    }
}

impl SettingDecode for FileRuntimeSettingData {
    fn decode(data: &str) -> lsys_setting::dao::SettingResult<Self> {
        serde_json::from_str(data).map_err(lsys_setting::dao::SettingError::SerdeJson)
    }
}

impl SettingEncode for FileRuntimeSettingData {
    fn encode(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

// ==================== FileRuntimeSettingDao ====================

/// 文件运行时配置管理 DAO
///
/// 提供运行时配置的读取和更新功能。
/// 配置存储在数据库中，支持动态修改。
pub struct FileRuntimeSettingDao {
    setting: Arc<SingleSetting>,
    /// 任务超时时间（秒），用于验证 download_timeout_secs
    task_timeout: usize,
    /// 上传单分片大小上限（字节），用于验证 upload_max_file_size
    upload_chunk_max: u64,
}

impl FileRuntimeSettingDao {
    pub fn new(setting: Arc<SingleSetting>, task_timeout: usize, upload_chunk_max: u64) -> Self {
        Self {
            setting,
            task_timeout,
            upload_chunk_max,
        }
    }

    /// 获取运行时配置
    ///
    /// 如果数据库中不存在配置，返回默认值。
    pub async fn get_config(&self) -> FileResult<FileRuntimeSettingData> {
        match self.setting.load::<FileRuntimeSettingData>(None).await {
            Ok(setting_data) => {
                // SettingData implements Deref, so we can clone the inner data
                Ok((*setting_data).clone())
            }
            Err(_) => {
                // 如果配置不存在，返回默认值
                Ok(FileRuntimeSettingData::default())
            }
        }
    }

    /// 更新运行时配置
    ///
    /// # Arguments
    /// * `config` - 新的配置数据
    /// * `change_user_id` - 修改用户 ID
    /// * `env_data` - 请求环境信息
    ///
    /// # Returns
    /// * `Err` - 如果 download_timeout_secs >= task_timeout，或 upload_max_file_size 小于 upload_chunk_max
    pub async fn update_config(
        &self,
        config: &FileRuntimeSettingData,
        change_user_id: u64,
        env_data: Option<&lsys_core::utils::RequestEnv>,
    ) -> FileResult<()> {
        // 验证 download_timeout_secs 必须小于 task_timeout
        if config.download_timeout_secs >= self.task_timeout as u64 {
            return Err(FileError::System(lsys_core::fluent_message!(
                "file-download-timeout-invalid",
                {
                    "download_timeout": config.download_timeout_secs,
                    "task_timeout": self.task_timeout
                }
            )));
        }

        // 上传大小上限不得小于上传单分片上限，否则分片逻辑会乱套
        if config.upload_max_file_size > 0 && config.upload_max_file_size < self.upload_chunk_max {
            return Err(FileError::Param(lsys_core::fluent_message!(
                "file-upload-max-size-invalid",
                {
                    "upload_max_file_size": config.upload_max_file_size,
                    "upload_chunk_max": self.upload_chunk_max
                }
            )));
        }

        let param = SingleSettingData {
            name: "File Runtime Config",
            data: config,
        };

        self.setting
            .save::<FileRuntimeSettingData>(None, &param, change_user_id, None, env_data)
            .await?;
        Ok(())
    }

    /// 获取本地公开文件 URL 前缀
    ///
    /// 这是一个便捷方法，用于快速获取 URL 前缀。
    pub async fn get_local_public_url_prefix(&self) -> FileResult<String> {
        let config = self.get_config().await?;
        Ok(config.local_public_url_prefix)
    }

    /// 获取最大下载并发数
    pub async fn get_max_download_concurrency(&self) -> FileResult<usize> {
        let config = self.get_config().await?;
        Ok(config.max_download_concurrency)
    }

    /// 获取下载超时时间
    pub async fn get_download_timeout_secs(&self) -> FileResult<u64> {
        let config = self.get_config().await?;
        Ok(config.download_timeout_secs)
    }

    /// 获取上传文件最大大小（字节）
    ///
    /// 返回值为 0 表示不限制。
    pub async fn get_upload_max_file_size(&self) -> FileResult<u64> {
        let config = self.get_config().await?;
        Ok(config.upload_max_file_size)
    }
}
