// Web 文件管理模块

mod token_upload;

use std::sync::Arc;

use deadpool_redis::Pool as RedisPool;
use lsys_app::dao::AppNotifySender;
use lsys_core::app_core::AppCore;
use lsys_core::task_lifecycle::TaskNode;
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
        task_node: Arc<TaskNode>,
    ) -> WebResult<Self> {
        let _ = app_core;

        let download_node = task_node.child("web-file-download");
        let download_wait_node = task_node.child("web-file-download-wait");
        let progress_node = task_node.child("web-file-progress");
        let expiration_node = task_node.child("web-file-expiration");
        let unfinished_node = task_node.child("web-file-unfinished-timeout");

        let d_download = file_dao.clone();
        download_node.spawn(move |token| {
            async move {
                d_download.run_download_listener(token).await;
            }
        });

        let d_download_wait = file_dao.clone();
        download_wait_node.spawn(move |token| {
            async move {
                d_download_wait.run_download_wait_listener(token).await;
            }
        });

        let d_progress = file_dao.clone();
        progress_node.spawn(move |token| {
            async move {
                d_progress.run_progress_write_worker(token).await;
            }
        });

        let d_expiration = file_dao.clone();
        expiration_node.spawn(move |token| {
            async move {
                d_expiration.run_expiration_task(None, token).await;
            }
        });

        let d_unfinished = file_dao.clone();
        unfinished_node.spawn(move |token| {
            async move {
                d_unfinished.run_unfinished_timeout_task(None, token).await;
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
