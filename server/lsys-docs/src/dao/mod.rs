use std::env;

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

mod docs;
mod git;
mod logger;
mod task;
pub use docs::*;

use lsys_core::RemoteNotify;
use lsys_logger::dao::ChangeLoggerDao;
use relative_path::RelativePath;
use sqlx::MySql;
pub use task::*;

use sqlx::Pool;
mod result;
pub use result::*;

pub struct DocsDao {
    //内部依赖
    pub docs: Arc<GitDocs>,
    pub task: Arc<GitTask>,
}

impl DocsDao {
    pub async fn new(
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        logger: Arc<ChangeLoggerDao>,
        task_size: Option<usize>,
        save_dir: Option<&str>,
    ) -> Self {
        let save_dir = save_dir.map(|e| e.to_string()).unwrap_or_else(|| {
            env::temp_dir().to_string_lossy().to_string()
            // let config_dir = config!(self.app_core.config)
            // .get_string("doc_git_dir")
            // .unwrap_or_else(|_| env::temp_dir().to_string_lossy().to_string());
        });
        let task = Arc::from(GitTask::new(
            // app_core.clone(),
            db.clone(),
            remote_notify,
            task_size,
            &save_dir,
        ));
        let docs = Arc::from(GitDocs::new(db, logger, task.clone(), &save_dir));
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
