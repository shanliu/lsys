use git2::Repository;
use lsys_logger::dao::ChangeLogger;
use regex::Regex;
use serde_json::Value;
use tokio::fs::{read, read_dir};
use tracing::{debug, info, warn};
use url::Url;

use crate::{
    dao::{logger::LogDocClone, GitDocError},
    model::{
        DocGitCloneModel, DocGitCloneModelRef, DocGitCloneStatus, DocGitModel, DocGitModelRef,
        DocGitStatus, DocGitTagModel, DocGitTagModelRef, DocGitTagStatus, DocLogsModel,
        DocLogsModelRef, DocMenuModel, DocMenuModelRef, DocMenuStatus,
    },
};
use lsys_core::{now_time, AppCore, PageParam, RequestEnv};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use sqlx_model::{
    model_option_set, sql_format, Insert, ModelTableName, Select, Update, WhereOption,
};
use sqlx_model::{SqlExpr, SqlQuote};
use std::{
    collections::HashSet,
    env, format,
    path::{Path, PathBuf},
    sync::Arc,
};

use super::{
    git_doc_path,
    logger::{LogDocInfo, LogDocMenu, LogDocTag},
    GitDocResult, GitTask,
};

// 1. 添加doc_menu (任意一台完成即可)
//    先根据 doc_git_id+host[当前] 确定 status[doc_clone]已完成,在clone-id的目录加当前路径,可以过滤返回文件列表及目录 还有读取文件内容 ,当存在 根据doc_git_id的status出异常提示

// 2. 目录读取 (doc_menu)
//    读 doc_tag  status=[已启用] join doc_menu status=[正常] => doc_menu.menu_data + doc_menu.menu_path + doc_menu.id

// 3. 查看文件：
//    doc_menu id，访问路径路径[去除?.*] [RAW跟MD文件,MD文件返回JSON]
//    找到对应的记录数据，得到文件基础目录，加访问路径，整理路径，判断是否在安全目录,判断扩展名，读取文件，返回内容。header加版本

// 发送任务抽象实现
pub struct GitDocs {
    db: Pool<MySql>,
    task: Arc<GitTask>,
    app_core: Arc<AppCore>,
    logger: Arc<ChangeLogger>,
}

impl GitDocs {
    pub fn new(
        db: Pool<MySql>,
        app_core: Arc<AppCore>,
        logger: Arc<ChangeLogger>,
        task: Arc<GitTask>,
    ) -> Self {
        Self {
            db,
            app_core,
            logger,
            task,
        }
    }
}

#[derive(Serialize)]
pub struct GitDetail {
    pub tag: String,
    pub version: String,
}
impl GitDocs {
    /// 通知发送模块进行发送操作
    pub async fn git_detail(&self, url: &str) -> GitDocResult<Vec<GitDetail>> {
        if let Err(err) = Url::parse(url) {
            return Err(crate::dao::GitDocError::System(format!(
                "url parse fail:{}",
                err
            )));
        }
        let tmp_dir = tempfile::tempdir();
        let tmp_dir = match tmp_dir {
            Ok(dir) => dir,
            Err(err) => {
                return Err(crate::dao::GitDocError::System(format!(
                    "create tmp dir fail:{}",
                    err
                )));
            }
        };
        let url = url.to_owned();
        let task = tokio::task::spawn_blocking(move || {
            let repo = match Repository::init(tmp_dir.path()) {
                Ok(rep) => rep,
                Err(err) => {
                    return Err(crate::dao::GitDocError::System(format!(
                        "init git fail:{}",
                        err
                    )));
                }
            };
            let mut remote = match repo
                .find_remote(&url)
                .or_else(|_| repo.remote_anonymous(&url))
            {
                Ok(rep) => rep,
                Err(err) => {
                    return Err(crate::dao::GitDocError::System(format!(
                        "set remote fail:{}",
                        err
                    )));
                }
            };

            let connection = match remote.connect_auth(git2::Direction::Fetch, None, None) {
                Ok(conn) => conn,
                Err(err) => {
                    return Err(crate::dao::GitDocError::System(format!(
                        "connect git fail:{}",
                        err
                    )));
                }
            };
            let list_data = match connection.list() {
                Ok(head) => head,
                Err(err) => {
                    return Err(crate::dao::GitDocError::System(format!(
                        "get git head data fail:{}",
                        err
                    )));
                }
            };
            let mut out = vec![];
            // Get the list of references on the remote and print out their name next to
            // what they point to.
            for head in list_data.iter() {
                let handpath = head.name().to_string();
                let tag = if let Some(br) = handpath.strip_prefix("refs/tags/") {
                    br.to_string()
                } else {
                    continue;
                };
                out.push(GitDetail {
                    tag,
                    version: head.oid().to_string(),
                })
            }
            Ok(out)
        })
        .await;
        task.map_err(|err| GitDocError::System(format!("create git task fail:{}", err)))?
    }
}

