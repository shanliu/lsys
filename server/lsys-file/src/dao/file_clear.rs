//! 文件删除与物理清理
//!
//! 将删除流程拆分为以下私有方法，主入口 [`FileOps::delete_file`] 只做流程编排：
//!
//! | 方法                        | 职责                                                      |
//! |-----------------------------|-----------------------------------------------------------|
//! | `delete_file`               | 公开入口：事务内软删 file_ref，按剩余引用数决定后续动作   |
//! | `delete_file_model`         | 软删 FileModel，触发所有权转移检查或物理清理              |
//! | `try_transfer_ownership`    | 本地文件所有权转移事务（`local_path_owner_id` 重分配）    |
//! | `try_cleanup_physical_file` | 按文件类型分发到本地或 OSS 物理清理                       |
//! | `cleanup_local_file`        | 删除本地磁盘文件                                          |
//! | `cleanup_oss_file`          | 删除 OSS 对象                                             |
//!
//! ## 并发安全
//!
//! 旧代码在事务外先查 `normal_count`，后删 file_ref，存在 TOCTOU 窗口：
//! 另一线程可在 count 与 delete 之间插入新的 file_ref，导致删除方误以为
//! 自己是"最后一个引用"而错误清理了 FileModel。
//!
//! 新实现改为：**在同一事务内** 软删 file_ref，然后立即查剩余引用数
//! (`remaining_count`)，以事务提交后的 `remaining_count == 0` 作为
//! 是否清理 FileModel 的唯一判据，彻底消除该竞争窗口。

use std::path::PathBuf;

use lsys_core::db::{QueryBuilderExt, TableMeta, Update};
use lsys_core::fluent_message;
use lsys_core::utils::{RequestEnv, now_time};
use tracing::{info, warn};

use super::file_op_context::FileOpContext;
use super::file_ops::FileOps;
use super::logger::*;
use super::*;
use crate::model::*;

impl FileOps {
    // =========================================================================
    // 公开入口
    // =========================================================================

