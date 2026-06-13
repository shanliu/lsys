use lsys_core::app_core::AppCore;
use lsys_core::cache::LocalCacheConfig;

/// 文件下载任务派发配置
#[derive(Debug, Clone)]
pub struct FileDownloadConfig {
    /// 并发任务数量（None 则使用 CPU 核心数）
    pub task_size: Option<usize>,
    /// 任务超时时间（秒）
    pub task_timeout: usize,
    /// 是否启用超时检测
    pub is_timeout_check: bool,
    /// 等待通知超时（秒）
    pub wait_timeout: u8,
    /// 进度订阅最大并发连接数（None 则默认 100）
    pub max_subscribe_conns: Option<usize>,
    /// 写入 Redis 的 channel 容量上限（None 则默认 512）
    pub progress_write_channel_cap: Option<usize>,
}

impl Default for FileDownloadConfig {
    fn default() -> Self {
        Self {
            task_size: None,
            task_timeout: 30 * 60, // 30 分钟
            is_timeout_check: true,
            wait_timeout: 30, // 30 秒
            max_subscribe_conns: None,
            progress_write_channel_cap: None,
        }
    }
}

/// 文件服务静态配置
///
/// 这些配置从配置文件读取，在部署时确定，通常不需要运行时修改。
/// 对于需要动态调整的配置（如 URL 前缀、并发数等），请使用 FileRuntimeSettingDao。
#[derive(Debug, Clone)]
pub struct FileConfig {
    /// 公开文件存储基础路径, 默认 /tmp/public
    storage_base_path_public: String,
    /// 私有文件存储基础路径, 默认 /tmp/private
    storage_base_path_private: String,
    /// 加密文件存储基础路径, 默认 /tmp/crypto
    storage_base_path_crypto: String,
    /// 清理开关(默认开启)
    pub cleanup_enabled: bool,
    /// 下载单分片大小上限(字节)：URL 下载按总大小平均拆分，每片不超过此值, 默认 5MB
    pub download_chunk_max: u64,
    /// 上传单分片大小上限(字节)：前端据此对文件分片, 默认 5MB
    ///
    /// 运行时 `max_upload_size` 更新时已被约束为不得小于此值
    pub upload_chunk_max: u64,
    /// OSS 同步锁超时时间（秒），即单次同步任务的最长允许执行时间，默认 30 分钟
    pub sync_lock_timeout: u64,
    /// 下载任务派发配置
    pub download_config: FileDownloadConfig,
    /// 文件过期任务最大连续执行时间（秒），默认 300 秒（5 分钟）
    pub expiration_task_timeout: usize,
    /// 上传最大允许时长（秒）：
    /// 1. `get_upload_handle` 会拒绝 add_time + 此值 < now() 的文件继续上传
    /// 2. 超时扫描任务用同一阈值将 Unfinished → Failed
    /// 
    /// 默认 6 小时，故意取长，兼容大文件/慢速网络
    pub upload_max_duration: u64,
    /// 文件 URL 缓存配置
    pub file_url_cache: LocalCacheConfig,
    /// 文件 Ref 缓存配置
    pub file_ref_cache: LocalCacheConfig,
    /// 文件 Model 缓存配置
    pub file_model_cache: LocalCacheConfig,
    /// 文件 Local 表缓存配置
    pub file_local_cache: LocalCacheConfig,
    /// 文件 OSS 表缓存配置
    pub file_oss_cache: LocalCacheConfig,
    /// OSS 配置缓存配置（按 config_key 缓存）
    pub oss_config_cache: LocalCacheConfig,
    /// 用于文件 ID 混淆的盐值
    pub file_key_salt: String,
    /// 混淆字符串的最小长度
    pub file_key_min_length: u8,
}

