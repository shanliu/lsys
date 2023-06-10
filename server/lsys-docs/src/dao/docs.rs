use git2::Repository;

use lsys_core::{now_time, PageParam};
use lsys_setting::dao::SingleSetting;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySql, Pool, Row};
use sqlx_model::{
    model_option_set, sql_format, Insert, ModelTableName, Select, Update, WhereOption,
};
use tokio::fs;
use url::Url;

use crate::{
    dao::{DocSetting, GitDocError},
    model::{
        DocBuildModel, DocBuildStatus, DocCloneModel, DocGitModel, DocGitModelRef, DocGitStatus,
        DocLogsModel, DocMenuModel, DocMenuModelRef, DocMenuStatus,
    },
};
use sqlx_model::SqlQuote;
use std::{format, path::PathBuf, sync::Arc};

use super::GitDocResult;

// 发送任务抽象实现
pub struct GitDocs {
    db: Pool<MySql>,
    setting: Arc<SingleSetting>,
}

impl GitDocs {
    pub fn new(db: Pool<MySql>, setting: Arc<SingleSetting>) -> Self {
        Self { db, setting }
    }
}

#[derive(Serialize)]
pub struct GitDetail {
    pub branch: String,
    pub is_tag: bool,
    pub version: String,
}
impl GitDocs {
    /// 通知发送模块进行发送操作
    pub async fn git_detail(&self, url: String) -> GitDocResult<Vec<GitDetail>> {
        if let Err(err) = Url::parse(&url) {
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
                let (branch, is_tag) = if let Some(br) = handpath.strip_prefix("refs/tags/") {
                    (br.to_string(), true)
                } else if let Some(br) = handpath.strip_prefix("refs/heads/") {
                    (br.to_string(), false)
                } else {
                    continue;
                };
                out.push(GitDetail {
                    branch,
                    is_tag,
                    version: head.oid().to_string(),
                })
            }
            Ok(out)
        })
        .await;
        task.map_err(|err| GitDocError::System(format!("create git task fail:{}", err)))?
    }
}

pub struct GitDocsMenuItem {
    pub menu_path: String,
    pub access_path: Option<String>,
}
pub struct GitDocsGitAdd {
    pub url: String,
    pub branch: String,
    pub build_version: String,
    pub is_update: bool,
    pub is_tag: bool,
    pub menu_data: Vec<GitDocsMenuItem>,
    pub clear_rule: Option<Vec<String>>,
}