    /// 删除文件用户引用，并在最后一个引用被删除时清理底层 FileModel 及物理文件。
    ///
    /// # 并发安全
    ///
    /// soft-delete file_ref 与 count 剩余引用均在同一数据库事务内完成，
    /// 以避免"并发新增 file_ref"与"删除最后一个引用"之间的竞争条件。
    pub async fn delete_file(
        &self,
        ctx: FileOpContext<'_>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<()> {
        let file_ref_id = ctx.file_ref.id;
        let user_id = ctx.file_ref.user_id;
        let app_id = ctx.file_ref.app_id;
        let file_id = ctx.file_ref.file_id;

        info!(
            "delete_file: user_id={}, app_id={}, file_id={}, file_ref_id={}",
            user_id, app_id, file_id, file_ref_id
        );

        let now = now_time()?;

        // ── Step 1: 事务内软删 file_ref + 查剩余引用数 ───────────────────────
        //
        // 关键设计：在同一事务中完成"删除"和"计数"，保证两者基于相同的数据视图。
        // 若计数在删除之前执行（旧方案），并发新增可在两者之间插入新引用，
        // 使删除方错误地认为自己是最后一个引用，进而触发不必要的 FileModel 清理。
        let (was_deleted, remaining_count) = self
            .delete_ref_and_count(file_ref_id, file_id, now)
            .await?;

        if !was_deleted {
            // file_ref 已不在 Normal 状态，说明并发删除已处理，静默返回
            info!(
                "delete_file: file_ref_id={} not found or already deleted, skip",
                file_ref_id
            );
            return Ok(());
        }

        info!(
            "delete_file: file_ref_id={} deleted, remaining_refs={}",
            file_ref_id, remaining_count
        );

        self.log_dao
            .add(file_id, 0, user_id, "delete_file: file_ref deleted", None)
            .await;

        // ── Step 2: 删除该 file_ref 关联的所有标签 ───────────────────────────
        self.tag_dao
            .remove_all_tags(file_id, user_id, app_id, None)
            .await?;

        // ── Step 2b: 软删该用户/应用下与此文件相关的所有派生关系记录（双向）──────
        // 当用户删除自己的 file_ref 时，该用户视角下：
        //   - 从此文件派生出去的记录（src=file_id）已失效
        //   - 此文件本身来自某个源头的记录（dst=file_id）也已失效
        // 注意：Unfinished/Failed 状态的文件不会产生 lineage 记录，此处只影响 Normal 文件。
        for (col, label) in &[("src_file_id", "src"), ("dst_file_id", "dst")] {
            let sql = format!(
                "UPDATE {} SET status=? WHERE {}=? AND user_id=? AND app_id=? AND status=?",
                FileLineageModel::table_name(),
                col
            );
            if let Err(e) = sqlx::query(&sql)
                .bind(FileLineageStatus::Deleted as i8)
                .bind(file_id)
                .bind(user_id)
                .bind(app_id)
                .bind(FileLineageStatus::Normal as i8)
                .execute(&self.helper.db)
                .await
            {
                warn!(
                    "delete_file: soft-delete {} lineage failed file_id={} user_id={}: {}",
                    label, file_id, user_id, e
                );
            }
        }

        // ── Step 3: 若已无正常引用，触发 FileModel 及物理文件清理 ─────────────
        if remaining_count > 0 {
            info!(
                "delete_file: {} ref(s) remain for file_id={}, skip FileModel cleanup",
                remaining_count, file_id
            );
        } else {
            // 从 ctx 获取 oss_provider（借用 ctx，不消费它）
            let oss_provider = ctx.oss_provider().await.ok();
            self.delete_file_model(file_id, now, user_id, oss_provider, env_data)
                .await?;
        }

        // ── Step 4: 写操作日志 ────────────────────────────────────────────────
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

    // =========================================================================
    // 私有：事务内软删 file_ref 并返回剩余引用数
    // =========================================================================

    /// 在单个数据库事务中：
    /// 1. 将指定 `file_ref_id` 的状态更新为 `Deleted`。
    /// 2. 统计 `file_id` 下剩余的 `Normal` 状态引用数。
    ///
    /// 返回 `(was_deleted, remaining_count)`：
    /// - `was_deleted`      : 本次操作实际执行了软删（false = 已被删或不存在）
    /// - `remaining_count`  : 软删后的剩余正常引用数
    async fn delete_ref_and_count(
        &self,
        file_ref_id: u64,
        file_id: u64,
        now: u64,
    ) -> FileResult<(bool, i64)> {
        let mut tx = self.helper.db.begin().await?;

        let res = Update::<_, FileRefModel>::new()
            .set(FileRefModel::STATUS, FileUserStatus::Deleted as i8)
            .set(FileRefModel::DELETE_TIME, now)
            .execute(&mut *tx, |qb| {
                qb.push_where()
                    .field_eq("id", file_ref_id)
                    .push_and()
                    .field_eq("status", FileUserStatus::Normal as i8);
            })
            .await?;

        if res.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok((false, 0));
        }

        // 在同一事务的一致性视图下统计剩余引用
        let remaining: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM {} WHERE file_id=? AND status=?",
            FileRefModel::table_name(),
        ))
        .bind(file_id)
        .bind(FileUserStatus::Normal as i8)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok((true, remaining))
    }

    // =========================================================================
    // 私有：软删 FileModel 并触发物理清理
    // =========================================================================

    /// 软删 FileModel，并决策是否清理物理文件。
    ///
    /// # 并发安全
    ///
    /// 整个"验证无引用 → 所有权转移/软删 FileModel"在同一个数据库事务中完成。
    /// 事务内通过 `SELECT ... FOR UPDATE` 对 `file_ref` 行加锁，
    /// 阻止并发 `INSERT` 在验证与删除之间插入新引用，防止新引用指向已删 FileModel。
    async fn delete_file_model(
        &self,
        file_id: u64,
        now: u64,
        user_id: u64,
        oss_provider: Option<&dyn OssProvider>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<()> {
        // ── 开启事务，锁住 file_ref 行防止并发新增引用 ─────────────────────
        let mut tx = self.helper.db.begin().await?;

        // FOR UPDATE 对 file_ref 中 file_id=? 的行加行锁，
        // 同时在 InnoDB 的 RR 隔离级别下会对索引间隙加间隙锁，
        // 阻止其他事务在相同 file_id 下 INSERT 新的 file_ref 行。
        let remaining: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM {} WHERE file_id=? AND status=? FOR UPDATE",
            FileRefModel::table_name(),
        ))
        .bind(file_id)
        .bind(FileUserStatus::Normal as i8)
        .fetch_one(&mut *tx)
        .await?;

        if remaining > 0 {
            // 并发新增了引用，放弃 FileModel 清理
            tx.rollback().await.ok();
            info!(
                "delete_file_model: {} ref(s) re-appeared for file_id={} (concurrent insert), abort cleanup",
                remaining, file_id
            );
            return Ok(());
        }

        // 加载 FileModel（事务内读取保证一致性）
        let file = match self.helper.find_file_by_id(file_id).await? {
            Some(f) => f,
            None => {
                tx.rollback().await.ok();
                warn!(
                    "delete_file_model: file_id={} not found, skip cleanup",
                    file_id
                );
                return Ok(());
            }
        };

        // 状态判断：
        // - Deleted    → 并发已处理，静默跳过
        // - Normal     → 正常删除流程（含所有权转移检查）
        // - Failed     → 超时任务已标记，跳过所有权转移，直接软删 + 物理清理
        // - Unfinished → 正在上传中，拒绝操作
        let is_failed = FileStatus::Failed.eq(file.status);
        if FileStatus::Deleted.eq(file.status) {
            tx.rollback().await.ok();
            info!(
                "delete_file_model: file_id={} already Deleted, skip",
                file_id
            );
            return Ok(());
        } else if FileStatus::Unfinished.eq(file.status) {
            tx.rollback().await.ok();
            return Err(FileError::Param(fluent_message!(
                "file-unfinished",
                "file is still uploading, please wait for it to complete or timeout"
            )));
        } else if !FileStatus::Normal.eq(file.status) && !is_failed {
            tx.rollback().await.ok();
            warn!(
                "delete_file_model: file_id={} unknown status={}, skip",
                file_id, file.status
            );
            return Ok(());
        }

        // 本地文件所有权转移检查（Failed 状态跳过）
        if !is_failed && file.local_path_owner_id == 0 && file.is_local() {
            let transferred = self
                .try_transfer_ownership_in_tx(&mut tx, file_id, now, user_id, env_data)
                .await?;

            if transferred {
                tx.commit().await?;

                // file_id 已被软删，清理所有方向的 lineage（不限用户/应用）
                for (col, label) in &[("src_file_id", "src"), ("dst_file_id", "dst")] {
                    let sql = format!(
                        "UPDATE {} SET status=? WHERE {}=? AND status=?",
                        FileLineageModel::table_name(),
                        col
                    );
                    if let Err(e) = sqlx::query(&sql)
                        .bind(FileLineageStatus::Deleted as i8)
                        .bind(file_id)
                        .bind(FileLineageStatus::Normal as i8)
                        .execute(&self.helper.db)
                        .await
                    {
                        warn!(
                            "delete_file_model: soft-delete {} lineage (transfer) failed file_id={}: {}",
                            label, file_id, e
                        );
                    }
                }

                self.log_dao
                    .add(
                        file_id,
                        0,
                        user_id,
                        "delete_file: ownership transferred",
                        None,
                    )
                    .await;
                self.file_url_cache.clear(&file_id).await;

                // 物理文件由新 owner 接管，跳过物理删除
                info!(
                    "delete_file_model: ownership transferred for file_id={}, skip physical cleanup",
                    file_id
                );
                return Ok(());
            }
        }

        // ── 软删 FileModel（事务内） ────────────────────────────────────────
        // Normal 和 Failed 均可软删：匹配当前实际状态而非硬编码 Normal
        let res = Update::<_, FileModel>::new()
            .set(FileModel::STATUS, FileStatus::Deleted as i8)
            .set(FileModel::CHANGE_TIME, now)
            .execute(&mut *tx, |qb| {
                qb.push_where()
                    .field_eq("id", file_id)
                    .push_and()
                    .field_eq("status", file.status); // 匹配当前实际状态
            })
            .await?;

        if res.rows_affected() == 0 {
            tx.rollback().await.ok();
            info!(
                "delete_file_model: file_id={} already deleted by concurrent op, skip cleanup",
                file_id
            );
            return Ok(());
        }

        tx.commit().await?;

        // FileModel 已彻底软删，src/dst 两个方向的全部 lineage 记录均失效
        for (col, label) in &[("src_file_id", "src"), ("dst_file_id", "dst")] {
            let sql = format!(
                "UPDATE {} SET status=? WHERE {}=? AND status=?",
                FileLineageModel::table_name(),
                col
            );
            if let Err(e) = sqlx::query(&sql)
                .bind(FileLineageStatus::Deleted as i8)
                .bind(file_id)
                .bind(FileLineageStatus::Normal as i8)
                .execute(&self.helper.db)
                .await
            {
                warn!(
                    "delete_file_model: soft-delete {} lineage failed file_id={}: {}",
                    label, file_id, e
                );
            }
        }

        self.log_dao
            .add(file_id, 0, user_id, "delete_file: file deleted", None)
            .await;

        self.file_url_cache.clear(&file_id).await;

        // ── 物理文件清理 ───────────────────────────────────────────
        // Failed: 上传未完成，尝试清理分片残留文件 + 部分 local_path
        // Normal: 常规路径（崇重删除 / OSS 删除）
        if is_failed {
            self.cleanup_failed_file(file_id, &file).await;
        } else {
            self.try_cleanup_physical_file(file_id, Some(&file), oss_provider)
                .await;
        }

        Ok(())
    }

    // =========================================================================
    // 私有：本地文件所有权转移（事务内）
    // =========================================================================

    /// 在给定事务内尝试将 `local_path_owner_id == 0` 的物理文件所有权
    /// 从 `file_id` 转移给其他记录。
    ///
    /// 调用方负责管理事务的提交/回滚。
    ///
    /// 返回值：
    /// - `Ok(true)`  : 转移成功，物理文件由新 owner 接管
    /// - `Ok(false)` : 无依赖记录，调用方继续执行软删 + 物理删除
    async fn try_transfer_ownership_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
        file_id: u64,
        now: u64,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<bool> {
        // 查找 local_path_owner_id 指向 file_id 的最小 id（候选新 owner）
        let new_owner_id: Option<u64> = sqlx::query_scalar(&format!(
            "SELECT MIN(id) FROM {} WHERE local_path_owner_id=? AND status!=?",
            FileModel::table_name(),
        ))
        .bind(file_id)
        .bind(FileStatus::Deleted as i8)
        .fetch_optional(&mut **tx)
        .await?;

        let Some(new_owner_id) = new_owner_id else {
            return Ok(false); // 无依赖记录，无需转移
        };

        info!(
            "try_transfer_ownership: file_id={} → new_owner_id={}",
            file_id, new_owner_id
        );

        // 1. 软删旧 owner
        Update::<_, FileModel>::new()
            .set(FileModel::STATUS, FileStatus::Deleted as i8)
            .set(FileModel::CHANGE_TIME, now)
            .execute(&mut **tx, |qb| {
                qb.push_where()
                    .field_eq("id", file_id)
                    .push_and()
                    .field_eq("status", FileStatus::Normal as i8);
            })
            .await?;

        // 2. 新 owner 成为物理路径所有者（清除其 local_path_owner_id）
        Update::<_, FileModel>::new()
            .set(FileModel::LOCAL_PATH_OWNER_ID, 0u64)
            .execute(&mut **tx, |qb| {
                qb.push_where().field_eq("id", new_owner_id);
            })
            .await?;

        // 3. 将其余依赖记录重定向到新 owner（保持链深度 ≤ 1）
        sqlx::query(&format!(
            "UPDATE {} SET local_path_owner_id=? \
             WHERE local_path_owner_id=? AND id!=? AND status!=?",
            FileModel::table_name(),
        ))
        .bind(new_owner_id)
        .bind(file_id)
        .bind(new_owner_id)
        .bind(FileStatus::Deleted as i8)
        .execute(&mut **tx)
        .await?;

        self.logger
            .add(
                &LogFileDelete { user_id, file_id },
                Some(file_id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok(true)
    }

    // =========================================================================
    // 私有：Failed 文件残留清理
    // =========================================================================

    /// 清理 Failed 状态文件的所有磁盘残留。
    ///
    /// Failed 文件可能有两种残留：
    /// 1. 分片上传：`lst_file_local_chunk` 中每个分片的 `chunk_path` 磁盘文件
    /// 2. `lst_file_local` 中的 `local_path`（合并失败时可能已写入部分）
    ///
    /// 此方法为 fire-and-forget，内部错误只记日志。
    async fn cleanup_failed_file(&self, file_id: u64, file: &FileModel) {
        // ── 1. 清理分片残留文件 ───────────────────────────────────────
        match self.helper.find_chunks_by_file_id(file_id).await {
            Ok(chunks) => {
                for chunk in &chunks {
                    if chunk.chunk_path.is_empty() {
                        continue;
                    }
                    let full = self
                        .helper
                        .get_full_local_path(&file.storage_type, &chunk.chunk_path)
                        .await
                        .unwrap_or_else(|_| PathBuf::from(&chunk.chunk_path));
                    match tokio::fs::remove_file(&full).await {
                        Ok(_) => {
                            info!(
                                "cleanup_failed_file: removed chunk file {:?} (file_id={}, chunk_id={})",
                                full, file_id, chunk.id
                            );
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                            // 已不存在，跳过
                        }
                        Err(e) => {
                            warn!(
                                "cleanup_failed_file: failed to remove chunk {:?}: {} (file_id={}, chunk_id={})",
                                full, e, file_id, chunk.id
                            );
                        }
                    }
                }
            }
            Err(e) => {
                warn!(
                    "cleanup_failed_file: query chunks failed for file_id={}: {}",
                    file_id, e
                );
            }
        }

        // ── 2. 清理 local_path（合并失败时可能已写部分内容） ─────────
        match self.helper.find_file_local_by_file_id(file_id).await {
            Ok(Some(local)) if !local.local_path.is_empty() => {
                let full = self
                    .helper
                    .get_full_local_path(&file.storage_type, &local.local_path)
                    .await
                    .unwrap_or_else(|_| PathBuf::from(&local.local_path));
                match tokio::fs::remove_file(&full).await {
                    Ok(_) => {
                        info!(
                            "cleanup_failed_file: removed partial local_path {:?} (file_id={})",
                            full, file_id
                        );
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => {
                        warn!(
                            "cleanup_failed_file: failed to remove local_path {:?}: {} (file_id={})",
                            full, e, file_id
                        );
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                warn!(
                    "cleanup_failed_file: query file_local failed for file_id={}: {}",
                    file_id, e
                );
            }
        }

        self.log_dao
            .add(
                file_id,
                0,
                0,
                "cleanup_failed_file: residual cleanup done",
                None,
            )
            .await;
    }

    // =========================================================================
    // 私有：物理文件清理分发
    // =========================================================================

    /// 尝试清理物理文件（本地磁盘或 OSS），按 `file.is_local()` 分发。
    ///
    /// 此方法为 fire-and-forget：内部错误只记日志，不向上传播。
    /// 调用时传入的 `file` 应已处于软删状态（`status = Deleted`）。
    async fn try_cleanup_physical_file(
        &self,
        file_id: u64,
        file: Option<&FileModel>,
        oss_provider: Option<&dyn OssProvider>,
    ) {
        // 若未传入 FileModel，从 DB 加载
        let owned;
        let file = match file {
            Some(f) => f,
            None => match self.helper.find_file_by_id(file_id).await {
                Ok(Some(f)) => {
                    owned = f;
                    &owned
                }
                Ok(None) => {
                    warn!(
                        "try_cleanup_physical_file: file_id={} not found, skip",
                        file_id
                    );
                    return;
                }
                Err(e) => {
                    warn!(
                        "try_cleanup_physical_file: db error for file_id={}: {}",
                        file_id, e
                    );
                    return;
                }
            },
        };

        info!(
            "try_cleanup_physical_file: file_id={}, is_local={}",
            file_id,
            file.is_local()
        );

        if file.is_local() {
            self.cleanup_local_file(file_id, file).await;
        } else {
            self.cleanup_oss_file(file_id, file, oss_provider).await;
        }
    }

    // =========================================================================
    // 私有：本地磁盘文件清理
    // =========================================================================

    /// 删除本地磁盘文件。
    ///
    /// 删除前检查是否有其他活跃记录共享同一物理路径（`local_path`），
    /// 若有则跳过物理删除。
    async fn cleanup_local_file(&self, file_id: u64, file: &FileModel) {
        let local = match self.helper.find_file_local_by_file_id(file_id).await {
            Ok(Some(l)) => l,
            Ok(None) => {
                warn!(
                    "cleanup_local_file: file_local not found for file_id={}",
                    file_id
                );
                return;
            }
            Err(e) => {
                warn!(
                    "cleanup_local_file: db error for file_id={}: {}",
                    file_id, e
                );
                return;
            }
        };

        if local.local_path.is_empty() {
            info!(
                "cleanup_local_file: local_path is empty for file_id={}, skip",
                file_id
            );
            return;
        }

        // 检查是否有其他活跃记录共享同一物理路径
        let shared_refs: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM {} fl \
             INNER JOIN {} f ON fl.file_id = f.id \
             WHERE fl.local_path=? AND f.status!=? AND f.id!=?",
            FileLocalModel::table_name(),
            FileModel::table_name(),
        ))
        .bind(&local.local_path)
        .bind(FileStatus::Deleted as i8)
        .bind(file_id)
        .fetch_one(&self.helper.db)
        .await
        .unwrap_or(0);

        if shared_refs > 0 {
            info!(
                "cleanup_local_file: file_id={} local_path shared by {} other(s), skip physical delete",
                file_id, shared_refs
            );
            self.log_dao
                .add(
                    file_id,
                    0,
                    0,
                    &format!(
                        "delete: skip physical delete, shared local_path refs={}",
                        shared_refs
                    ),
                    None,
                )
                .await;
            return;
        }

        let full_path = self
            .helper
            .get_full_local_path(&file.storage_type, &local.local_path)
            .await
            .unwrap_or_else(|_| PathBuf::from(&local.local_path));

        info!("cleanup_local_file: removing {:?}", full_path);

        match tokio::fs::remove_file(&full_path).await {
            Ok(_) => {
                info!("cleanup_local_file: successfully deleted {:?}", full_path);
                self.log_dao
                    .add(file_id, 0, 0, "delete: physical file deleted", None)
                    .await;
            }
            Err(e) => {
                warn!("cleanup_local_file: failed to delete {:?}: {}", full_path, e);
                self.log_dao
                    .add(
                        file_id,
                        0,
                        0,
                        &format!("delete: physical delete failed: {}", e),
                        None,
                    )
                    .await;
            }
        }
    }

    // =========================================================================
    // 私有：OSS 对象清理
    // =========================================================================

    /// 删除 OSS 对象。
    ///
    /// 删除前检查是否有其他活跃记录引用同一 `object_key`（相同 storage_type），
    /// 若有则跳过删除（同 MD5 但不同 object_key 的独立上传视为独立对象）。
    async fn cleanup_oss_file(
        &self,
        file_id: u64,
        file: &FileModel,
        oss_provider: Option<&dyn OssProvider>,
    ) {
        let Some(provider) = oss_provider else {
            info!(
                "cleanup_oss_file: no OSS provider available for file_id={}, skip",
                file_id
            );
            return;
        };

        let oss = match self.helper.find_file_oss_by_file_id(file_id).await {
            Ok(Some(o)) => o,
            Ok(None) => {
                info!(
                    "cleanup_oss_file: no OSS record for file_id={}, skip",
                    file_id
                );
                return;
            }
            Err(e) => {
                warn!(
                    "cleanup_oss_file: db error for file_id={}: {}",
                    file_id, e
                );
                return;
            }
        };

        // 检查是否有其他活跃记录引用同一 object_key（同 storage_type）
        if !oss.object_key.is_empty() {
            let shared_refs: i64 = sqlx::query_scalar(&format!(
                "SELECT COUNT(*) FROM {} fo \
                 INNER JOIN {} f ON fo.file_id = f.id \
                 WHERE fo.object_key=? AND f.storage_type=? AND f.status!=? AND f.id!=?",
                FileOssModel::table_name(),
                FileModel::table_name(),
            ))
            .bind(&oss.object_key)
            .bind(&file.storage_type)
            .bind(FileStatus::Deleted as i8)
            .bind(file_id)
            .fetch_one(&self.helper.db)
            .await
            .unwrap_or(0);

            if shared_refs > 0 {
                info!(
                    "cleanup_oss_file: file_id={} object_key shared by {} other(s), skip OSS delete",
                    file_id, shared_refs
                );
                self.log_dao
                    .add(
                        file_id,
                        0,
                        0,
                        &format!(
                            "delete: skip OSS delete, object_key refs={}",
                            shared_refs
                        ),
                        None,
                    )
                    .await;
                return;
            }
        }

        match provider.delete_object(&oss).await {
            Ok(_) => {
                info!(
                    "cleanup_oss_file: OSS object deleted for file_id={}",
                    file_id
                );
                self.log_dao
                    .add(file_id, 0, 0, "delete: OSS object deleted", None)
                    .await;
            }
            Err(e) => {
                warn!(
                    "cleanup_oss_file: OSS delete failed for file_id={}: {}",
                    file_id, e
                );
                self.log_dao
                    .add(
                        file_id,
                        0,
                        0,
                        &format!("delete: OSS delete failed: {}", e),
                        None,
                    )
                    .await;
            }
        }
    }
}
