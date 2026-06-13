use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::cache::LocalCache;
use lsys_core::db::{Insert, QueryBuilderExt, Update};
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::timeout_task::TimeOutTaskNotify;
use lsys_core::utils::{RequestEnv, now_time};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};
use tracing::warn;

use super::file_helpers::FileHelper;
use super::file_log::FileLogDao;
use super::file_op_context::FileOpContext;
use super::file_setting_oss::FileOssConfigDao;
use super::file_tag::FileTagDao;
use super::logger::*;
use super::*;
use crate::model::*;

/// 文件操作核心功能
///
/// 包含文件的创建、删除、路径获取、过期时间管理等核心操作
pub struct FileOps {
    pub(crate) helper: Arc<FileHelper>,
    pub(crate) oss_config: Arc<FileOssConfigDao>,
    pub(crate) logger: Arc<ChangeLoggerDao>,
    pub(crate) log_dao: FileLogDao,
    pub(crate) tag_dao: Arc<FileTagDao>,
    pub(crate) file_url_cache: Arc<LocalCache<u64, Option<String>>>,
    pub(crate) expiration_notify: Arc<TimeOutTaskNotify>,
}

impl FileOps {
    pub fn new(
        helper: Arc<FileHelper>,
        oss_config: Arc<FileOssConfigDao>,
        logger: Arc<ChangeLoggerDao>,
        tag_dao: Arc<FileTagDao>,
        file_url_cache: Arc<LocalCache<u64, Option<String>>>,
        expiration_notify: Arc<TimeOutTaskNotify>,
    ) -> Self {
        let log_dao = FileLogDao::new(helper.db.clone());
        Self {
            helper,
            oss_config,
            logger,
            log_dao,
            tag_dao,
            file_url_cache,
            expiration_notify,
        }
    }

    pub fn db(&self) -> &Pool<MySql> {
        &self.helper.db
    }

    pub fn helper(&self) -> &FileHelper {
        &self.helper
    }

    /// 创建 FileOpContext（已绑定 helper 和 oss_config）
    pub fn create_context<'a>(&'a self, file_ref: &'a FileRefModel) -> FileOpContext<'a> {
        FileOpContext::new(file_ref, &self.helper, &self.oss_config)
    }

    // ==================== 文件派生关系 ====================

    /// 创建文件派生关系记录 (lst_file_lineage)
    pub(crate) async fn add_lineage(
        &self,
        user_id: u64,
        app_id: u64,
        src_file_id: u64,
        dst_file_id: u64,
        rel_type: i8,
    ) -> FileResult<()> {
        let now = now_time()?;
        Insert::<sqlx::MySql, FileLineageModel>::new()
            .set(FileLineageModel::USER_ID, user_id)
            .set(FileLineageModel::APP_ID, app_id)
            .set(FileLineageModel::SRC_FILE_ID, src_file_id)
            .set(FileLineageModel::DST_FILE_ID, dst_file_id)
            .set(FileLineageModel::REL_TYPE, rel_type)
            .set(FileLineageModel::STATUS, FileLineageStatus::Normal as i8)
            .set(FileLineageModel::ADD_TIME, now)
            .execute(&self.helper.db)
            .await?;
        Ok(())
    }

    // ==================== 文件过期时间管理 ====================

