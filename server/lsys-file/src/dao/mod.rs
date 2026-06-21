mod file_cache;
mod file_clear;
mod file_clear_timeout;
mod file_config;
mod file_data;
mod file_download;
mod file_download_dispatch;
mod file_expiration;
mod file_from;
mod file_helpers;
mod file_log;
mod file_op_context;
mod file_ops;
mod file_progress;
mod file_setting_oss;
mod file_setting_runtime;
mod file_tag;
mod logger;

pub use file_cache::*;
pub use file_clear_timeout::*;
pub use file_config::*;
pub use file_data::*;
pub use file_download::*;
pub use file_download_dispatch::*;
pub use file_expiration::*;
pub use file_from::file_from_local::*;
pub use file_from::file_from_upload::*;
pub use file_helpers::*;
pub use file_log::*;
pub use file_op_context::*;
pub use file_ops::*;
pub use file_progress::*;
pub use file_setting_oss::*;
pub use file_setting_runtime::*;
pub use file_tag::*;

// Re-export common types
pub use crate::common::*;

use lsys_core::app_core::AppCore;
use lsys_core::cache::LocalCache;
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::secret::SecretManager;
use lsys_core::timeout_task::TimeOutTask;
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};
use std::sync::Arc;

use crate::model::{FileLocalModel, FileModel, FileOssModel, FileRefModel};
use crate::oss::OssProviderRegistry;

/// 返回文件模块的操作日志类型列表
pub fn log_types() -> Vec<&'static str> {
    use logger::{
        LogFileCopy, LogFileCreate, LogFileDelete, LogFileExpireTimeUpdate, LogFileSync,
        LogFileUpload,
    };
    use lsys_logger::dao::ChangeLogData;
    vec![
        LogFileCreate::log_type(),
        LogFileUpload::log_type(),
        LogFileDelete::log_type(),
        LogFileCopy::log_type(),
        LogFileSync::log_type(),
        LogFileExpireTimeUpdate::log_type(),
    ]
}

/// 文件 DAO 主入口
pub struct FileDao {
    pub(crate) helper: Arc<FileHelper>,
    pub(crate) download_manager: Arc<FileDownloadDispatchManager>,
    pub(crate) oss_config: Arc<FileOssConfigDao>,
    pub(crate) runtime_setting: Arc<FileRuntimeSettingDao>,
    pub(crate) logger: Arc<ChangeLoggerDao>,
    pub(crate) log_dao: FileLogDao,
    pub(crate) data_dao: FileDataDao,
    pub(crate) tag_dao: Arc<FileTagDao>,
    pub(crate) file_url_cache: Arc<LocalCache<u64, Option<String>>>,
    pub(crate) file_ref_cache: Arc<LocalCache<u64, FileRefModel>>,
    pub(crate) file_model_cache: Arc<LocalCache<u64, FileModel>>,
    pub(crate) file_local_cache: Arc<LocalCache<u64, FileLocalModel>>,
    pub(crate) file_oss_cache: Arc<LocalCache<u64, FileOssModel>>,
    pub(crate) oss_config_cache:
        Arc<LocalCache<String, Option<lsys_setting::dao::SettingData<crate::dao::OssSettingData>>>>,
    pub(crate) file_key_encoder: Arc<FileKeyEncoder>,
    pub(crate) file_ops: Arc<FileOps>,
    pub(crate) app_core: Arc<AppCore>,
    pub(crate) redis: deadpool_redis::Pool,
    pub(crate) expiration_notify: Arc<lsys_core::timeout_task::TimeOutTaskNotify>,
}

impl FileDao {
    #[allow(clippy::too_many_arguments)]
    fn new(
        app_core: Arc<AppCore>,
        helper: Arc<FileHelper>,
        download_manager: Arc<FileDownloadDispatchManager>,
        oss_config: Arc<FileOssConfigDao>,
        runtime_setting: Arc<FileRuntimeSettingDao>,
        logger: Arc<ChangeLoggerDao>,
        file_url_cache: Arc<LocalCache<u64, Option<String>>>,
        file_ref_cache: Arc<LocalCache<u64, FileRefModel>>,
        file_model_cache: Arc<LocalCache<u64, FileModel>>,
        file_local_cache: Arc<LocalCache<u64, FileLocalModel>>,
        file_oss_cache: Arc<LocalCache<u64, FileOssModel>>,
        oss_config_cache: Arc<
            LocalCache<String, Option<lsys_setting::dao::SettingData<crate::dao::OssSettingData>>>,
        >,
        file_key_encoder: Arc<FileKeyEncoder>,
        redis: deadpool_redis::Pool,
        expiration_notify: Arc<lsys_core::timeout_task::TimeOutTaskNotify>,
    ) -> Self {
        let log_dao = FileLogDao::new(helper.db.clone());
        let data_dao = FileDataDao::new(
            helper.clone(),
            oss_config.clone(),
            runtime_setting.clone(),
            download_manager.clone(),
            file_key_encoder.clone(),
        );
        let tag_dao = Arc::new(FileTagDao::new(helper.db.clone()));
        let file_ops = Arc::new(FileOps::new(
            helper.clone(),
            oss_config.clone(),
            logger.clone(),
            tag_dao.clone(),
            file_url_cache.clone(),
            expiration_notify.clone(),
        ));
        Self {
            app_core,
            helper,
            download_manager,
            oss_config,
            runtime_setting,
            logger,
            log_dao,
            data_dao,
            tag_dao,
            file_url_cache,
            file_ref_cache,
            file_model_cache,
            file_local_cache,
            file_oss_cache,
            oss_config_cache,
            file_key_encoder,
            file_ops,
            redis,
            expiration_notify,
        }
    }

