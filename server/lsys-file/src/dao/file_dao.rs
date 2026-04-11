use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::cache::LocalCache;
use lsys_core::db::{Insert, QueryBuilderExt, TableMeta, Update};
use lsys_core::utils::{RequestEnv, now_time};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, QueryBuilder};
use tracing::warn;

use super::file_data::FileDataDao;
use super::file_download::FileDownloadManager;
use super::file_helpers::FileHelper;
use super::file_log::FileLogDao;
use super::file_tag::FileTagDao;
use super::logger::*;
use super::*;
use crate::model::*;

/// 文件 DAO 主入口
pub struct FileDao {
    pub(crate) helper: Arc<FileHelper>,
    pub(crate) download_manager: Arc<FileDownloadManager>,
    pub(crate) oss_config: Arc<FileOssConfigDao>,
    pub(crate) logger: Arc<ChangeLoggerDao>,
    pub(crate) log_dao: FileLogDao,
    pub(crate) data_dao: FileDataDao,
    pub(crate) tag_dao: FileTagDao,
    pub(crate) file_url_cache: Arc<LocalCache<u64, Option<String>>>,
}

impl FileDao {
    pub fn new(
        helper: Arc<FileHelper>,
        download_manager: Arc<FileDownloadManager>,
        oss_config: Arc<FileOssConfigDao>,
        logger: Arc<ChangeLoggerDao>,
        file_url_cache: Arc<LocalCache<u64, Option<String>>>,
    ) -> Self {
        let log_dao = FileLogDao::new(helper.db.clone());
        let data_dao = FileDataDao::new(helper.clone());
        let tag_dao = FileTagDao::new(helper.db.clone());
        Self {
            helper,
            download_manager,
            oss_config,
            logger,
            log_dao,
            data_dao,
            tag_dao,
            file_url_cache,
        }
    }

    pub fn helper(&self) -> &FileHelper {
        &self.helper
    }

    /// 创建 FileOpContext（已绑定 helper 和 oss_config）
    pub fn create_context<'a>(&'a self, file_user: &'a FileUserModel) -> FileOpContext<'a> {
        FileOpContext::new(file_user, &self.helper, &self.oss_config)
    }

