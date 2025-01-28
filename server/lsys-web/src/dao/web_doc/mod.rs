use crate::common::JsonResult;
use lsys_core::{AppCore, RemoteNotify};
use lsys_docs::dao::DocPath;
use lsys_docs::dao::{DocsDao, GitRemoteTask};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use tokio::fs::read;
use tracing::debug;
pub struct WebDoc {
    pub docs_dao: DocsDao,
}

impl WebDoc {
    pub async fn new(
        app_core: Arc<AppCore>,
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        change_logger: Arc<ChangeLoggerDao>,
    ) -> WebDoc {
        let doc_dir = app_core.config.find(None).get_string("doc_git_dir").ok();
        let docs = DocsDao::new(
            // app_core.clone(),
            db.clone(),
            remote_notify.clone(),
            change_logger.clone(),
            None,
            doc_dir.as_deref(),
        )
        .await;
        // 文档后台同步任务
        let task_docs = docs.task.clone();
        tokio::spawn(async move {
            task_docs.dispatch().await;
        });

        remote_notify
            .push_run(Box::new(GitRemoteTask::new(docs.task.clone())))
            .await;

        WebDoc { docs_dao: docs }
    }
}

#[derive(serde::Serialize)]
pub struct DocsTagFileReuslt {
    pub id: u64,
    pub version: String,
    pub data: String,
}

impl WebDoc {
    //指定文档tag的文件信息
    pub async fn docs_tag_file_info(
        &self,
        tag_id: u64,
        file_path: &str,
    ) -> JsonResult<DocsTagFileReuslt> {
        let tag = self.docs_dao.docs.find_tag_by_id(&tag_id).await?;
        let data = self.docs_dao.docs.menu_file_read(&tag, file_path).await?;
        let dat = read(data.file_path).await?;
        Ok(DocsTagFileReuslt {
            id: data.clone_id,
            version: data.version,
            data: String::from_utf8_lossy(&dat).to_string(),
        })
    }
}
impl WebDoc {
    //读取指定菜单下url的文档内容
    pub async fn docs_md_read(&self, menu_id: u32, url: &str) -> JsonResult<(DocPath, String)> {
        let data = self.docs_dao.docs.menu_file(menu_id, url).await?;
        debug!("read markdown file:{}", &data.file_path.to_string_lossy());
        let dat = read(&data.file_path).await?;
        Ok((data, String::from_utf8_lossy(&dat).to_string()))
    }
}