fn default_storage_path(sub: &str) -> String {
    std::env::temp_dir().join(sub).to_string_lossy().into_owned()
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            storage_base_path_public: default_storage_path("public"),
            storage_base_path_private: default_storage_path("private"),
            storage_base_path_crypto: default_storage_path("crypto"),
            cleanup_enabled: true,
            download_chunk_max: 5 * 1024 * 1024, // 5MB
            upload_chunk_max: 5 * 1024 * 1024,   // 5MB
            sync_lock_timeout: 30 * 60,      // 30 分钟
            download_config: FileDownloadConfig::default(),
            expiration_task_timeout: 300, // 5 分钟
            upload_max_duration: 6 * 60 * 60,      // 6 小时
            file_url_cache: LocalCacheConfig::new("file_url", Some(1000), Some(300)),
            file_ref_cache: LocalCacheConfig::new("file_ref", Some(2000), Some(60)),
            file_model_cache: LocalCacheConfig::new("file_model", Some(2000), Some(60)),
            file_local_cache: LocalCacheConfig::new("file_local", Some(2000), Some(60)),
            file_oss_cache: LocalCacheConfig::new("file_oss", Some(2000), Some(60)),
            oss_config_cache: LocalCacheConfig::new("oss_config", Some(100), Some(300)),
            file_key_salt: "lsys_file_default_salt".to_string(),
            file_key_min_length: 8,
        }
    }
}

impl FileConfig {
    /// 从 AppCore 配置中创建 FileConfig
    ///
    /// 读取配置键（带 file_ 前缀）：
    /// - file_storage_base_path: 存储基础路径，public/private/crypto 子目录自动派生（默认: 系统临时目录）
    /// - file_storage_base_path_public: 公开文件存储基础路径（覆盖 file_storage_base_path 派生值）
    /// - file_storage_base_path_private: 私有文件存储基础路径（覆盖 file_storage_base_path 派生值）
    /// - file_storage_base_path_crypto: 加密文件存储基础路径（覆盖 file_storage_base_path 派生值）
    /// - file_cleanup_enabled: 清理开关（默认: true）
    /// - file_download_chunk_max: 下载单分片大小上限(字节)（默认: 5242880 即 5MB）
    /// - file_upload_chunk_max: 上传单分片大小上限(字节)（默认: 5242880 即 5MB）
    /// - file_sync_lock_timeout: OSS 同步锁超时时间(秒)（默认: 1800）
    /// - file_upload_max_duration: 上传最大允许时长(秒)（默认: 21600 即 6 小时）
    /// - file_expiration_task_timeout: 文件过期任务最大连续执行时间(秒)（默认: 300）
    /// - file_download_task_size: 下载任务并发数
    /// - file_download_task_timeout: 下载任务超时时间(秒)（默认: 1800）
    /// - file_download_is_timeout_check: 是否启用超时检测（默认: true）
    /// - file_download_wait_timeout: 等待通知超时(秒)（默认: 30）
    /// - file_download_max_subscribe_conns: 进度订阅最大并发连接数（默认: 100）
    /// - file_key_salt: 文件 ID 混淆盐值（默认: "lsys_file_default_salt"）
    /// - file_key_min_length: 混淆字符串最小长度（默认: 8）
    ///
    /// 注意：运行时配置（URL 前缀、并发数、超时时间）已移至 FileRuntimeSettingDao
    pub fn from_config(app_core: &AppCore) -> Self {
        let config = lsys_core::config!(app_core.config);
        let default = Self::default();
        let default_download = FileDownloadConfig::default();
        let base: Option<std::path::PathBuf> = config
            .get_string("file_storage_base_path")
            .ok()
            .map(std::path::PathBuf::from);
        let sub_path = |sub: &str| -> String {
            base.as_ref()
                .map(|b| b.join(sub))
                .unwrap_or_else(|| std::env::temp_dir().join(sub))
                .to_string_lossy()
                .into_owned()
        };

        Self {
            storage_base_path_public: config
                .get_string("file_storage_base_path_public")
                .unwrap_or_else(|_| sub_path("public")),
            storage_base_path_private: config
                .get_string("file_storage_base_path_private")
                .unwrap_or_else(|_| sub_path("private")),
            storage_base_path_crypto: config
                .get_string("file_storage_base_path_crypto")
                .unwrap_or_else(|_| sub_path("crypto")),
            cleanup_enabled: config
                .get_bool("file_cleanup_enabled")
                .unwrap_or(default.cleanup_enabled),
            download_chunk_max: config
                .get_int("file_download_chunk_max")
                .map(|v| v as u64)
                .unwrap_or(default.download_chunk_max),
            upload_chunk_max: config
                .get_int("file_upload_chunk_max")
                .map(|v| v as u64)
                .unwrap_or(default.upload_chunk_max),
            sync_lock_timeout: config
                .get_int("file_sync_lock_timeout")
                .map(|v| v as u64)
                .unwrap_or(default.sync_lock_timeout),
            expiration_task_timeout: config
                .get_int("file_expiration_task_timeout")
                .map(|v| v as usize)
                .unwrap_or(default.expiration_task_timeout),
            upload_max_duration: config
                .get_int("file_upload_max_duration")
                .map(|v| v as u64)
                .unwrap_or(default.upload_max_duration),
            download_config: FileDownloadConfig {
                task_size: config
                    .get_int("file_download_task_size")
                    .ok()
                    .map(|v| v as usize),
                task_timeout: config
                    .get_int("file_download_task_timeout")
                    .map(|v| v as usize)
                    .unwrap_or(default_download.task_timeout),
                is_timeout_check: config
                    .get_bool("file_download_is_timeout_check")
                    .unwrap_or(default_download.is_timeout_check),
                wait_timeout: config
                    .get_int("file_download_wait_timeout")
                    .map(|v| v as u8)
                    .unwrap_or(default_download.wait_timeout),
                max_subscribe_conns: config
                    .get_int("file_download_max_subscribe_conns")
                    .ok()
                    .map(|v| v as usize),
                progress_write_channel_cap: config
                    .get_int("file_download_progress_write_channel_cap")
                    .ok()
                    .map(|v| v as usize),
            },
            file_url_cache: default.file_url_cache,
            file_ref_cache: default.file_ref_cache,
            file_model_cache: default.file_model_cache,
            file_local_cache: default.file_local_cache,
            file_oss_cache: default.file_oss_cache,
            oss_config_cache: default.oss_config_cache,
            file_key_salt: config
                .get_string("file_key_salt")
                .unwrap_or(default.file_key_salt),
            file_key_min_length: config
                .get_int("file_key_min_length")
                .map(|v| v as u8)
                .unwrap_or(default.file_key_min_length),
        }
    }