    /// 构建文件 DAO（使用 TaskDispatch 多节点下载）
    #[allow(clippy::too_many_arguments)]
    pub async fn build(
        db: Pool<MySql>,
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        secret_manager: Arc<SecretManager>,
        setting_dao: Arc<lsys_setting::dao::SettingDao>,
        registry: Arc<OssProviderRegistry>,
        logger: Arc<ChangeLoggerDao>,
        remote_notify: Arc<RemoteNotify>,
    ) -> Self {
        let config = FileConfig::from_config(&app_core);
        Self::build_with_config(
            db,
            app_core,
            redis,
            secret_manager,
            config,
            setting_dao,
            registry,
            logger,
            remote_notify,
        )
        .await
    }

    /// 构建文件 DAO（直接传入配置）
    #[allow(clippy::too_many_arguments)]
    pub async fn build_with_config(
        db: Pool<MySql>,
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        secret_manager: Arc<SecretManager>,
        config: FileConfig,
        setting_dao: Arc<lsys_setting::dao::SettingDao>,
        registry: Arc<OssProviderRegistry>,
        logger: Arc<ChangeLoggerDao>,
        remote_notify: Arc<RemoteNotify>,
    ) -> Self {
        // 确保所有存储目录在启动时已创建
        if let Err(e) = config.ensure_storage_dirs().await {
            tracing::error!("Failed to ensure storage directories: {}", e);
        }

        let runtime_setting = Arc::new(FileRuntimeSettingDao::new(
            setting_dao.single.clone(),
            config.download_config.task_timeout,
            config.upload_chunk_max,
        ));

        let helper = Arc::new(FileHelper::new(
            db.clone(),
            redis.clone(),
            app_core.clone(),
            config.clone(),
            runtime_setting.clone(),
            secret_manager.clone(),
        ));

        // 使用 TaskDispatch 多节点下载管理器
        let download = Arc::new(FileDownloadDispatchManager::new(
            redis.clone(),
            app_core.clone(),
            &config.download_config,
        ));

        // 创建 OSS 配置缓存（需要在 FileOssConfigDao 之前创建）
        let oss_config_cache = Arc::new(LocalCache::new(
            remote_notify.clone(),
            config.oss_config_cache,
        ));

        let oss_config = Arc::new(FileOssConfigDao::new(
            db.clone(),
            setting_dao.multiple.clone(),
            registry,
            oss_config_cache.clone(),
        ));

        // 创建文件 URL 缓存
        let file_url_cache = Arc::new(LocalCache::new(
            remote_notify.clone(),
            config.file_url_cache,
        ));

        // 创建 file_ref 缓存
        let file_ref_cache = Arc::new(LocalCache::new(
            remote_notify.clone(),
            config.file_ref_cache,
        ));

        // 创建 file_model 缓存
        let file_model_cache = Arc::new(LocalCache::new(
            remote_notify.clone(),
            config.file_model_cache,
        ));

        // 创建 file_local 缓存
        let file_local_cache = Arc::new(LocalCache::new(
            remote_notify.clone(),
            config.file_local_cache,
        ));

        // 创建 file_oss 缓存
        let file_oss_cache = Arc::new(LocalCache::new(
            remote_notify.clone(),
            config.file_oss_cache,
        ));

        // 创建混淆编码器（优先从 SecretManager 读取 file_key_salt，回退到 FileConfig）
        let file_key_salt = secret_manager
            .get("file_key_salt")
            .and_then(|b| std::str::from_utf8(b).ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| config.file_key_salt.clone());
        let file_key_encoder = Arc::new(FileKeyEncoder::new(
            &file_key_salt,
            config.file_key_min_length,
        ));

        // 创建文件过期任务通知器（在 FileDao 内部管理，与后台任务共享）
        let expiration_notify = Arc::new(lsys_core::timeout_task::TimeOutTaskNotify::new(
            redis.clone(),
            lsys_core::timeout_task::TimeOutTaskConfig::new(
                "file-expiration-task",
                config.expiration_task_timeout,
            ),
        ));

        Self::new(
            app_core,
            helper,
            download,
            oss_config,
            runtime_setting,
            logger,
            file_url_cache,
            file_ref_cache,
            file_model_cache,
            file_local_cache,
            file_oss_cache,
            oss_config_cache,
            file_key_encoder,
            redis,
            expiration_notify,
        )
    }