impl GitDocs {
    /// 通知发送模块进行发送操作
    pub async fn add_git(&self, param: &GitDocsGitAdd, user_id: u64) -> GitDocResult<u32> {
        if param.menu_data.is_empty() {
            return Err(crate::dao::GitDocError::System(
                "miss menu data ".to_string(),
            ));
        }
        for tmp in param.menu_data.iter() {
            if tmp.menu_path.trim().is_empty() {
                return Err(crate::dao::GitDocError::System(
                    "menu path can't be empty".to_string(),
                ));
            }
        }
        let url = match Url::parse(&param.url) {
            Ok(url) => url.to_string(),
            Err(err) => {
                return Err(crate::dao::GitDocError::System(format!(
                    "url parse fail:{}",
                    err
                )))
            }
        };

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
        if param.branch.trim().is_empty() {
            return Err(crate::dao::GitDocError::System(format!(
                "submit branch is empty:{}",
                param.branch
            )));
        }
        let mut db = self.db.begin().await?;
        let clear_rule = serde_json::json!(param.clear_rule).to_string();
        let is_update = if param.is_update { 1 } else { 0 };
        let is_tag = if param.is_tag { 1 } else { 0 };
        let status = DocGitStatus::Enable as i8;
        let add_time = now_time().unwrap_or_default();
        let vdata = model_option_set!(DocGitModelRef, {
            url: url,
            branch: param.branch,
            build_version:param.build_version,
            is_update:is_update,
            is_tag:is_tag,
            clear_rule:clear_rule,
            status:status,
            finish_time:0,
            change_user_id:user_id,
            change_time:add_time,
        });
        let id = match Insert::<sqlx::MySql, DocGitModel, _>::new(vdata)
            .execute(&mut db)
            .await
        {
            Ok(row) => row.last_insert_id(),
            Err(err) => {
                db.rollback().await?;
                return Err(err.into());
            }
        } as u32;
        let status = DocMenuStatus::Enable as i8;
        let mut vdatas = Vec::with_capacity(param.menu_data.len());
        let add_data = param
            .menu_data
            .iter()
            .map(|e| {
                (
                    e.menu_path.trim().to_string(),
                    e.access_path
                        .clone()
                        .map(|e| e.trim().to_string())
                        .unwrap_or_default(),
                )
            })
            .collect::<Vec<_>>();
        for (menu_path, access_path) in add_data.iter() {
            vdatas.push(model_option_set!(DocMenuModelRef, {
                doc_git_id: id ,
                build_version:param.build_version,
                menu_path:menu_path,
                access_path:access_path,
                status:status,
                finish_time:0,
                change_user_id:user_id,
                change_time:add_time,
            }));
        }
        if let Err(err) = Insert::<sqlx::MySql, DocMenuModel, _>::new_vec(vdatas)
            .execute(&mut db)
            .await
        {
            db.rollback().await?;
            return Err(err.into());
        };
        db.commit().await?;
        Ok(id)
    }
}
pub struct GitDocsGitEdit {
    pub branch: String,
    pub build_version: String,
    pub is_update: bool,
    pub is_tag: bool,
    pub menu_data: Vec<GitDocsMenuItem>,
    pub clear_rule: Option<Vec<String>>,
}
impl GitDocs {
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u32,
        DocGitModel,
        GitDocResult<DocGitModel>,
        id,
        "id={id}"
    );
    pub async fn edit_git(
        &self,
        doc_git: &DocGitModel,
        param: &GitDocsGitEdit,
        user_id: u64,
    ) -> GitDocResult<()> {
        if DocGitStatus::Enable.eq(doc_git.status) {
            return Err(crate::dao::GitDocError::System(
                "doc git not find ".to_string(),
            ));
        }
        if param.menu_data.is_empty() {
            return Err(crate::dao::GitDocError::System(
                "miss menu data ".to_string(),
            ));
        }
        for tmp in param.menu_data.iter() {
            if tmp.menu_path.trim().is_empty() {
                return Err(crate::dao::GitDocError::System(
                    "menu path can't be empty".to_string(),
                ));
            }
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
        if param.branch.trim().is_empty() {
            return Err(crate::dao::GitDocError::System(format!(
                "submit branch is empty:{}",
                param.branch
            )));
        }

        let menu_data = {
            let sql = sql_format!(
                "doc_git_id = {} and build_version={} and status=?",
                doc_git.id,
                doc_git.build_version
            );
            Select::type_new::<DocMenuModel>()
                .fetch_all_by_where_call::<DocMenuModel, _, _>(
                    &sql,
                    |e, _| e.bind(DocMenuStatus::Enable as i8),
                    &self.db,
                )
                .await?
        };

        let mut db = self.db.begin().await?;
        let clear_rule = serde_json::json!(param.clear_rule).to_string();
        let is_update = if param.is_update { 1 } else { 0 };
        let is_tag = if param.is_tag { 1 } else { 0 };
        let add_time = now_time().unwrap_or_default();
        let change = model_option_set!(DocGitModelRef, {
            branch: param.branch,
            build_version:param.build_version,
            is_update:is_update,
            is_tag:is_tag,
            clear_rule:clear_rule,
            finish_time:0,
            change_user_id:user_id,
            change_time:add_time,
        });
        let res = Update::<sqlx::MySql, DocGitModel, _>::new(change)
            .execute_by_pk(doc_git, &mut db)
            .await;
        if let Err(err) = res {
            db.rollback().await?;
            return Err(err.into());
        }
        let dsql = if doc_git.build_version != param.build_version || doc_git.branch != param.branch
        {
            sql_format!("doc_git_id={}", doc_git.id)
        } else {
            sql_format!(
                "doc_git_id={} and build_version={} and menu_path not in ({})",
                doc_git.id,
                doc_git.build_version,
                param
                    .menu_data
                    .iter()
                    .map(|e| e.menu_path.clone())
                    .collect::<Vec<String>>()
            )
        };
        let status = DocMenuStatus::Delete as i8;
        let change = model_option_set!(DocMenuModelRef, {
            status:status,
            change_user_id:user_id,
            change_time:add_time,
        });
        let res = Update::<sqlx::MySql, DocMenuModel, _>::new(change)
            .execute_by_where(&WhereOption::Where(dsql), &mut db)
            .await;
        if let Err(err) = res {
            db.rollback().await?;
            return Err(err.into());
        }
        let add = {
            let mut out = vec![];
            for tti in param.menu_data.iter() {
                if let Some(fi) = menu_data.iter().find(|e| e.menu_path == tti.menu_path) {
                    let accpath = tti.access_path.to_owned().unwrap_or_default();
                    let change = model_option_set!(DocMenuModelRef, {
                        access_path:accpath,
                        change_user_id:user_id,
                        change_time:add_time,
                    });
                    let res = Update::<sqlx::MySql, DocMenuModel, _>::new(change)
                        .execute_by_pk(fi, &mut db)
                        .await;
                    if let Err(err) = res {
                        db.rollback().await?;
                        return Err(err.into());
                    }
                } else {
                    out.push(tti)
                }
            }
            out
        };
        let mut vdatas = Vec::with_capacity(
            if doc_git.build_version != param.build_version || doc_git.branch != param.branch {
                param.menu_data.len()
            } else {
                add.len()
            },
        );
        let add_data =
            if doc_git.build_version != param.build_version || doc_git.branch != param.branch {
                param.menu_data.iter().collect::<Vec<_>>()
            } else {
                add
            }
            .iter()
            .map(|e| {
                (
                    e.menu_path.trim().to_string(),
                    e.access_path
                        .clone()
                        .map(|e| e.trim().to_string())
                        .unwrap_or_default(),
                )
            })
            .collect::<Vec<_>>();
        for (menu_path, access_path) in add_data.iter() {
            vdatas.push(model_option_set!(DocMenuModelRef, {
                doc_git_id: doc_git.id ,
                build_version:param.build_version,
                menu_path:menu_path,
                access_path:access_path,
                status:status,
                finish_time:0,
                change_user_id:user_id,
                change_time:add_time,
            }));
        }
        if let Err(err) = Insert::<sqlx::MySql, DocMenuModel, _>::new_vec(vdatas)
            .execute(&mut db)
            .await
        {
            db.rollback().await?;
            return Err(err.into());
        };
        db.commit().await?;
        Ok(())
    }
    /// 通知发送模块进行发送操作
    pub async fn git_notify(&self, doc_git: &DocGitModel) -> GitDocResult<()> {
        if doc_git.is_update == 0 {
            return Ok(());
        }
        let data = self.git_detail(doc_git.url.to_owned()).await?;
        for tmp in data {
            if tmp.branch == doc_git.branch {
                if tmp.version == doc_git.build_version {
                    return Ok(());
                }
                let mut db = self.db.begin().await?;
                let change = sqlx_model::model_option_set!(DocGitModelRef,{
                    branch: tmp.branch,
                    build_version: tmp.version,
                    finish_time:0,
                });
                let res = Update::<sqlx::MySql, DocGitModel, _>::new(change)
                    .execute_by_pk(doc_git, &mut db)
                    .await;
                if let Err(err) = res {
                    db.rollback().await?;
                    return Err(err.into());
                }
                let change = sqlx_model::model_option_set!(DocMenuModelRef,{
                    build_version: tmp.version,
                    finish_time:0,
                });
                let tmp = Update::<sqlx::MySql, DocMenuModel, _>::new(change)
                    .execute_by_where_call("doc_git_id=?", |e, _| e.bind(doc_git.id), &mut db)
                    .await;
                if let Err(err) = tmp {
                    db.rollback().await?;
                    return Err(err.into());
                }
                db.commit().await?;
                return Ok(());
            }
        }
        Err(crate::dao::GitDocError::System(format!(
            "not find branch[{}] on this git:{}",
            doc_git.build_version, doc_git.url
        )))
    }
}