    /// 返回公开文件存储目录路径（字符串）
    pub fn public_dir(&self) -> &str {
        &self.storage_base_path_public
    }

    /// 根据存储类型获取对应的基础路径
    ///
    /// 此方法会：
    /// 1. 验证存储类型是否有效
    /// 2. 检查目录是否存在
    /// 3. 如果目录不存在，尝试创建
    ///
    /// # Arguments
    /// * `storage_type` - 存储类型
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - 对应的基础路径
    /// * `Err(std::io::Error)` - 未知的存储类型或目录不存在/无法创建
    pub async fn get_base_path(&self, storage_type: &str) -> std::io::Result<std::path::PathBuf> {
        use crate::model::FileModel;

        let path_str = match storage_type {
            FileModel::STORAGE_TYPE_LOCAL_PUBLIC => &self.storage_base_path_public,
            FileModel::STORAGE_TYPE_LOCAL_PRIVATE => &self.storage_base_path_private,
            FileModel::STORAGE_TYPE_LOCAL_CRYPTO => &self.storage_base_path_crypto,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unknown storage type: {}", storage_type),
                ));
            }
        };

        let path = std::path::PathBuf::from(path_str);

        // 检查目录是否存在，不存在则创建
        if !tokio::fs::try_exists(&path).await.map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to check storage directory '{}': {}",
                    path.display(),
                    e
                ),
            )
        })? {
            tokio::fs::create_dir_all(&path).await.map_err(|e| {
                std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to create storage directory '{}': {}",
                        path.display(),
                        e
                    ),
                )
            })?;
        }

        Ok(path)
    }

    /// 确保所有存储目录存在，不存在则创建
    ///
    /// 应在 FileDao 初始化时调用，保证 `public_dir()` 等同步访问器
    /// 返回的路径在文件系统上已存在（例如 actix_files 静态文件服务依赖此行为）。
    pub async fn ensure_storage_dirs(&self) -> std::io::Result<()> {
        use crate::model::FileModel;
        // 依次确保三种存储目录存在，复用 get_base_path 的创建逻辑
        self.get_base_path(FileModel::STORAGE_TYPE_LOCAL_PUBLIC).await?;
        self.get_base_path(FileModel::STORAGE_TYPE_LOCAL_PRIVATE).await?;
        self.get_base_path(FileModel::STORAGE_TYPE_LOCAL_CRYPTO).await?;
        Ok(())
    }
}
