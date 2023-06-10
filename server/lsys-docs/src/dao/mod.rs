use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::sync::Arc;

mod docs;
mod task;
pub use docs::*;
use lsys_setting::dao::SettingDecode;
use lsys_setting::dao::SettingEncode;
use lsys_setting::dao::SettingError;
use lsys_setting::dao::SettingJson;
use lsys_setting::dao::SettingKey;
use lsys_setting::dao::SettingResult;
use lsys_setting::dao::SingleSetting;
use serde::Deserialize;
use serde::Serialize;
use sqlx::MySql;
use sqlx::Pool;
pub use task::*;

#[derive(Deserialize, Serialize)]
pub struct DocSetting {
    pub save_dir: String,
}

impl SettingKey for DocSetting {
    fn key<'t>() -> &'t str {
        "docs-setting"
    }
}

impl SettingDecode for DocSetting {
    fn decode(data: &str) -> SettingResult<Self> {
        SettingJson::decode(data)
    }
}
impl SettingEncode for DocSetting {
    fn encode(&self) -> String {
        SettingJson::encode(self)
    }
}
impl SettingJson<'_> for DocSetting {}

#[derive(Debug)]
pub enum GitDocError {
    Sqlx(sqlx::Error),
    Redis(String),
    System(String),
}
impl Display for GitDocError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for GitDocError {}

impl From<sqlx::Error> for GitDocError {
    fn from(err: sqlx::Error) -> Self {
        GitDocError::Sqlx(err)
    }
}
impl From<SettingError> for GitDocError {
    fn from(err: SettingError) -> Self {
        match err {
            SettingError::Sqlx(e) => GitDocError::Sqlx(e),
            SettingError::System(e) => GitDocError::System(e),
        }
    }
}

pub type GitDocResult<T> = Result<T, GitDocError>;

pub struct DocsDao {
    //内部依赖
    pub docs: Arc<GitDocs>,
    pub task: Arc<GitTask>,
}

impl DocsDao {
    pub async fn new(db: Pool<MySql>, setting: Arc<SingleSetting>) -> Self {
        let docs = Arc::from(GitDocs::new(db.clone(), setting));
        let task = Arc::from(GitTask::new(db));
        DocsDao { docs, task }
    }
}
