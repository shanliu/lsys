// Web 文件管理模块

mod token_upload;

use std::sync::Arc;

use deadpool_redis::Pool as RedisPool;
use lsys_app::dao::AppNotifySender;
use lsys_core::app_core::AppCore;
use lsys_file::dao::FileDao;
use lsys_file_manager::dao::UploadTokenManager;

use crate::dao::result::WebResult;

/// Web 文件管理
pub struct WebFile {
    pub file_dao: Arc<FileDao>,
    /// 上传令牌管理（service 与 rest 两类场景共用）
    pub upload_token: UploadTokenManager,
    /// 文件上传完成回调发送器（仅 rest 场景、且应用配置了回调地址时生效）
    file_notify_sender: Arc<AppNotifySender>,
}

impl WebFile {
    pub fn new(
        redis: RedisPool,
        app_core: Arc<AppCore>,
        file_dao: Arc<FileDao>,
        file_notify_sender: Arc<AppNotifySender>,
    ) -> WebResult<Self> {
        let _ = app_core;

        // 启动文件相关后台任务
        tokio::spawn({
            let d = file_dao.clone();
            async move {
                d.run_download_listener().await;
            }
        });

        tokio::spawn({
            let d = file_dao.clone();
            async move {
                d.run_download_wait_listener().await;
            }
        });

        tokio::spawn({
            let d = file_dao.clone();
            async move {
                d.run_progress_write_worker().await;
            }
        });

        tokio::spawn({
            let d = file_dao.clone();
            async move {
                d.run_expiration_task(None).await;
            }
        });

        tokio::spawn({
            let d = file_dao.clone();
            async move {
                d.run_unfinished_timeout_task(None).await;
            }
        });

        let upload_token = UploadTokenManager::new(redis);

        Ok(Self {
            file_dao,
            upload_token,
            file_notify_sender,
        })
    }
}