#[derive(Serialize, Deserialize)]
pub struct DocItem {
    pub doc: DocGitModel,
    pub menu: Vec<DocMenuModel>,
}
impl GitDocs {
    /// 通知发送模块进行发送操作
    pub async fn list_data(&self) -> GitDocResult<Vec<DocItem>> {
        let data = Select::type_new::<DocGitModel>()
            .fetch_all_by_where_call::<DocGitModel, _, _>(
                "status=?",
                |e, _| e.bind(DocGitStatus::Enable as i8),
                &self.db,
            )
            .await?;

        let mut menu_data = if !data.is_empty() {
            let sql = sql_format!(
                "doc_git_id in ({}) and status=?",
                data.iter().map(|e| e.id).collect::<Vec<_>>()
            );
            Select::type_new::<DocMenuModel>()
                .fetch_all_by_where_call::<DocMenuModel, _, _>(
                    &sql,
                    |e, _| e.bind(DocMenuStatus::Enable as i8),
                    &self.db,
                )
                .await?
        } else {
            vec![]
        };
        let mut out = Vec::with_capacity(data.len());
        for tmp in data {
            let mout;
            (menu_data, mout) = menu_data.into_iter().partition(|e| e.doc_git_id != tmp.id);
            out.push(DocItem {
                doc: tmp,
                menu: mout,
            })
        }
        Ok(out)
    }
}
#[derive(Serialize, Deserialize)]
pub struct DocBuildItem {
    pub doc: DocCloneModel,
    pub build: Vec<(DocBuildModel, String)>,
    pub logs: Vec<DocLogsModel>,
}
impl GitDocs {
    pub async fn logs_count(&self, git_id: &u32, host: &Option<String>) -> GitDocResult<i64> {
        let mut sql = sql_format!(
            "select count(*) as total from {} where doc_git_id ={}",
            DocCloneModel::table_name(),
            git_id
        );
        if let Some(host) = host {
            sql += &sql_format!(" and host ={}", host)
        }
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    pub async fn logs_data(
        &self,
        git_id: &u32,
        host: &Option<String>,
        page: &Option<PageParam>,
    ) -> GitDocResult<Vec<DocBuildItem>> {
        let mut where_sql = sql_format!(" doc_git_id ={}", git_id);
        if let Some(host) = host {
            where_sql += &sql_format!(" and host ={}", host)
        }
        if let Some(pdat) = page {
            where_sql += &format!(
                " order by id desc limit {} offset {} ",
                pdat.limit, pdat.offset
            )
        } else {
            where_sql += " order by id desc"
        };

        let data = Select::type_new::<DocCloneModel>()
            .fetch_all_by_where::<DocCloneModel, _>(&WhereOption::Where(where_sql), &self.db)
            .await?;
        let mut build_data = if !data.is_empty() {
            let mut sql = sql_format!(
                "select build.*,menu.menu_path from {} as build join {} as menu on build.doc_menu_id =menu.id 
                    where doc_git_id ={} and build_version in ({}) and status={}",
                DocBuildModel::table_name(),
                DocMenuModel::table_name(),
                git_id,
                data.iter().map(|e| e.build_version.to_owned()).collect::<Vec<_>>(),
                DocGitStatus::Enable as i8
            );
            if let Some(host) = host {
                sql += &sql_format!(" and host ={}", host)
            }
            sqlx::query(sql.as_str())
                .try_map(|row: sqlx::mysql::MySqlRow| {
                    let menu_path = row
                        .try_get::<String, &str>("menu_path")
                        .unwrap_or_else(|_| "".to_string());
                    match DocBuildModel::from_row(&row) {
                        Ok(res) => Ok((res, menu_path)),
                        Err(e) => Err(e),
                    }
                })
                .fetch_all(&self.db)
                .await?
        } else {
            vec![]
        };
        let mut logs_data = if !data.is_empty() {
            let mut sql = sql_format!(
                "select * from {} 
                    where doc_git_id ={} and build_version in ({}) ",
                DocLogsModel::table_name(),
                git_id,
                data.iter()
                    .map(|e| e.build_version.to_owned())
                    .collect::<Vec<_>>(),
            );
            if let Some(host) = host {
                sql += &sql_format!(" and host ={}", host)
            }
            Select::type_new::<DocLogsModel>()
                .fetch_all_by_where::<DocLogsModel, _>(&WhereOption::Where(sql), &self.db)
                .await?
        } else {
            vec![]
        };
        let mut out = Vec::with_capacity(data.len());
        for tmp in data {
            let bout;
            let lout;
            (build_data, bout) = build_data
                .into_iter()
                .partition(|e| e.0.build_version != tmp.build_version);
            (logs_data, lout) = logs_data
                .into_iter()
                .partition(|e| e.build_version != tmp.build_version);
            out.push(DocBuildItem {
                doc: tmp,
                build: bout,
                logs: lout,
            })
        }
        Ok(out)
    }
}

#[derive(Serialize, Deserialize)]
pub struct DocFile {
    pub build_id: u32,
    pub path: PathBuf,
    pub version: String,
}
impl GitDocs {
    /// 通知发送模块进行发送操作
    pub async fn open_file(&self, menu_id: u64, path: &str) -> GitDocResult<DocFile> {
        let host_name = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let sql = sql_format!(
            "select menu.menu_path,menu.access_path,build.build_version,build.id
            from {}  as menu join {} as build on menu.id=build.doc_menu_id
            where menu.id={}  and menu.status={} and build.status in ({},{}) and build.host={} order by id desc limit 1
            ",
            DocMenuModel::table_name(),
            DocBuildModel::table_name(),
            menu_id,
            DocBuildStatus::Finish,
            DocBuildStatus::Succ,
            DocMenuStatus::Enable,
            host_name
        );
        let (menu_path, access_path, build_version, bid) =
            sqlx::query_as::<_, (String, String, String, u32)>(sql.as_str())
                .fetch_one(&self.db)
                .await?;
        let safe_path = self
            .create_path(&menu_id, &build_version, &Some(access_path))
            .await?;
        let file_path = self
            .create_path(
                &menu_id,
                &build_version,
                &PathBuf::from(menu_path)
                    .parent()
                    .map(|e| e.to_string_lossy().to_string()),
            )
            .await?;
        if file_path.starts_with(safe_path) {
            return Err(crate::dao::GitDocError::System(format!(
                "access fail on dir:{}",
                path,
            )));
        }
        Ok(DocFile {
            build_id: bid,
            path: file_path,
            version: build_version,
        })
    }
    async fn create_path(
        &self,
        menu_id: &u64,
        build_version: &str,
        sub_path: &Option<String>,
    ) -> GitDocResult<PathBuf> {
        let config = self.setting.load::<DocSetting>(&None).await?;
        let mut safe_path = config.save_dir.to_owned();
        safe_path += &format!(
            "{}{}_{}/",
            if safe_path.ends_with('/') || safe_path.ends_with('\\') {
                ""
            } else {
                "/"
            },
            menu_id,
            build_version
        );
        //PathBuf::from(access_path).parent() .to_string_lossy()
        if let Some(access_path) = sub_path {
            safe_path += access_path
        }
        fs::canonicalize(&safe_path)
            .await
            .map_err(|err| crate::dao::GitDocError::System(format!("parse path fail:{}", err)))
    }
}

#[derive(Debug, Serialize)]
pub struct MenuItem {
    pub build_id: u64,
    pub data: String,
}

impl GitDocs {
    /// 通知发送模块进行发送操作
    pub async fn menu(&self) -> GitDocResult<Vec<MenuItem>> {
        let md = Select::type_new::<DocMenuModel>()
            .fetch_all_by_where_call::<DocMenuModel, _, _>(
                "finish_time>0 and status=?",
                |e, _| e.bind(DocMenuStatus::Enable as i8),
                &self.db,
            )
            .await?;
        if md.is_empty() {
            return Ok(vec![]);
        }
        //todo 这开始
        let host_name = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let sql = md
            .iter()
            .map(|sq| {
                sql_format!(
                    "select * from {} where
                    doc_menu_id = {} and build_version = {}
                    and host={} and status in ({}) order by id desc limit 1
                ",
                    DocBuildModel::table_name(),
                    sq.id,
                    sq.build_version,
                    host_name,
                    &[DocBuildStatus::Finish as i8, DocBuildStatus::Succ as i8],
                )
            })
            .collect::<Vec<String>>();
        let mut md =
            sqlx::query_as::<_, DocBuildModel>(&format!("({})", sql.join(" )union all( ")))
                .fetch_all(&self.db)
                .await?;
        let nmid = md
            .iter()
            .filter(|e| !md.iter().any(|t| t.doc_menu_id == e.doc_menu_id))
            .collect::<Vec<_>>();
        if !nmid.is_empty() {
            let sql = nmid
                .iter()
                .map(|sq| {
                    sql_format!(
                        "select * from {} where
                        doc_menu_id = {}  and host={} and status in ({}) 
                         order by id desc limit 1
                ",
                        DocBuildModel::table_name(),
                        sq.id,
                        host_name,
                        &[DocBuildStatus::Finish as i8, DocBuildStatus::Succ as i8],
                    )
                })
                .collect::<Vec<String>>();
            let tmp =
                sqlx::query_as::<_, DocBuildModel>(&format!("({})", sql.join(" )union all( ")))
                    .fetch_all(&self.db)
                    .await?;
            md.extend(tmp);
        }
        let mut out = vec![];
        for tmp in md {
            //todo xxx
            out.push(MenuItem {
                build_id: tmp.id,
                data: tmp.build_data,
            });
        }
        Ok(out)
    }
}