    /// 获取文件配置引用
    pub fn config(&self) -> &super::file_config::FileConfig {
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

    /// 运行下载监听后台循环。
    /// 通常通过 `tokio::spawn` 调用。
    pub async fn run_download_listener(&self) {
        self.download_manager.listen().await;
    }

    // ==================== 创建方法 3: 本地上传 ====================

    /// 3.1 通过已有 FILE 对象创建 file_user 关联
    ///
    /// - `user_id`: 文件属于的用户ID,0=系统
    /// - `add_user_id`: 文件添加(上传)用户ID
    pub async fn create_file_user(
        &self,
        file: &FileModel,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<u64> {
        let now = now_time()?;
        let res = Insert::<_, FileUserModel>::new()
            .set(FileUserModel::USER_ID, user_id)
            .set(FileUserModel::ADD_USER_ID, add_user_id)
            .set(FileUserModel::APP_ID, app_id)
            .set(FileUserModel::FILE_ID, file.id)
            .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
            .set(FileUserModel::SOURCE_URL, "")
            .set(FileUserModel::SOURCE_MD5, "")
            .set(FileUserModel::ADD_TIME, now)
            .set(FileUserModel::DELETE_TIME, 0u64)
            .execute(&self.helper.db)
            .await?;

        self.logger
            .add(
                &LogFileCreate {
                    action: "create_file_user",
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

    // ==================== 操作方法 3: 转换为URL访问地址 ====================
    pub async fn get_file_url(&self, file: &FileModel) -> FileResult<Option<String>> {
        let urls = self.get_file_urls(std::slice::from_ref(file)).await?;
        Ok(urls.get(&file.id).cloned())
    }

    /// 批量获取文件 URL
    ///
    /// 传入多个 FileModel，一次性查询所有 local / oss 记录，返回 file_id -> url 的映射。
    /// 仅状态为 Normal 的文件才会返回 URL。
    pub async fn get_file_urls(
        &self,
        files: &[FileModel],
    ) -> FileResult<std::collections::HashMap<u64, String>> {
        use std::collections::HashMap;

        let mut result: HashMap<u64, String> = HashMap::new();
        if files.is_empty() {
            return Ok(result);
        }

        // 按存储类型分组，仅处理 Normal 状态
        let mut local_ids: Vec<u64> = Vec::new();
        let mut oss_ids: Vec<u64> = Vec::new();
        for f in files {
            if !FileStatus::Normal.eq(f.status) {
                continue;
            }
            if f.is_local() {
                local_ids.push(f.id);
            } else {
                oss_ids.push(f.id);
            }
        }

        // 批量查询 local 记录
        if !local_ids.is_empty() {
            let mut qb: QueryBuilder<MySql> =
                QueryBuilder::new(format!("SELECT * FROM {}", FileLocalModel::table_name(),));
            qb.push_where().field_in_copied("file_id", &local_ids);
            let locals: Vec<FileLocalModel> =
                qb.build_query_as().fetch_all(&self.helper.db).await?;
            let prefix = self.helper.config.get_local_public_url_prefix();
            for local_rec in &locals {
                if !local_rec.local_path.is_empty() {
                    result.insert(
                        local_rec.file_id,
                        format!("{}{}", prefix, local_rec.local_path),
                    );
                }
            }
        }

        // 批量查询 oss 记录
        if !oss_ids.is_empty() {
            let mut qb: QueryBuilder<MySql> =
                QueryBuilder::new(format!("SELECT * FROM {}", FileOssModel::table_name(),));
            qb.push_where().field_in_copied("file_id", &oss_ids);
            let osses: Vec<FileOssModel> = qb.build_query_as().fetch_all(&self.helper.db).await?;
            for oss_rec in &osses {
                if !oss_rec.object_url.is_empty() {
                    result.insert(oss_rec.file_id, oss_rec.object_url.clone());
                }
            }
        }

        Ok(result)
    }

    // ==================== 操作方法 4: 获取本地文件路径 ====================
    /// 获取用户文件的本地存储路径
    ///
    /// 通过 `FileOpContext` 传入参数：
    /// - 必须: `file_user`
    /// - 可选: `file`（未提供时自动查询）
    /// - 可选: `oss_provider`（OSS 文件需要同步到本地时使用）
    pub async fn get_local_path(
        &self,
        ctx: FileOpContext<'_>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<Option<PathBuf>> {
        let file_user = ctx.file_user;
        let file = ctx.file().await?;
        if file.is_local() {
            let file_id = file.id;
            let local = self.helper.find_file_local_by_file_id(file_id).await?;
            if let Some(local_rec) = local {
                if local_rec.local_path.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(self.helper.get_full_local_path(&file.storage_type, &local_rec.local_path).await?));
            }
            Ok(None)
        } else {
            // 非local类型, 先同步到本地
            let sync_ctx = self.create_context(file_user).with_file(file)?;
            let (local_file, _local_file_user) = self.sync_oss_to_local(sync_ctx, env_data).await?;
            let local = self
                .helper
                .find_file_local_by_file_id(local_file.id)
                .await?;
            if let Some(local_rec) = local {
                if local_rec.local_path.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(self.helper.get_full_local_path(&local_file.storage_type, &local_rec.local_path).await?));
            }
            Ok(None)
        }
    }

    // ==================== 操作方法 8: 删除文件 ====================
    /// 删除文件
    ///
    /// 通过 `FileOpContext` 传入参数：
    /// - 必须: `file_user`
    /// - 可选: `file`（未提供时不影响删除逻辑）
    /// - 可选: `oss_provider`（OSS 文件物理删除时使用）
    pub async fn delete_file(
        &self,
        ctx: FileOpContext<'_>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<()> {
        use tracing::info;

        let user_id = ctx.file_user.user_id;
        let app_id = ctx.file_user.app_id;
        let file_id = ctx.file_user.file_id;

        info!(
            "delete_file: starting, user_id={}, app_id={}, file_id={}",
            user_id, app_id, file_id
        );
        let now = now_time()?;

        // 查询该 file_id 的正常状态 file_user 数量
        let normal_count = sqlx::query_scalar::<_, i64>(&format!(
            "SELECT COUNT(*) FROM {} WHERE file_id=? AND status=?",
            FileUserModel::table_name(),
        ))
        .bind(file_id)
        .bind(FileUserStatus::Normal as i8)
        .fetch_one(&self.helper.db)
        .await?;

        info!(
            "delete_file: file_id={}, normal_count={}",
            file_id, normal_count
        );

        // 软删除 file_user
        let res = Update::<_, FileUserModel>::new()
            .set(FileUserModel::STATUS, FileUserStatus::Deleted as i8)
            .set(FileUserModel::DELETE_TIME, now)
            .execute(&self.helper.db, |qb| {
                qb.push_where()
                    .field_eq("id", ctx.file_user.id)
                    .push_and()
                    .field_eq("status", FileUserStatus::Normal as i8);
            })
            .await?;

        if res.rows_affected() == 0 {
            // 物理文件删除判断
            return Ok(());
        }

        self.log_dao
            .add(file_id, 0, user_id, "delete_file: file_user deleted", None)
            .await;

        // 同时删除该 file_user 关联的所有标签
        self.tag_dao
            .remove_all_tags(file_id, user_id, app_id, None)
            .await?;

        // 如果还有其他引用，不删除 file
        if normal_count > 1 {
            info!("delete_file: other refs exist, skipping file deletion");
        } else {
            // 软删除 file
            let file_res = Update::<_, FileModel>::new()
                .set(FileModel::STATUS, FileStatus::Deleted as i8)
                .set(FileModel::CHANGE_TIME, now)
                .execute(&self.helper.db, |qb| {
                    qb.push_where()
                        .field_eq("id", file_id)
                        .push_and()
                        .field_eq("status", FileStatus::Normal as i8);
                })
                .await?;

            if file_res.rows_affected() > 0 {
                self.log_dao
                    .add(file_id, 0, user_id, "delete_file: file deleted", None)
                    .await;
                
                // 清理关联关系：将所有指向该文件的关联字段重置为 0
                if let Some(file) = ctx.file().await.ok() {
                    if file.is_local() {
                        // 获取该文件的 local 记录
                        if self.helper.find_file_local_by_file_id(file_id).await.ok().flatten().is_some() {
                            // 根据文件类型，清理其他文件指向它的关联
                            let update_field = match file.storage_type.as_str() {
                                FileModel::STORAGE_TYPE_LOCAL_PUBLIC => Some(FileLocalModel::PUBLIC_FILE_ID),
                                FileModel::STORAGE_TYPE_LOCAL_PRIVATE => Some(FileLocalModel::PRIVATE_FILE_ID),
                                FileModel::STORAGE_TYPE_LOCAL_CRYPTO => Some(FileLocalModel::CRYPTO_FILE_ID),
                                _ => None,
                            };

                            if let Some(field) = update_field {
                                // 清理所有指向该文件的关联
                                let _ = sqlx::query(&format!(
                                    "UPDATE {} SET {} = 0 WHERE {} = ?",
                                    FileLocalModel::table_name(),
                                    field,
                                    field
                                ))
                                .bind(file_id)
                                .execute(&self.helper.db)
                                .await;
                            }

                            // 清理该文件自己的关联字段
                            let _ = sqlx::query(&format!(
                                "UPDATE {} SET public_file_id = 0, private_file_id = 0, crypto_file_id = 0 WHERE file_id = ?",
                                FileLocalModel::table_name()
                            ))
                            .bind(file_id)
                            .execute(&self.helper.db)
                            .await;
                        }
                    } else {
                        // OSS 文件：清理 local_file_id
                        let _ = sqlx::query(&format!(
                            "UPDATE {} SET local_file_id = 0 WHERE file_id = ?",
                            FileOssModel::table_name()
                        ))
                        .bind(file_id)
                        .execute(&self.helper.db)
                        .await;

                        // 同时清理所有指向该 OSS 文件的 local 文件的 local_file_id
                        let _ = sqlx::query(&format!(
                            "UPDATE {} SET local_file_id = 0 WHERE local_file_id = ?",
                            FileOssModel::table_name()
                        ))
                        .bind(file_id)
                        .execute(&self.helper.db)
                        .await;
                    }
                }
                
                // 清除缓存
                self.file_url_cache.clear(&file_id).await;
                
                // 物理文件删除判断
                self.try_cleanup_physical_file(
                    file_id,
                    ctx.file().await.ok(),
                    ctx.oss_provider().await.ok(),
                )
                .await;
            }
        }

        self.logger
            .add(
                &LogFileDelete { user_id, file_id },
                Some(file_id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }

    /// 尝试清理物理文件
    async fn try_cleanup_physical_file(
        &self,
        file_id: u64,
        file_opt: Option<&FileModel>,
        oss_provider: Option<&dyn OssProvider>,
    ) {
        use tracing::info;

        info!("try_cleanup_physical_file: checking file_id={}", file_id);

        // 优先使用调用方传入的 file，否则从 DB 查询
        let owned_file;
        let file = if let Some(f) = file_opt {
            f
        } else {
            match self.helper.find_file_by_id(file_id).await {
                Ok(Some(f)) => {
                    owned_file = f;
                    &owned_file
                }
                Ok(None) => {
                    warn!("try_cleanup_physical_file: file_id={} not found", file_id);
                    return;
                }
                Err(e) => {
                    warn!(
                        "try_cleanup_physical_file: query file_id={} error: {}",
                        file_id, e
                    );
                    return;
                }
            }
        };

        if file.is_local() {
            let local = match self.helper.find_file_local_by_file_id(file_id).await {
                Ok(Some(l)) => l,
                Ok(None) => {
                    warn!(
                        "try_cleanup_physical_file: file_local not found for file_id={}",
                        file_id
                    );
                    return;
                }
                Err(e) => {
                    warn!("try_cleanup_physical_file: query file_local error: {}", e);
                    return;
                }
            };

            if local.local_path.is_empty() {
                info!(
                    "try_cleanup_physical_file: local_path is empty for file_id={}",
                    file_id
                );
                return;
            }

            // 检查是否有其他 local_path 引用
            let local_path_refs = sqlx::query_scalar::<_, i64>(&format!(
                "SELECT COUNT(*) FROM {} fl INNER JOIN {} f ON fl.file_id=f.id \
                     WHERE fl.local_path=? AND f.status=? AND f.id!=?",
                FileLocalModel::table_name(),
                FileModel::table_name(),
            ))
            .bind(&local.local_path)
            .bind(FileStatus::Normal as i8)
            .bind(file_id)
            .fetch_one(&self.helper.db)
            .await
            .unwrap_or(0);

            // 检查是否有相同 file_md5 的其他文件引用（排除拷贝文件，拷贝文件拥有独立的物理文件）
            let md5_refs = if !file.file_md5.is_empty() {
                sqlx::query_scalar::<_, i64>(
                    &format!(
                        "SELECT COUNT(*) FROM {} WHERE file_md5=? AND storage_type=? AND status=? AND id!=? AND copy_file_id=0",
                        FileModel::table_name(),
                    )
                )
                .bind(&file.file_md5)
                .bind(&file.storage_type)
                .bind(FileStatus::Normal as i8)
                .bind(file_id)
                .fetch_one(&self.helper.db)
                .await
                .unwrap_or(0)
            } else {
                0
            };

            info!(
                "try_cleanup_physical_file: file_id={}, local_path_refs={}, md5_refs={}",
                file_id, local_path_refs, md5_refs
            );

            if local_path_refs > 0 || md5_refs > 0 {
                self.log_dao
                    .add(
                        file_id,
                        0,
                        0,
                        &format!(
                            "delete: skip physical delete, refs exist (local_path={}, md5={})",
                            local_path_refs, md5_refs
                        ),
                        None,
                    )
                    .await;
                return;
            }

            // 删除物理文件
            let full = self.helper.get_full_local_path(&file.storage_type, &local.local_path).await
                .unwrap_or_else(|_| PathBuf::from(&local.local_path));
            info!("try_cleanup_physical_file: deleting file {:?}", full);

            if let Err(e) = tokio::fs::remove_file(&full).await {
                warn!("delete physical file failed: {}", e);
                self.log_dao
                    .add(
                        file_id,
                        0,
                        0,
                        &format!("delete: physical delete failed: {}", e),
                        None,
                    )
                    .await;
            } else {
                info!("try_cleanup_physical_file: successfully deleted {:?}", full);
                self.log_dao
                    .add(file_id, 0, 0, "delete: physical file deleted", None)
                    .await;
            }
        } else {
            // OSS 文件删除
            info!(
                "try_cleanup_physical_file: processing OSS file for file_id={}",
                file_id
            );

            if let Some(provider) = oss_provider {
                if let Ok(Some(oss)) = self.helper.find_file_oss_by_file_id(file_id).await {
                    // 检查是否有相同 file_md5 的其他 OSS 文件引用
                    let md5_refs = if !file.file_md5.is_empty() {
                        sqlx::query_scalar::<_, i64>(
                            &format!(
                                "SELECT COUNT(*) FROM {} WHERE file_md5=? AND storage_type=? AND status=? AND id!=?",
                                FileModel::table_name(),
                            )
                        )
                        .bind(&file.file_md5)
                        .bind(&file.storage_type)
                        .bind(FileStatus::Normal as i8)
                        .bind(file_id)
                        .fetch_one(&self.helper.db)
                        .await
                        .unwrap_or(0)
                    } else {
                        0
                    };

                    if md5_refs > 0 {
                        info!(
                            "try_cleanup_physical_file: skip OSS delete, md5_refs={} for file_id={}",
                            md5_refs, file_id
                        );
                        self.log_dao
                            .add(
                                file_id,
                                0,
                                0,
                                &format!(
                                    "delete: skip OSS delete, md5 refs exist (count={})",
                                    md5_refs
                                ),
                                None,
                            )
                            .await;
                        return;
                    }

                    if let Err(e) = provider.delete_object(&oss).await {
                        warn!("delete OSS object failed: {}", e);
                        self.log_dao
                            .add(
                                file_id,
                                0,
                                0,
                                &format!("delete: OSS delete failed: {}", e),
                                None,
                            )
                            .await;
                    } else {
                        info!(
                            "try_cleanup_physical_file: OSS object deleted for file_id={}",
                            file_id
                        );
                        self.log_dao
                            .add(file_id, 0, 0, "delete: OSS object deleted", None)
                            .await;
                    }
                }
            } else {
                info!(
                    "try_cleanup_physical_file: no OSS provider for file_id={}",
                    file_id
                );
            }
        }
    }
}
