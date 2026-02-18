use lsys_core::AppCore;

/// 文件服务配置
#[derive(Debug, Clone)]
pub struct FileConfig {
    /// 存储基础路径, 默认 /tmp
    pub storage_base_path: String,
    /// 本地文件URL前缀, 如 http://127.0.0.1/file/ 或 /file/
    pub local_file_url_prefix: String,
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
            storage_base_path: "/tmp".to_string(),
            local_file_url_prefix: "/file/".to_string(),
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
    /// - file_storage_base_path: 存储基础路径
    /// - file_local_url_prefix: 本地文件URL前缀
    /// - file_cleanup_enabled: 清理开关
    /// - file_min_chunk_size: 最小分片大小(字节)
    /// - file_max_download_concurrency: 最大下载并发数
    /// - file_download_timeout_secs: 下载超时时间(秒)
    pub fn from_config(app_core: &AppCore) -> Self {
        let config = lsys_core::config!(app_core.config);

        Self {
            storage_base_path: config
                .get_string("file_storage_base_path")
                .unwrap_or_else(|_| "/tmp".to_string()),
            local_file_url_prefix: config
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
}
