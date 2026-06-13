use lsys_core::app_core::AppCore;
use lsys_core::dist_lock::{DistLock, DistLockConfig};
use lsys_core::secret::SecretManager;
use sqlx::{MySql, Pool};
use std::sync::Arc;

use super::file_config::FileConfig;
use super::file_progress::FileProgressTracker;
use super::file_setting_runtime::FileRuntimeSettingDao;

/// 辅助函数集合
pub struct FileHelper {
    pub(super) db: Pool<MySql>,
    pub(super) config: FileConfig,
    pub(super) runtime_setting: Arc<FileRuntimeSettingDao>,
    pub(super) sync_locker: DistLock,
    /// 密钥管理器，用于加密/解密文件
    pub(super) secret_manager: Arc<SecretManager>,
    /// 文件传输进度跟踪器（下载/上传循环调用 record_bytes，对外暴露 get_progress/subscribe_progress）
    pub progress_tracker: Arc<FileProgressTracker>,
}

impl FileHelper {
    pub fn new(
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        app_core: Arc<AppCore>,
        config: FileConfig,
        runtime_setting: Arc<FileRuntimeSettingDao>,
        secret_manager: Arc<SecretManager>,
    ) -> Self {
        let sync_locker = DistLock::new(Arc::new(
            DistLockConfig::builder(redis.clone())
                .key_prefix("file:sync:lock")
                .build(),
        ));
        let progress_tracker = Arc::new(FileProgressTracker::new(
            redis,
            app_core,
            config.download_config.task_timeout as u64,
            config.download_config.max_subscribe_conns.unwrap_or(100),
            config.download_config.progress_write_channel_cap.unwrap_or(512),
        ));
        Self {
            db,
            config,
            runtime_setting,
            sync_locker,
            secret_manager,
            progress_tracker,
        }
    }
}

mod chunk;
mod complete;
mod crypto;
mod file_key_encoder;
mod http;
mod ops;
mod query;

// Re-export types
pub use chunk::ChunkInfo;
pub use crypto::CRYPTO_SECRET_KEY_ID;
pub use file_key_encoder::FileKeyEncoder;
pub use http::UrlFileInfo;
