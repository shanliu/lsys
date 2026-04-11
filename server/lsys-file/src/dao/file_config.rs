use lsys_core::app_core::AppCore;

/// 文件服务配置
#[derive(Debug, Clone)]
pub struct FileConfig {
    /// 公开文件存储基础路径, 默认 /tmp/public
    storage_base_path_public: String,
    /// 私有文件存储基础路径, 默认 /tmp/private
    storage_base_path_private: String,
    /// 加密文件存储基础路径, 默认 /tmp/crypto
    storage_base_path_crypto: String,
    /// 本地文件URL前缀, 如 http://127.0.0.1/file/ 或 /file/
    local_public_url_prefix: String,
    /// 清理开关(默认开启)
    pub cleanup_enabled: bool,
    /// 最小分片大小(字节), 默认 1MB
    pub min_chunk_size: u64,
    /// 最大下载并发数, 默认 10
    pub max_download_concurrency: usize,
    /// 下载超时时间(秒), 默认 60
    pub download_timeout_secs: u64,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            storage_base_path_public: "/tmp/public".to_string(),
            storage_base_path_private: "/tmp/private".to_string(),
            storage_base_path_crypto: "/tmp/crypto".to_string(),
            local_public_url_prefix: "/file/".to_string(),
            cleanup_enabled: true,
            min_chunk_size: 1024 * 1024, // 1MB
            max_download_concurrency: 10,
            download_timeout_secs: 60,
        }
    }
}

impl FileConfig {
    /// 从 AppCore 配置中创建 FileConfig
    /// 读取配置键（带 file_ 前缀）：
    /// - file_storage_base_path_public: 公开文件存储基础路径
    /// - file_storage_base_path_private: 私有文件存储基础路径
    /// - file_storage_base_path_crypto: 加密文件存储基础路径
    /// - file_local_url_prefix: 本地文件URL前缀
    /// - file_cleanup_enabled: 清理开关
    /// - file_min_chunk_size: 最小分片大小(字节)
    /// - file_max_download_concurrency: 最大下载并发数
    /// - file_download_timeout_secs: 下载超时时间(秒)
    pub fn from_config(app_core: &AppCore) -> Self {
        let config = lsys_core::config!(app_core.config);

        Self {
            storage_base_path_public: config
                .get_string("file_storage_base_path_public")
                .unwrap_or_else(|_| "/tmp/public".to_string()),
            storage_base_path_private: config
                .get_string("file_storage_base_path_private")
                .unwrap_or_else(|_| "/tmp/private".to_string()),
            storage_base_path_crypto: config
                .get_string("file_storage_base_path_crypto")
                .unwrap_or_else(|_| "/tmp/crypto".to_string()),
            local_public_url_prefix: config
                .get_string("file_local_url_prefix")
                .unwrap_or_else(|_| "/file/".to_string()),
            cleanup_enabled: config.get_bool("file_cleanup_enabled").unwrap_or(true),
            min_chunk_size: config
                .get_int("file_min_chunk_size")
                .map(|v| v as u64)
                .unwrap_or(1024 * 1024), // 默认 1MB
            max_download_concurrency: config
                .get_int("file_max_download_concurrency")
                .map(|v| v as usize)
                .unwrap_or(10),
            download_timeout_secs: config
                .get_int("file_download_timeout_secs")
                .map(|v| v as u64)
                .unwrap_or(60),
        }
    }

    /// 根据存储类型获取对应的基础路径
    ///
    /// # Arguments
    /// * `storage_type` - 存储类型，应为 STORAGE_TYPE_LOCAL_* 之一
    ///
    /// # Returns
    /// * `Some(&str)` - 对应的基础路径
    /// * `None` - 如果不是本地存储类型
    ////
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

    /// 获取本地文件URL前缀
    pub fn get_local_public_url_prefix(&self) -> &str {
        &self.local_public_url_prefix
    }
}
