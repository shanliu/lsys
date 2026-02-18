use sqlx::{MySql, Pool};

use super::file_config::FileConfig;
use super::file_log::FileLogDao;

/// 辅助函数集合
pub struct FileHelper {
    pub(super) db: Pool<MySql>,
    pub(super) config: FileConfig,
    pub(super) log_dao: FileLogDao,
}

impl FileHelper {
    pub fn new(db: Pool<MySql>, config: FileConfig) -> Self {
        let log_dao = FileLogDao::new(db.clone());
        Self {
            db,
            config,
            log_dao,
        }
    }

    pub fn log_dao(&self) -> &FileLogDao {
        &self.log_dao
    }

    pub fn config(&self) -> &FileConfig {
        &self.config
    }

    pub fn db(&self) -> &Pool<MySql> {
        &self.db
    }
}

mod chunk;
mod complete;
mod http;
mod ops;
mod query;

// Re-export types
pub use chunk::{validate_chunks, ChunkInfo};
pub use http::UrlFileInfo;