    pub fn helper(&self) -> &FileHelper {
        &self.helper
    }

    /// 获取文件操作核心功能
    pub fn file_ops(&self) -> &FileOps {
        &self.file_ops
    }

    /// 获取文件配置引用
    pub fn config(&self) -> &FileConfig {
        &self.helper.config
    }

    /// 获取文件日志 DAO
    pub fn log_dao(&self) -> &FileLogDao {
        &self.log_dao
    }

    /// 获取文件数据查询 DAO
    pub fn data_dao(&self) -> &FileDataDao {
        &self.data_dao
    }

    /// 获取文件标签 DAO
    pub fn tag_dao(&self) -> &FileTagDao {
        &self.tag_dao
    }

    /// 获取 OSS 配置管理 DAO
    pub fn oss_config(&self) -> &FileOssConfigDao {
        &self.oss_config
    }

    /// 获取文件下载进度跟踪器
    pub fn progress_tracker(&self) -> &FileProgressTracker {
        &self.helper.progress_tracker
    }

    /// 获取运行时配置管理 DAO
    pub fn runtime_setting(&self) -> &FileRuntimeSettingDao {
        &self.runtime_setting
    }

    /// 清空全部文件 URL 缓存（包含远程节点）
    pub async fn clear_all_file_url_cache(&self) {
        self.file_url_cache.clear_all().await;
    }

    /// 获取文件 key 编码器
    pub fn file_key_encoder(&self) -> &FileKeyEncoder {
        &self.file_key_encoder
    }

    /// 运行下载监听后台循环。
    /// 通常通过 `tokio::spawn` 调用。
    pub async fn run_download_listener(&self, cancel_token: tokio_util::sync::CancellationToken) {
        let acquisition = DownloadTaskAcquisition::new(self.helper.clone());
        let executor = Arc::new(DownloadTaskExecutorImpl::new(
            self.helper.clone(),
            self.download_manager.wait_notify.clone(),
        ));

        self.download_manager
            .task_dispatch
            .dispatch(self.app_core.clone(), &acquisition, executor, cancel_token)
            .await;
    }

    /// 运行下载等待通知监听
    /// 通常通过 `tokio::spawn` 调用。
    pub async fn run_download_wait_listener(&self, cancel_token: tokio_util::sync::CancellationToken) {
        self.download_manager.wait_notify.listen(cancel_token).await;
    }

    /// 运行进度写入后台循环（write_worker）。
    /// 通常通过 `tokio::spawn` 调用。
    pub async fn run_progress_write_worker(&self, cancel_token: tokio_util::sync::CancellationToken) {
        self.helper.progress_tracker.run_write_worker(cancel_token).await;
    }

    /// 运行 Unfinished 文件超时扫描任务监听。
    /// 定期将超过 `upload_unfinished_timeout` 仍处于 Unfinished 状态的文件标记为 Failed。
    /// 通常通过 `tokio::spawn` 调用。
    ///
    /// # Arguments
    /// * `channel_buffer` - 通道缓冲大小（可选）
    pub async fn run_unfinished_timeout_task(&self, channel_buffer: Option<usize>, cancel_token: tokio_util::sync::CancellationToken) {
        let notify = Arc::new(lsys_core::timeout_task::TimeOutTaskNotify::new(
            self.redis.clone(),
            lsys_core::timeout_task::TimeOutTaskConfig::new(
                "file-unfinished-timeout-task",
                self.helper.config.expiration_task_timeout,
            ),
        ));
        let task = Arc::new(FileUnfinishedTimeoutTask::new(
            self.helper.db.clone(),
            self.helper.config.upload_max_duration,
        ));

        TimeOutTask::<FileUnfinishedTimeoutTaskExecutor>::new(
            self.app_core.clone(),
            notify,
            task.clone(),
            task,
        )
        .listen(channel_buffer, cancel_token)
        .await;
    }

    /// 运行文件过期任务监听。
    /// 通常通过 `tokio::spawn` 调用。
    ///
    /// # Arguments
    /// * `channel_buffer` - 通道缓冲大小（可选）
    pub async fn run_expiration_task(&self, channel_buffer: Option<usize>, cancel_token: tokio_util::sync::CancellationToken) {
        let expiration_task = Arc::new(FileExpirationTask::new(
            self.helper.db.clone(),
            self.file_ops.clone(),
        ));

        TimeOutTask::<FileExpirationTask>::new(
            self.app_core.clone(),
            self.expiration_notify.clone(),
            expiration_task.clone(),
            expiration_task,
        )
        .listen(channel_buffer, cancel_token)
        .await;
    }
}
