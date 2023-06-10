use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

mod docs;
mod git;
mod logger;
mod task;
pub use docs::*;

use lsys_core::AppCore;
use lsys_core::RemoteNotify;
use lsys_logger::dao::ChangeLogger;
use relative_path::RelativePath;
pub use task::*;

use sqlx::MySql;
use sqlx::Pool;

#[derive(Debug)]
pub enum GitDocError {
    Sqlx(sqlx::Error),
    Redis(String),
    System(String),
    Remote(String),
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

pub type GitDocResult<T> = Result<T, GitDocError>;

pub struct DocsDao {
    //内部依赖
    pub docs: Arc<GitDocs>,
    pub task: Arc<GitTask>,
}

impl DocsDao {
    pub async fn new(
        app_core: Arc<AppCore>,
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        logger: Arc<ChangeLogger>,
        task_size: Option<usize>,
    ) -> Self {
        let task = Arc::from(GitTask::new(
            app_core.clone(),
            db.clone(),
            remote_notify,
            task_size,
        ));
        let docs = Arc::from(GitDocs::new(db, app_core, logger, task.clone()));
        DocsDao { docs, task }
    }
}

async fn git_doc_path(
    save_dir: &str,
    clone_id: &u64,
    sub_path: &Option<String>,
) -> GitDocResult<PathBuf> {
    let mut clear_path = save_dir.to_owned();
    clear_path += &format!(
        "{}{}/",
        if clear_path.ends_with('/') || clear_path.ends_with('\\') {
            ""
        } else {
            "/"
        },
        clone_id,
    );
    //PathBuf::from(access_path).parent() .to_string_lossy()
    if let Some(file_path) = sub_path {
        if !file_path.trim().is_empty() {
            let file_path = file_path.trim();
            let file_path = file_path.strip_suffix('/').unwrap_or(file_path);
            let file_path = file_path.strip_suffix('\\').unwrap_or(file_path);
            clear_path += file_path;
        }
    }
    let mut path = Path::new("/").to_path_buf();
    path.push(RelativePath::new(&clear_path).to_logical_path(""));
    Ok(path)
}
