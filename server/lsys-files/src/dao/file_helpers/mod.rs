use sqlx::{MySql, Pool};

use super::file_config::FileConfig;

/// 辅助函数集合
pub struct FileHelper {
    pub(super) db: Pool<MySql>,
    pub(super) config: FileConfig,
}

impl FileHelper {
    pub fn new(db: Pool<MySql>, config: FileConfig) -> Self {
        Self { db, config }
    }
}

mod chunk;
mod complete;
mod http;
mod ops;
mod query;

// Re-export types
pub use chunk::ChunkInfo;
pub use http::UrlFileInfo;