    /// 更新文件过期时间
    ///
    /// # Arguments
    /// * `file_ref` - 文件用户引用模型
    /// * `expire_time` - 新的过期时间（Unix 时间戳），0 表示永不过期
    /// * `change_user_id` - 修改用户 ID
    /// * `env_data` - 请求环境信息
    ///
    /// # Returns
    /// * `Ok(u64)` - 受影响的行数
    /// * `Err(FileError)` - 更新失败
    pub async fn update_expire_time(
        &self,
        file_ref_user: &FileRefModel,
        expire_time: u64,
        change_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<u64> {
        // 检查文件是否已过期
        let current_time = now_time().unwrap_or(0);
        if file_ref_user.expire_time > 0 && file_ref_user.expire_time < current_time {
            return Err(FileError::Param(lsys_core::fluent_message!(
                "file-expired",
                {
                    "file_ref_id": file_ref_user.id,
                    "expire_time": file_ref_user.expire_time,
                    "current_time": current_time
                }
            )));
        }

        let rows = Update::<_, FileRefModel>::new()
            .set(FileRefModel::EXPIRE_TIME, expire_time)
            .execute(&self.helper.db, |qb| {
                qb.push_where()
                    .field_eq("id", file_ref_user.id)
                    .push_and()
                    .field_eq("status", FileUserStatus::Normal as i8);
            })
            .await?;

        if rows.rows_affected() > 0 {
            self.log_dao
                .add(
                    file_ref_user.id,
                    0,
                    change_user_id,
                    &format!("update_expire_time: expire_time={}", expire_time),
                    None,
                )
                .await;

            self.logger
                .add(
                    &LogFileExpireTimeUpdate {
                        file_ref_id: file_ref_user.id,
                        expire_time,
                        change_user_id,
                    },
                    Some(file_ref_user.id),
                    Some(change_user_id),
                    None,
                    env_data,
                )
                .await;

            // 主动通知过期后台任务尽快运行
            if expire_time > 0 {
                let ntime = now_time().unwrap_or_default();
                let timeout = expire_time.saturating_sub(ntime);
                if let Err(e) = self.expiration_notify.notify_timeout(timeout).await {
                    warn!(
                        "Failed to notify expiration task: {}",
                        e.to_fluent_message().default_format()
                    );
                }
            }
        }

        Ok(rows.rows_affected())
    }

    // ==================== 创建方法: 本地上传 ====================

    /// 通过已有 FILE 对象创建 file_ref 关联
    ///
    /// - `user_id`: 文件属于的用户ID,0=系统
    /// - `add_user_id`: 文件添加(上传)用户ID
    pub async fn create_file_ref(
        &self,
        file: &FileModel,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        file_name: &str,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<u64> {
        let now = now_time()?;
        let res = Insert::<_, FileRefModel>::new()
            .set(FileRefModel::USER_ID, user_id)
            .set(FileRefModel::ADD_USER_ID, add_user_id)
            .set(FileRefModel::APP_ID, app_id)
            .set(FileRefModel::FILE_ID, file.id)
            .set(FileRefModel::STATUS, FileUserStatus::Normal as i8)
            .set(FileRefModel::SOURCE_URL, "")
            .set(FileRefModel::SOURCE_MD5, "")
            .set(FileRefModel::FILE_NAME, file_name)
            .set(FileRefModel::ADD_TIME, now)
            .set(FileRefModel::DELETE_TIME, 0u64)
            .execute(&self.helper.db)
            .await?;

        self.file_url_cache.clear(&file.id).await;

        self.logger
            .add(
                &LogFileCreate {
                    action: "create_file_ref",
                    storage_type: &file.storage_type,
                    user_id,
                    file_id: file.id,
                    file_md5: &file.file_md5,
                },
                Some(file.id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok(res.last_insert_id())
    }

    // ==================== 操作方法: 获取本地文件路径 ====================
    /// 获取用户文件的本地存储路径
    ///
    /// 通过 `FileOpContext` 传入参数：
    /// - 必须: `file_ref`
    /// - 可选: `file`（未提供时自动查询）
    /// - 可选: `oss_provider`（OSS 文件需要同步到本地时使用）
    ///
    /// # 参数
    /// - `ctx`: 文件操作上下文
    /// - `oss_to_local_storage_type`: 当文件为 OSS 类型时，同步到本地使用的存储类型
    /// - `sync_oss_to_local_fn`: OSS 同步到本地的函数（避免循环依赖）
    pub async fn get_local_path<F, Fut>(
        &self,
        ctx: FileOpContext<'_>,
        oss_to_local_storage_type: &str,
        sync_oss_to_local_fn: F,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<Option<PathBuf>>
    where
        F: FnOnce(FileOpContext<'_>, &str, Option<&RequestEnv>) -> Fut,
        Fut: std::future::Future<Output = FileResult<(FileModel, FileRefModel)>>,
    {
        let file_ref = ctx.file_ref;
        let file = ctx.file().await?;
        if file.is_local() {
            let file_id = file.id;
            let local = self.helper.find_file_local_by_file_id(file_id).await?;
            if let Some(local_rec) = local {
                if local_rec.local_path.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(
                    self.helper
                        .get_full_local_path(&file.storage_type, &local_rec.local_path)
                        .await?,
                ));
            }
            Ok(None)
        } else {
            // 非local类型, 先同步到本地
            let sync_ctx = self.create_context(file_ref).with_file(file)?;
            let (local_file, _local_file_ref) =
                sync_oss_to_local_fn(sync_ctx, oss_to_local_storage_type, env_data).await?;
            let local = self
                .helper
                .find_file_local_by_file_id(local_file.id)
                .await?;
            if let Some(local_rec) = local {
                if local_rec.local_path.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(
                    self.helper
                        .get_full_local_path(&local_file.storage_type, &local_rec.local_path)
                        .await?,
                ));
            }
            Ok(None)
        }
    }

}
