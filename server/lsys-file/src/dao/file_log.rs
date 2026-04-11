use lsys_core::db::{Insert, OptionTxExecutor, utils::FetchField};
use lsys_core::utils::now_time;
use sqlx::{MySql, Pool, Transaction};
use tracing::{debug, warn};

use crate::model::FileLogModel;

/// 文件日志 DAO
#[derive(Clone)]
pub struct FileLogDao {
    db: Pool<MySql>,
}

impl FileLogDao {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }

    /// 添加文件日志
    pub async fn add(
        &self,
        file_id: u64,
        file_chunk_id: u64,
        user_id: u64,
        message: &str,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) {
        let time = now_time().unwrap_or_default();
        let msg_max = FetchField::new(&self.db)
            .string_max::<FileLogModel>(&FileLogModel::MESSAGE)
            .await
            .len_or(1024);
        let msg: String = message.chars().take(msg_max as usize).collect();

        let res = Insert::<_, FileLogModel>::new()
            .set(FileLogModel::FILE_ID, file_id)
            .set(FileLogModel::FILE_CHUNK_ID, file_chunk_id)
            .set(FileLogModel::MESSAGE, msg)
            .set(FileLogModel::USER_ID, user_id)
            .set(FileLogModel::ADD_TIME, time)
            .execute(OptionTxExecutor::new(transaction, &self.db))
            .await;
        match res {
            Err(err) => warn!("add file log fail:{}", err),
            Ok(r) => debug!("add file log id:{}", r.last_insert_id()),
        };
    }
}