pub struct GitDocsData {
    pub name: String,
    pub url: String,
    pub max_try: u8,
}

impl GitDocs {
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_git_by_id,
        u32,
        DocGitModel,
        GitDocResult<DocGitModel>,
        id,
        "id={id}"
    );
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_menu_by_id,
        u64,
        DocMenuModel,
        GitDocResult<DocMenuModel>,
        id,
        "id={id}"
    );
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_tag_by_id,
        u64,
        DocGitTagModel,
        GitDocResult<DocGitTagModel>,
        id,
        "id={id}"
    );
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_clone_by_id,
        u64,
        DocGitCloneModel,
        GitDocResult<DocGitCloneModel>,
        id,
        "id={id}"
    );
    /// 通知发送模块进行发送操作
    pub async fn git_add(
        &self,
        param: &GitDocsData,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> GitDocResult<u32> {
        let url = match Url::parse(&param.url) {
            Ok(url) => url.to_string(),
            Err(err) => {
                return Err(crate::dao::GitDocError::System(format!(
                    "url parse fail:{}",
                    err
                )))
            }
        };
        self.git_detail(&url).await?;
        let name = {
            if param.name.trim().is_empty() {
                return Err(crate::dao::GitDocError::System(
                    "name can't be empty".to_string(),
                ));
            }
            param.name.trim().to_string()
        };

        let status = DocGitStatus::Enable as i8;
        let add_time = now_time().unwrap_or_default();
        let vdata = model_option_set!(DocGitModelRef, {
            name:name,
            url: url,

            max_try:param.max_try,

            status:status,

            change_user_id:user_id,
            change_time:add_time,
        });
        let add_id = Insert::<sqlx::MySql, DocGitModel, _>::new(vdata)
            .execute(&self.db)
            .await?
            .last_insert_id();
        self.logger
            .add(
                &LogDocInfo {
                    action: "add",
                    name,
                    url,
                    max_try: param.max_try,
                },
                &Some(add_id),
                &Some(user_id),
                &Some(user_id),
                None,
                env_data,
            )
            .await;
        Ok(add_id as u32)
    }
    /// 通知发送模块进行发送操作
    pub async fn git_edit(
        &self,
        git_model: &DocGitModel,
        param: &GitDocsData,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> GitDocResult<()> {
        let url = match Url::parse(&param.url) {
            Ok(url) => url.to_string(),
            Err(err) => {
                return Err(crate::dao::GitDocError::System(format!(
                    "url parse fail:{}",
                    err
                )))
            }
        };
        let name = {
            if param.name.trim().is_empty() {
                return Err(crate::dao::GitDocError::System(
                    "name can't be empty".to_string(),
                ));
            }
            param.name.trim().to_string()
        };

        let tag_data = {
            Select::type_new::<DocGitTagModel>()
                .fetch_all_by_where::<DocGitTagModel, _>(
                    &WhereOption::Where(sql_format!(
                        "doc_git_id={} and status in ({})",
                        git_model.id,
                        &[DocGitTagStatus::Build as i8, DocGitTagStatus::Publish as i8]
                    )),
                    &self.db,
                )
                .await?
        };
        let data = self.git_detail(&param.url).await?;
        if !tag_data.is_empty() {
            for tmp in tag_data {
                if !data
                    .iter()
                    .any(|e| e.version == tmp.build_version && e.tag == tmp.tag)
                {
                    return Err(crate::dao::GitDocError::System(format!(
                        "can't update url to {} ,version not find:{} [{}]",
                        param.url, tmp.tag, tmp.build_version
                    )));
                }
            }
        }
        let add_time = now_time().unwrap_or_default();
        let change = model_option_set!(DocGitModelRef, {
            name:name,
            url: url,
            max_try:param.max_try,
            change_user_id:user_id,
            change_time:add_time,
        });
        Update::<sqlx::MySql, DocGitModel, _>::new(change)
            .execute_by_pk(git_model, &self.db)
            .await?;

        self.logger
            .add(
                &LogDocInfo {
                    action: "edit",
                    name,
                    url,
                    max_try: param.max_try,
                },
                &Some(git_model.id as u64),
                &Some(user_id),
                &Some(user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    /// 通知发送模块进行发送操作
    pub async fn git_del(
        &self,
        git_model: &DocGitModel,
        user_id: &u64,
        timeout: &u64,
        env_data: Option<&RequestEnv>,
    ) -> GitDocResult<()> {
        let clone_res = Select::type_new::<DocGitTagModel>()
            .fetch_all_by_where(
                &WhereOption::Where(sql_format!(
                    "status!={} and doc_git_id={}",
                    DocGitTagStatus::Delete as i8,
                    git_model.id
                )),
                &self.db,
            )
            .await?;
        for tmp in clone_res {
            self.tag_del(&tmp, user_id, timeout, env_data).await?;
        }
        let change_user_id = user_id.to_owned();
        let change_time = now_time().unwrap_or_default();
        let status = DocGitStatus::Delete as i8;
        let change = sqlx_model::model_option_set!(DocGitModelRef, {
            status:status,
            change_user_id:change_user_id,
            change_time:change_time
        });
        if let Err(err) = Update::<MySql, DocGitModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("id={}", git_model.id,)),
                &self.db,
            )
            .await
        {
            warn!("delete tag fail:{}", err);
            return Err(err.into());
        }
        self.logger
            .add(
                &LogDocInfo {
                    action: "del",
                    name: git_model.name.clone(),
                    url: git_model.url.clone(),
                    max_try: git_model.max_try,
                },
                &Some(git_model.id as u64),
                &Some(user_id.to_owned()),
                &Some(user_id.to_owned()),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    /// 通知发送模块进行发送操作
    pub async fn git_list(&self) -> GitDocResult<Vec<DocGitModel>> {
        Ok(Select::type_new::<DocGitModel>()
            .fetch_all_by_where::<DocGitModel, _>(
                &WhereOption::Where(sql_format!("status={}", DocGitStatus::Enable)),
                &self.db,
            )
            .await?)
    }
}
pub struct GitDocsGitTag {
    pub tag: String,
    pub build_version: String,
    pub clear_rule: Option<Vec<String>>,
}

impl GitDocs {
    pub async fn tag_add(
        &self,
        doc_git: &DocGitModel,
        param: &GitDocsGitTag,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> GitDocResult<u64> {
        if DocGitStatus::Delete.eq(doc_git.status) {
            return Err(crate::dao::GitDocError::System(
                "doc git not find ".to_string(),
            ));
        }

        if let Some(rule) = &param.clear_rule {
            for tmp in rule {
                if let Err(re) = Regex::new(tmp) {
                    return Err(crate::dao::GitDocError::System(format!(
                        "clear rule is vaild:{}",
                        re
                    )));
                }
            }
        }

        if let Ok(re) = Regex::new(r"^[0-9a-f]{40}$") {
            if !re.is_match(&param.build_version) {
                return Err(crate::dao::GitDocError::System(format!(
                    "submit build version is wrong:{}",
                    &param.build_version
                )));
            }
        }

        let data = self.git_detail(&doc_git.url).await?;
        if !data
            .iter()
            .any(|e| e.tag == param.tag && e.version == param.build_version)
        {
            return Err(crate::dao::GitDocError::System(format!(
                "git:{} not find your submit tag:{}[{}]",
                doc_git.url, param.tag, param.build_version
            )));
        }

        if param.tag.trim().is_empty() {
            return Err(crate::dao::GitDocError::System(format!(
                "submit tag is empty:{}",
                param.tag
            )));
        }

        let clear_rule = serde_json::to_string(&param.clear_rule)
            .map_err(|e| GitDocError::System(e.to_string()))?;
        let status = DocGitTagStatus::Build as i8;
        let add_time = now_time().unwrap_or_default();
        let vdata = model_option_set!(DocGitTagModelRef, {
            doc_git_id:doc_git.id,
            tag: param.tag,
            build_version:param.build_version,
            clear_rule:clear_rule,
            status:status,
            add_user_id:user_id,
            add_time:add_time,
        });
        let add_id = Insert::<sqlx::MySql, DocGitTagModel, _>::new(vdata)
            .execute(&self.db)
            .await?
            .last_insert_id();

        self.logger
            .add(
                &LogDocTag {
                    action: "add",
                    doc_git_id: doc_git.id,
                    tag: param.tag.clone(),
                    build_version: param.build_version.clone(),
                    clear_rule,
                },
                &Some(add_id),
                &Some(user_id),
                &Some(user_id),
                None,
                env_data,
            )
            .await;
        self.task.remote_clone().await;
        Ok(add_id)
    }
    pub async fn tag_del(
        &self,
        git_tag: &DocGitTagModel,
        user_id: &u64,
        timeout: &u64,
        env_data: Option<&RequestEnv>,
    ) -> GitDocResult<()> {
        let clone_res = Select::type_new::<DocGitCloneModel>()
            .fetch_all_by_where(
                &WhereOption::Where(sql_format!(
                    "status!={} and doc_tag_id={}",
                    DocGitCloneStatus::Delete as i8,
                    git_tag.id
                )),
                &self.db,
            )
            .await?;

        for tmp in clone_res {
            self.tag_clone_del(&tmp, timeout, user_id, env_data).await?;
        }
        let status = DocGitTagStatus::Delete as i8;
        let change = sqlx_model::model_option_set!(DocGitTagModelRef, { status: status });
        if let Err(err) = Update::<MySql, DocGitTagModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("id={}", git_tag.id,)),
                &self.db,
            )
            .await
        {
            warn!("delete tag fail:{}", err);
            return Err(err.into());
        }

        self.logger
            .add(
                &LogDocTag {
                    action: "del",
                    doc_git_id: git_tag.doc_git_id,
                    tag: git_tag.tag.clone(),
                    build_version: git_tag.build_version.clone(),
                    clear_rule: git_tag.clear_rule.clone(),
                },
                &Some(git_tag.id),
                &Some(user_id.to_owned()),
                &Some(user_id.to_owned()),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    pub async fn tag_clone_del(
        &self,
        git_clone: &DocGitCloneModel,
        timeout: &u64,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> GitDocResult<()> {
        if let Err(err) = self
            .task
            .remote_delete_clone(&git_clone.id, &git_clone.host, timeout)
            .await
        {
            info!("tag clone del fail:{}", err)
        };
        let rgit_clone = Select::type_new::<DocGitCloneModel>()
            .reload(git_clone, &self.db)
            .await?;
        if !DocGitCloneStatus::Delete.eq(rgit_clone.status) {
            let finish_time = now_time().unwrap_or_default();
            let status = DocGitCloneStatus::Delete as i8;
            let change = sqlx_model::model_option_set!(DocGitCloneModelRef, {
                finish_time: finish_time,
                status:status
            });
            if let Err(err) = Update::<MySql, DocGitCloneModel, _>::new(change)
                .execute_by_where(
                    &WhereOption::Where(sql_format!("id={}", rgit_clone.id,)),
                    &self.db,
                )
                .await
            {
                warn!("double delete clone  fail:{}", err);
                return Err(err.into());
            }
            let add_time = now_time().unwrap_or_default();
            let message = "double clear delete clone".to_string();
            let vdata = model_option_set!(DocLogsModelRef, {
                doc_tag_id:rgit_clone.doc_tag_id,
                doc_clone_id:rgit_clone.id,
                host:rgit_clone.host,
                message:message,
                add_time:add_time,
            });
            if let Err(err) = Insert::<sqlx::MySql, DocLogsModel, _>::new(vdata)
                .execute(&self.db)
                .await
            {
                info!("double delete clone add log fail:{}", err);
            }
        }

        self.logger
            .add(
                &LogDocClone {
                    doc_tag_id: git_clone.doc_tag_id,
                    action: "delete",
                    host: git_clone.host.clone(),
                },
                &Some(git_clone.id),
                &Some(user_id.to_owned()),
                &Some(user_id.to_owned()),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
pub struct DocTagItem {
    pub git_data: Option<DocGitModel>,
    pub tag_data: DocGitTagModel,
    pub clone_data: Vec<DocGitCloneModel>,
    pub menu_num: i64,
}
impl GitDocs {
    pub async fn tags_status(
        &self,
        git_tag: &DocGitTagModel,
        status: &DocGitTagStatus,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> GitDocResult<()> {
        if *status == DocGitTagStatus::Delete {
            return Err(crate::dao::GitDocError::System(format!(
                "plase use tags_del [{}]",
                git_tag.id
            )));
        }
        if status.eq(&DocGitTagStatus::Publish) {
            let data = self.menu_list(git_tag).await?;
            if data.is_empty() {
                return Err(crate::dao::GitDocError::System(format!(
                    "menu is empty,can't publish this tag [{}]",
                    git_tag.id
                )));
            }
        }
        let status = *status as i8;
        let change = sqlx_model::model_option_set!(DocGitTagModelRef, { status: status });
        if let Err(err) = Update::<MySql, DocGitTagModel, _>::new(change)
            .execute_by_pk(git_tag, &self.db)
            .await
        {
            warn!("change tag status fail:{}", err);
            return Err(err.into());
        }
        self.logger
            .add(
                &LogDocTag {
                    action: "status",
                    doc_git_id: git_tag.doc_git_id,
                    tag: git_tag.tag.clone(),
                    build_version: git_tag.build_version.clone(),
                    clear_rule: git_tag.clear_rule.clone(),
                },
                &Some(git_tag.id),
                &Some(*user_id),
                &Some(*user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    //TAG 列表
    pub async fn tags_list(
        &self,
        git_id: &Option<u32>,
        status: &Option<DocGitTagStatus>,
        key_word: &Option<String>, //find tag or build_version
        page: &Option<PageParam>,
    ) -> GitDocResult<Vec<DocTagItem>> {
        let mut where_sql = match status {
            Some(s) => sql_format!("status = {}", *s as i8),
            None => sql_format!(
                "status in ({})",
                &[DocGitTagStatus::Build as i8, DocGitTagStatus::Publish as i8,]
            ),
        };
        if let Some(git_id) = git_id {
            where_sql += &sql_format!(" and doc_git_id = {}", git_id);
        }
        if let Some(kw) = key_word {
            where_sql += &sql_format!(
                " and (tag like {} or  build_version like {})",
                format!("%{}%", kw),
                format!("%{}%", kw)
            );
        }
        if let Some(pdat) = page {
            where_sql += &format!(
                " order by id desc limit {} offset {} ",
                pdat.limit, pdat.offset
            )
        } else {
            where_sql += " order by id desc"
        };

        let data = Select::type_new::<DocGitTagModel>()
            .fetch_all_by_where::<DocGitTagModel, _>(&WhereOption::Where(where_sql), &self.db)
            .await?;
        let git_ids = data
            .iter()
            .map(|e| e.doc_git_id)
            .collect::<HashSet<u32>>()
            .into_iter()
            .collect::<Vec<_>>();
        let git_all_data = if !git_ids.is_empty() {
            Select::type_new::<DocGitModel>()
                .fetch_all_by_where::<DocGitModel, _>(
                    &WhereOption::Where(sql_format!("id in ({})", git_ids)),
                    &self.db,
                )
                .await?
        } else {
            vec![]
        };

        let git_tag_ids = data.iter().map(|e| e.id).collect::<Vec<_>>();
        let mut clone_all_data = if !git_tag_ids.is_empty() {
            Select::type_new::<DocGitCloneModel>()
                .fetch_all_by_where::<DocGitCloneModel, _>(
                    &WhereOption::Where(sql_format!(
                        "doc_tag_id in ({}) and status!={}",
                        git_tag_ids,
                        DocGitCloneStatus::Delete as i8
                    )),
                    &self.db,
                )
                .await?
        } else {
            vec![]
        };

        let menu_all_num = if !git_tag_ids.is_empty() {
            let sqls = sql_format!(
                "select doc_tag_id, count(*) as total from {} where doc_tag_id in ({}) and status={} group by doc_tag_id",
                DocMenuModel::table_name(),
                git_tag_ids,
                DocMenuStatus::Enable as i8
            );
            sqlx::query_as::<_, (u64, i64)>(&sqls)
                .fetch_all(&self.db)
                .await?
        } else {
            vec![]
        };
        let mut out: Vec<_> = Vec::with_capacity(data.len());
        for tmp in data {
            let clone_data;
            (clone_all_data, clone_data) = clone_all_data
                .into_iter()
                .partition(|e| e.doc_tag_id != tmp.id);
            let menu_num = menu_all_num
                .iter()
                .find(|e| e.0 == tmp.id)
                .map(|e| e.1)
                .unwrap_or(0);
            let git_data = git_all_data
                .iter()
                .find(|e| e.id == tmp.doc_git_id)
                .map(|e| e.to_owned());
            out.push(DocTagItem {
                git_data,
                tag_data: tmp,
                clone_data,
                menu_num,
            })
        }
        Ok(out)
    }
    //TAG总数
    pub async fn tags_count(
        &self,
        git_id: &Option<u32>,
        status: &Option<DocGitTagStatus>,
        key_word: &Option<String>, //find tag or build_version
    ) -> GitDocResult<i64> {
        let mut where_sql = match status {
            Some(s) => sql_format!("status = {}", *s as i8),
            None => sql_format!(
                "status in ({})",
                &[DocGitTagStatus::Build as i8, DocGitTagStatus::Publish as i8,]
            ),
        };
        if let Some(git_id) = git_id {
            where_sql += &sql_format!(" and doc_git_id = {}", git_id);
        }
        if let Some(kw) = key_word {
            where_sql += &sql_format!(
                " and (tag like {} or  build_version like {})",
                format!("%{}%", kw),
                format!("%{}%", kw)
            );
        }
        let sqls = sql_format!(
            "select count(*) as total from {} where {}",
            DocGitTagModel::table_name(),
            SqlExpr(where_sql)
        );
        Ok(sqlx::query_scalar::<_, i64>(&sqls)
            .fetch_one(&self.db)
            .await?)
    }
    //指定TAG的日志
    pub async fn tags_logs(&self, git_tag_id: &u32) -> GitDocResult<Vec<DocLogsModel>> {
        Ok(Select::type_new::<DocLogsModel>()
            .fetch_all_by_where::<DocLogsModel, _>(
                &WhereOption::Where(sql_format!("doc_tag_id = {}", git_tag_id)),
                &self.db,
            )
            .await?)
    }
}
#[derive(Debug, Serialize)]
pub struct DocDirPath {
    pub clone_id: u64,
    pub url_path: PathBuf, //相对路径
    pub is_dir: bool,
}

impl GitDocs {
    pub async fn menu_file_list(
        &self,
        tag: &DocGitTagModel,
        prefix: &str,
    ) -> GitDocResult<Vec<DocDirPath>> {
        let host_name = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let clone_data = Select::type_new::<DocGitCloneModel>()
            .fetch_one_by_where::<DocGitCloneModel, _>(
                &WhereOption::Where(sql_format!(
                    "doc_tag_id={}   and host={}",
                    tag.id,
                    host_name
                )),
                &self.db,
            )
            .await?;

        if !DocGitCloneStatus::Cloned.eq(clone_data.status) {
            return Err(crate::dao::GitDocError::System(format!(
                "tag {} [{}] is clone not yet on:{}",
                tag.tag, tag.id, host_name
            )));
        }

        let config_dir = self
            .app_core
            .config
            .get_string("doc_git_dir")
            .unwrap_or_else(|_| env::temp_dir().to_string_lossy().to_string());

        let safe_path = git_doc_path(&config_dir, &clone_data.id, &None).await?;
        let file_path =
            git_doc_path(&config_dir, &clone_data.id, &Some(prefix.to_string())).await?;

        if !prefix.is_empty() && !file_path.starts_with(&safe_path) {
            return Err(crate::dao::GitDocError::System(format!(
                "access fail on dir:{}",
                prefix,
            )));
        }
        debug!(
            "list dir :{} on tag:{}",
            file_path.to_string_lossy(),
            tag.id
        );
        let mut out = vec![];
        let dir = Path::new(&file_path);
        // 判断是否是目录
        if dir.is_dir() {
            let mut entries = read_dir(dir)
                .await
                .map_err(|e| GitDocError::System(e.to_string()))?;
            while let Ok(Some(entry)) = entries.next_entry().await {
                // 获取路径
                let path = entry.path();
                let mut tmp = file_path.clone();
                tmp.push(&path);
                if let Ok(rpath) = tmp.strip_prefix(&safe_path) {
                    out.push(DocDirPath {
                        clone_id: clone_data.id,
                        url_path: rpath.to_path_buf(),
                        is_dir: path.is_dir(),
                    })
                }
            }
        }
        Ok(out)
    }
    pub async fn menu_file_read(
        &self,
        tag: &DocGitTagModel,
        menu_file: &str,
    ) -> GitDocResult<DocPath> {
        let host_name = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let clone_data = Select::type_new::<DocGitCloneModel>()
            .fetch_one_by_where::<DocGitCloneModel, _>(
                &WhereOption::Where(sql_format!("doc_tag_id={}  and host={}", tag.id, host_name)),
                &self.db,
            )
            .await?;
        if !DocGitCloneStatus::Cloned.eq(clone_data.status) {
            return Err(crate::dao::GitDocError::System(format!(
                "tag {} [{}] is clone not yet on:{}",
                tag.tag, tag.id, host_name
            )));
        }
        let config_dir = self
            .app_core
            .config
            .get_string("doc_git_dir")
            .unwrap_or_else(|_| env::temp_dir().to_string_lossy().to_string());

        let safe_path = git_doc_path(&config_dir, &clone_data.id, &None).await?;
        let file_path =
            git_doc_path(&config_dir, &clone_data.id, &Some(menu_file.to_owned())).await?;
        if !file_path.starts_with(&safe_path) {
            return Err(crate::dao::GitDocError::System(format!(
                "access fail on file:{}",
                menu_file,
            )));
        }
        if !file_path.is_file() {
            return Err(crate::dao::GitDocError::System(format!(
                "file not find:{}",
                file_path.to_string_lossy(),
            )));
        }
        let rpath = match file_path.strip_prefix(&safe_path) {
            Ok(rpath) => rpath.to_path_buf(),
            Err(_) => file_path.to_owned(),
        };
        Ok(DocPath {
            clone_id: clone_data.id,
            url_path: rpath,
            file_path,
            version: tag.build_version.clone(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct DocPath {
    pub clone_id: u64,
    pub url_path: PathBuf,  //相对路径
    pub file_path: PathBuf, //绝对路径
    pub version: String,
}

pub struct GitDocsMenuData {
    pub menu_path: String,
}
impl GitDocs {
    pub async fn menu_add(
        &self,
        tag: &DocGitTagModel,
        menu_param: &GitDocsMenuData,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> GitDocResult<u64> {
        if menu_param.menu_path.trim().is_empty() {
            return Err(crate::dao::GitDocError::System(
                "menu path can't be empty".to_string(),
            ));
        }
        let menu_file = self.menu_file_read(tag, &menu_param.menu_path).await?;
        let dat_u8 = read(&menu_file.file_path)
            .await
            .map_err(|e| GitDocError::System(format!("your sumbit path,can't read data:{}", e)))?;
        let dat_str = String::from_utf8_lossy(&dat_u8);

        if dat_str.trim().is_empty() || dat_str.trim() == "{}" {
            return Err(crate::dao::GitDocError::System(
                "can't add empty menu".to_string(),
            ));
        }
        if let Err(err) = serde_json::from_slice::<Value>(&dat_u8) {
            return Err(crate::dao::GitDocError::System(format!(
                "menu check fail:{}",
                err
            )));
        }
        let menu_path = menu_param.menu_path.trim().to_string();

        let sql = sql_format!(
            "doc_tag_id = {} and menu_path={} and status={}",
            tag.id,
            menu_path,
            DocMenuStatus::Enable as i8
        );
        match Select::type_new::<DocMenuModel>()
            .fetch_one_by_where::<DocMenuModel, _>(&WhereOption::Where(sql), &self.db)
            .await
        {
            Ok(id) => {
                return Err(GitDocError::System(format!(
                    "path[{}] is add,on:{}",
                    id.menu_path, id.id
                )))
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => return Err(err.into()),
        };

        let host_name = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let status = DocMenuStatus::Enable as i8;
        let add_time = now_time().unwrap_or_default();
        let vdata = model_option_set!(DocMenuModelRef, {
            doc_tag_id:tag.id,
            menu_path: menu_path,
            menu_check_host:host_name,
            status:status,
            add_user_id:user_id,
            add_time:add_time,
        });
        let add_id = Insert::<sqlx::MySql, DocMenuModel, _>::new(vdata)
            .execute(&self.db)
            .await?
            .last_insert_id();

        self.logger
            .add(
                &LogDocMenu {
                    action: "add",
                    doc_tag_id: tag.id,
                    menu_path,
                    menu_check_host: host_name,
                },
                &Some(add_id),
                &Some(user_id.to_owned()),
                &Some(user_id.to_owned()),
                None,
                env_data,
            )
            .await;
        Ok(add_id)
    }
    pub async fn menu_del<'t>(
        &self,
        menu: &DocMenuModel,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> GitDocResult<()> {
        let status = DocMenuStatus::Delete as i8;
        let change = sqlx_model::model_option_set!(DocMenuModelRef, { status: status });
        if let Err(err) = Update::<MySql, DocMenuModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("id={}", menu.id,)),
                &self.db,
            )
            .await
        {
            warn!(" delete menu  fail:{}", err);
            return Err(err.into());
        }
        self.logger
            .add(
                &LogDocMenu {
                    action: "del",
                    doc_tag_id: menu.doc_tag_id,
                    menu_path: menu.menu_path.clone(),
                    menu_check_host: menu.menu_check_host.clone(),
                },
                &Some(menu.id),
                &Some(user_id.to_owned()),
                &Some(user_id.to_owned()),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    pub async fn menu_list<'t>(&self, tag: &DocGitTagModel) -> GitDocResult<Vec<DocMenuModel>> {
        let sql = sql_format!(
            "doc_tag_id = {} and status={} order by id desc",
            tag.id,
            DocMenuStatus::Enable as i8
        );
        Ok(Select::type_new::<DocMenuModel>()
            .fetch_all_by_where::<DocMenuModel, _>(&WhereOption::Where(sql), &self.db)
            .await?)
    }
}

impl GitDocs {
    /// 通知发送模块进行发送操作
    pub async fn menu_file(&self, menu_id: u32, path: &str) -> GitDocResult<DocPath> {
        let host_name = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let sql = sql_format!(
            "select menu.menu_path,clone.id,tag.build_version
            from {}  as menu 
            join {} as tag on menu.doc_tag_id=tag.id
            join {} as clone on menu.doc_tag_id=clone.doc_tag_id
            where menu.id={}  and clone.status={} and tag.status ={} and clone.host={}
            order by id desc limit 1
            ",
            DocMenuModel::table_name(),
            DocGitTagModel::table_name(),
            DocGitCloneModel::table_name(),
            menu_id,
            DocGitCloneStatus::Cloned as i8,
            DocGitTagStatus::Publish as i8,
            host_name
        );

        let (menu_path, clone_id, version) =
            sqlx::query_as::<_, (String, u64, String)>(sql.as_str())
                .fetch_one(&self.db)
                .await?;
        let mut rel_file_path = PathBuf::from(menu_path)
            .parent()
            .map(|e| e.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("./"));
        rel_file_path.push(PathBuf::from(path));

        let config_dir = self
            .app_core
            .config
            .get_string("doc_git_dir")
            .unwrap_or_else(|_| env::temp_dir().to_string_lossy().to_string());
        let safe_path = git_doc_path(&config_dir, &clone_id, &None).await?;
        let file_path = git_doc_path(
            &config_dir,
            &clone_id,
            &Some(rel_file_path.to_string_lossy().to_string()),
        )
        .await?;
        if !file_path.starts_with(&safe_path) {
            return Err(crate::dao::GitDocError::System(format!(
                "access fail on file:{:?}",
                file_path,
            )));
        }
        let url_path = match file_path.strip_prefix(&safe_path) {
            Ok(rpath) => rpath.to_path_buf(),
            Err(_) => file_path.to_owned(),
        };

        Ok(DocPath {
            clone_id,
            url_path,
            file_path,
            version,
        })
    }
}

#[derive(Serialize)]
pub struct GitDocMenuData {
    pub tag_id: u64,
    pub menu_id: u64,
    pub menu_path: String,
    pub menu_data: Result<Vec<u8>, String>,
    pub version: String,
}

impl GitDocs {
    /// 通知发送模块进行发送操作
    pub async fn menu(&self) -> GitDocResult<Vec<GitDocMenuData>> {
        let sql = sql_format!(
            "select menu.id,tag.id,menu.menu_path,tag.build_version from {} as tag join {} as menu on tag.id=menu.doc_tag_id
             where  tag.status ={} and menu.status = {}
        ",
            DocGitTagModel::table_name(),
            DocMenuModel::table_name(),
            DocGitTagStatus::Publish as i8,
            DocMenuStatus::Enable as i8,
        );
        let data = sqlx::query_as::<_, (u64, u64, String, String)>(&sql)
            .fetch_all(&self.db)
            .await?;
        let clone_data = if !data.is_empty() {
            let host_name = hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let sql = sql_format!(
                "select id,doc_tag_id from {} where status={} and host={} and doc_tag_id in ({})
            ",
                DocGitCloneModel::table_name(),
                DocGitCloneStatus::Cloned as i8,
                host_name,
                data.iter().map(|e| e.1).collect::<Vec<u64>>()
            );
            sqlx::query_as::<_, (u64, u64)>(&sql)
                .fetch_all(&self.db)
                .await?
        } else {
            vec![]
        };

        let mut out = Vec::with_capacity(data.len());

        let config_dir = self
            .app_core
            .config
            .get_string("doc_git_dir")
            .unwrap_or_else(|_| env::temp_dir().to_string_lossy().to_string());

        for (menu_id, tag_id, menu_path, version) in data {
            let menu_data = match clone_data.iter().find(|e| e.1 == tag_id) {
                Some((clone_id, _)) => {
                    let safe_path = git_doc_path(&config_dir, clone_id, &None).await?;
                    let file_path =
                        git_doc_path(&config_dir, clone_id, &Some(menu_path.clone())).await?;
                    if !file_path.starts_with(&safe_path) {
                        Err(format!("access fail on file:{:?}", file_path,))
                    } else {
                        read(&file_path).await.map_err(|e| e.to_string())
                    }
                }
                None => Err(format!("tag {} find clone", tag_id,)),
            };
            out.push(GitDocMenuData {
                tag_id,
                menu_id,
                menu_data,
                menu_path,
                version,
            })
        }
        Ok(out)
    }
}
