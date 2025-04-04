use lsys_core::db::{Insert, ModelTableName, SqlExpr, Update, WhereOption};
use lsys_core::{
    fluent_message, now_time, IntoFluentMessage, LocalExecType, MsgSendBody, RemoteNotify,
    RemoteTask, ReplyWait,
};
use lsys_core::{model_option_set, sql_format};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{MySql, Pool};
use tokio::{
    fs::{self, remove_dir_all},
    sync::{
        mpsc::{self, error::TryRecvError, Receiver, Sender},
        Mutex,
    },
    task::{AbortHandle, JoinError, JoinSet},
    time::sleep,
};
use tracing::{debug, error, info, warn};

use crate::{
    dao::{git::git_clear, git_doc_path, GitDocError},
    model::{
        DocGitCloneModel, DocGitCloneModelRef, DocGitCloneStatus, DocGitModel, DocGitTagModel,
        DocLogsModel, DocLogsModelRef,
    },
};
use lsys_core::db::SqlQuote;
use std::{env, format, path::Path, sync::Arc, time::Duration};

use super::{git::git_download, CloneError, CloneResult, GitDocResult};

type TaskResult = (DocGitModel, DocGitTagModel, u64, CloneResult); //git model,tag model,clone id,clone result
type TaskIngData = (u64, u64, AbortHandle); //tag id ,clone id,abort hand

// 1. 添加 doc_tag 记录
//    redis 通知:构建任务

// 2. 构建任务
//    查找 doc_tag 中 status[未启用 已启用],且在当前机器没完成CLONE的数据
//    select * from doc_tag where id not in (select doc_tag_id from doc_clone where host='$self-host' and status='已克隆,克隆失败') and id not (tag task ids)
//    任务监听:tag model
//    创建 doc_clone 状态为 待克隆,加CLONE任务
//    以 doc_clone 的ID 为目录,进行CLONE,存在目录删除掉,失败时根据重试次数重试
//    完成或多次失败后,执行清理路径,更新, doc_clone 状态及 finish_time ,减CLONE任务

// 3. 指定HOST删除
//    redis通知所有节点删除操作,监听REDIS结果,节点接到删除任务,判断非本机忽略,本机检测 doc_git_id 是否在CLONE任务,返回:失败,不在,更改记录,删除文件,通知:构建任务,返回:完成

// 发送任务抽象实现
pub struct GitTask {
    db: Pool<MySql>,
    tx: Sender<()>,
    rx: Mutex<Option<Receiver<()>>>,
    task_size: usize,
    // app_core: Arc<AppCore>,
    task_ing: Arc<Mutex<Vec<TaskIngData>>>,
    remote_notify: Arc<RemoteNotify>,
    save_dir: String,
}

impl GitTask {
    pub fn new(
        // app_core: Arc<AppCore>,
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        task_size: Option<usize>,
        save_dir: &str,
    ) -> Self {
        let task_size = task_size.unwrap_or_else(num_cpus::get);
        let (tx, rx) = mpsc::channel::<()>(task_size);
        Self {
            tx,
            db,
            remote_notify,
            // app_core,
            rx: Mutex::new(Some(rx)),
            task_size,
            task_ing: Arc::new(Mutex::new(vec![])),
            save_dir: save_dir.to_string(),
        }
    }
    /// 通知发送模块进行发送操作
    pub fn notify(&self) -> GitDocResult<()> {
        match self.tx.try_send(()) {
            Ok(_) => Ok(()),
            Err(err) => match err {
                mpsc::error::TrySendError::Full(_) => Ok(()),
                mpsc::error::TrySendError::Closed(_) => {
                    warn!("git task is close");
                    Err(super::GitDocError::System(fluent_message!(
                        "doc-notify-channel-close",
                        err
                    )))
                }
            },
        }
    }
    //发现其中一个任务完成时处理
    async fn one_task_finish(
        db: &Pool<MySql>,
        task_ing: &Arc<Mutex<Vec<TaskIngData>>>, //进行中任务
        bad_task_res: &mut i32,                  //已完成的PANIC任务
        task_res: Result<TaskResult, JoinError>, //其中一个已完成任务
        run_size: &mut usize,                    //进行中任务数量
    ) -> bool //是否存在下一个已完成任务
    {
        let all_finish = task_ing
            .lock()
            .await
            .iter()
            .filter(|t| t.2.is_finished())
            .map(|t| (t.0, t.1))
            .collect::<Vec<_>>(); //当前已完成任务数量
        let next = all_finish.len() as i64 > (*bad_task_res + 1) as i64;
        let finish_time = now_time().unwrap_or_default();
        match task_res {
            Ok((_, git_tag_data, clone_id, run_res)) => {
                match run_res {
                    Err(err) => {
                        let status = DocGitCloneStatus::Fail as i8;
                        let change = lsys_core::model_option_set!(DocGitCloneModelRef, {
                            finish_time: finish_time,
                            status:status
                        });
                        if let Err(err) = Update::< DocGitCloneModel, _>::new(change)
                            .execute_by_where(
                                &WhereOption::Where(sql_format!(
                                    "id={} and status!={}",
                                    clone_id,
                                    DocGitCloneStatus::Delete as i8
                                )),
                                db,
                            )
                            .await
                        {
                            warn!("update clone bad status fail:{}", err);
                        }
                        let host_name = hostname::get()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let errmsg = match err {
                            CloneError::VerNotMatch(ver) => {
                                format!(
                                    "version not match {}!={} ",
                                    git_tag_data.build_version, ver
                                )
                            }
                            CloneError::Err(err) => err,
                        };
                        let vdata = model_option_set!(DocLogsModelRef, {
                            doc_clone_id:clone_id,
                            host:host_name,
                            doc_tag_id:git_tag_data.id,
                            message:errmsg,
                            add_time:finish_time,
                        });
                        if let Err(err) = Insert::<DocLogsModel, _>::new(vdata).execute(db).await {
                            warn!("add git clone log fail:{}", err);
                        }
                    }
                    //写doc_clone doc_build doc_logs 数据
                    Ok(_) => {
                        let status = DocGitCloneStatus::Cloned as i8;
                        let change = lsys_core::model_option_set!(DocGitCloneModelRef, {
                            finish_time: finish_time,
                            status:status
                        });
                        if let Err(err) = Update::< DocGitCloneModel, _>::new(change)
                            .execute_by_where(
                                &WhereOption::Where(sql_format!(
                                    "id={}  and status!={}",
                                    clone_id,
                                    DocGitCloneStatus::Delete as i8
                                )),
                                db,
                            )
                            .await
                        {
                            warn!("update clone succ status fail:{}", err);
                        }
                        let host_name = hostname::get()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let message = "clone finish".to_string();
                        let vdata = model_option_set!(DocLogsModelRef, {
                            doc_clone_id:clone_id,
                            host:host_name,
                            doc_tag_id:git_tag_data.id,
                            message:message,
                            add_time:finish_time,
                        });
                        if let Err(err) = Insert::<DocLogsModel, _>::new(vdata).execute(db).await {
                            warn!("add git clone succ log fail:{}", err);
                        }
                    }
                }
                *run_size += 1;
                task_ing.lock().await.retain(|x| x.1 != clone_id);
            }
            Err(err) => {
                //有任务PANIC了,非稳定版没法捕捉到任务ID,等TOKIO升级后在修改...
                error!("git task error:{:?}", err);
                if (*bad_task_res + 1) as i64 == all_finish.len() as i64 {
                    //失败任务数量等于已完成的任务数量,说明这些已完成任务都已经失败
                    for (tag_id, bad_clone_id) in all_finish {
                        task_ing.lock().await.retain(|x| x.1 != bad_clone_id);
                        *run_size += 1;
                        *bad_task_res -= 1;
                        let finish_time: u64 = now_time().unwrap_or_default();
                        let status = DocGitCloneStatus::Fail as i8;
                        let change = lsys_core::model_option_set!(DocGitCloneModelRef, {
                            finish_time: finish_time,
                            status:status
                        });
                        if let Err(err) = Update::< DocGitCloneModel, _>::new(change)
                            .execute_by_where(
                                &WhereOption::Where(sql_format!(
                                    "id={} and status={}",
                                    bad_clone_id,
                                    DocGitCloneStatus::Delete as i8
                                )),
                                db,
                            )
                            .await
                        {
                            warn!("update clone bad status fail:{}", err);
                        }
                        let errmsg = "task is panic".to_string();
                        let host_name = hostname::get()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let vdata = model_option_set!(DocLogsModelRef, {
                            doc_clone_id:bad_clone_id,
                            host:host_name,
                            doc_tag_id:tag_id,
                            message:errmsg,
                            add_time:finish_time,
                        });
                        if let Err(err) = Insert::<DocLogsModel, _>::new(vdata).execute(db).await {
                            warn!("add git clone log fail:{}", err);
                        }
                    }
                } else {
                    *bad_task_res += 1;
                }
            }
        }
        next
    }
    //进行GIT CLONE任务及 检查目录中文件是否存在
    fn run_task(
        git_data: &DocGitModel,
        git_tag_data: &DocGitTagModel,
        save_dir: &Path,
    ) -> CloneResult {
        match git_download(&git_data.url, save_dir, git_data.max_try, &git_tag_data.tag) {
            Ok(oid) => {
                if oid.to_string() != git_tag_data.build_version {
                    return Err(CloneError::VerNotMatch(oid.to_string()));
                }
                let rule = serde_json::from_str::<Option<Vec<String>>>(&git_tag_data.clear_rule)
                    .unwrap_or_default();
                git_clear(save_dir, &rule);
            }
            Err(err) => return Err(CloneError::Err(err)),
        };
        Ok(())
    }
    //检查任务ID是否正常,并执行任务
    //返回是否正常添加任务
    async fn add_task(
        db: &Pool<MySql>,
        save_dir: &str,
        task_set: &mut JoinSet<TaskResult>,
        task_ing: &Arc<Mutex<Vec<TaskIngData>>>, //进行中任务
        git_tag: DocGitTagModel,
        run_size: &mut usize,
    ) -> bool {
        let res = sqlx::query_as::<_, DocGitModel>(&sql_format!(
            "select * from {} where id={}",
            DocGitModel::table_name(),
            git_tag.doc_git_id
        ))
        .fetch_one(db)
        .await;

        let git_data = match res {
            Ok(data) => data,
            Err(err) => {
                warn!("{} add task fail:{}", git_tag.id, err);
                return false;
            }
        };
        let host_name = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let status = DocGitCloneStatus::Init as i8;
        let add_time = now_time().unwrap_or_default();
        let vdata = model_option_set!(DocGitCloneModelRef, {
            doc_tag_id:git_tag.id,
            host:host_name,
            start_time:add_time,
            finish_time:0,
            status:status,
        });

        let clone_id = match Insert::<DocGitCloneModel, _>::new(vdata).execute(db).await {
            Ok(row) => row.last_insert_id(),
            Err(err) => {
                warn!("add git clone log fail:{}", err);
                let add_time = now_time().unwrap_or_default();
                let message = format!("add clone fail:{}", err);
                let vdata = model_option_set!(DocLogsModelRef, {
                    doc_tag_id:git_tag.id,
                    host:host_name,
                    message:message,
                    add_time:add_time,
                });
                if let Err(err) = Insert::<DocLogsModel, _>::new(vdata).execute(db).await {
                    info!("add clone log fail:{}", err);
                }
                return false;
            }
        };

        // let config_dir = config!(app_core.config)
        //     .get_string("doc_git_dir")
        //     .unwrap_or_else(|_| env::temp_dir().to_string_lossy().to_string());

        let save_dir = match git_doc_path(save_dir, &clone_id, &None).await {
            Ok(set) => set,
            Err(err) => {
                warn!(
                    "{} doc save file dir :{}",
                    clone_id,
                    err.to_fluent_message().default_format()
                );
                return false;
            }
        };
        let tag_id = git_tag.id;
        *run_size -= 1; //添加任务前先减值
        let abort = task_set.spawn_blocking(move || {
            let res = Self::run_task(&git_data, &git_tag, save_dir.as_path());
            (git_data, git_tag, clone_id, res)
        });
        debug!("clone task end :{}", clone_id);
        task_ing.lock().await.push((tag_id, clone_id, abort));
        true
    }
    /// 获得发送中任务信息
    /// * `app_core` - 公共APP句柄,用于创建REDIS
    /// * `task_reader` - 任务读取实现
    /// * `task_executor` - 任务发送实现
    pub async fn dispatch(&self) {
        if self.task_size == 0 {
            error!("task can't is 0");
            return;
        }
        let db = self.db.clone();
        let max_size = self.task_size;
        // let app_core = self.app_core.clone();
        let save_dir = self.save_dir.clone();
        let task_ing = self.task_ing.clone(); //任务数据跟任务处理关联数组
        match self.rx.lock().await.take() {
            Some(mut rx) => {
                let (channel_sender, mut channel_receiver) =
                    mpsc::channel::<DocGitTagModel>(max_size);
                tokio::spawn(async move {
                    //连接REDIS
                    let mut run_size = max_size;
                    let mut task_empty;
                    let mut task_set = JoinSet::new(); //进行中任务,没法将任务数据在这关联,所以用 task_ing 关联
                    let mut bad_task_num = 0;
                    let exe_db = db.clone();
                    'task_main: loop {
                        debug!("start git doc task");
                        //从channel 获取任务,不阻塞
                        task_empty = match channel_receiver.try_recv() {
                            Ok(v) => {
                                //获取到任务,执行任务
                                if !Self::add_task(
                                    &exe_db,
                                    &save_dir,
                                    &mut task_set,
                                    &task_ing,
                                    v,
                                    &mut run_size,
                                )
                                .await
                                {
                                    continue;
                                }
                                false
                            }
                            Err(err) => {
                                if err != TryRecvError::Empty {
                                    error!("task channel error:{}", err);
                                    sleep(Duration::from_secs(1)).await;
                                    false
                                } else {
                                    //未获取到任务,下一步阻塞等待
                                    true
                                }
                            }
                        };
                        //未获取到任务,且还有闲置发送
                        if task_empty && run_size > 0 {
                            //异步阻塞等待任务
                            'recv: loop {
                                if task_set.is_empty() {
                                    Self::clear_delete_clone(&exe_db, &save_dir).await;
                                    //无进行中任务,只监听新增
                                    match channel_receiver.recv().await {
                                        Some(v) => {
                                            if !Self::add_task(
                                                &exe_db,
                                                &save_dir,
                                                &mut task_set,
                                                &task_ing,
                                                v,
                                                &mut run_size,
                                            )
                                            .await
                                            {
                                                continue 'recv;
                                            } else {
                                                break 'recv;
                                            }
                                        }
                                        None => {
                                            //异步阻塞未获取任务,可能发生错误,重新回到不阻塞 try_recv 去获取详细
                                            info!("channel no task ");
                                            continue 'task_main;
                                        }
                                    }
                                } else {
                                    //有进行中任务,监听任务完成跟新增
                                    tokio::select! {
                                        res = channel_receiver.recv() => {
                                            match res {
                                                Some(v) => {
                                                    if !Self::add_task(
                                                        &exe_db,
                                                        &save_dir ,
                                                        &mut task_set,
                                                        &task_ing,
                                                        v,
                                                        &mut run_size,
                                                    )
                                                    .await
                                                    {
                                                        continue 'recv;
                                                    }else{
                                                        break 'recv;
                                                    }
                                                }
                                                None => {
                                                    //异步阻塞未获取任务,可能发生错误,重新回到不阻塞 try_recv 去获取详细
                                                    info!("channel no task [select]");
                                                    continue 'task_main;
                                                }
                                            }
                                        }
                                        res = task_set.join_next()=> {
                                            if let Some(res)=res{
                                                Self::one_task_finish(&exe_db, &task_ing,&mut bad_task_num,res, &mut run_size).await;
                                                continue 'recv;
                                            }else{
                                                warn!("[git clone task] select task set is empty");//理论上,永远不会进入这里
                                            }
                                        },
                                    };
                                }
                            }
                        }
                        //任务处理已满,需等待进行中任务处理完在继续
                        if run_size == 0 && !task_set.is_empty() {
                            while let Some(res) = task_set.join_next().await {
                                let find_next_finish_task = Self::one_task_finish(
                                    &exe_db,
                                    &task_ing,
                                    &mut bad_task_num,
                                    res,
                                    &mut run_size,
                                )
                                .await;
                                if find_next_finish_task {
                                    continue;
                                }
                                break;
                                //退出任务完成检测,进入任务处理流程
                            }
                        }
                        debug!("next git doc task");
                    }
                });
                let _ = self.tx.try_send(()); //启动时检查未初始化的记录
                loop {
                    match rx.recv().await {
                        Some(()) => {
                            info!("git doc recv");
                            let mut start_id = 0;
                            'work: loop {
                                let ing_id = self
                                    .task_ing
                                    .lock()
                                    .await
                                    .iter()
                                    .filter(|e| e.0 > start_id)
                                    .map(|e| e.0)
                                    .collect::<Vec<_>>();
                                let hostname = hostname::get()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                let git_res = sqlx::query_as::<_, DocGitTagModel>(&sql_format!(
                                    "select * from {} where id >{} and
                                                id not in (select doc_tag_id from {} where 
                                                    host={} and status in ({},{})) {} 
                                                    order by id asc limit {} ",
                                    DocGitTagModel::table_name(),
                                    start_id,
                                    DocGitCloneModel::table_name(),
                                    hostname,
                                    //'已克隆,克隆失败'
                                    DocGitCloneStatus::Cloned,
                                    DocGitCloneStatus::Fail,
                                    if !ing_id.is_empty() {
                                        SqlExpr(sql_format!("and id not in  ({}) ", ing_id))
                                    } else {
                                        SqlExpr("".to_string())
                                    },
                                    max_size
                                ))
                                .fetch_all(&self.db)
                                .await;

                                let git_data = match git_res {
                                    Ok(res) => res,
                                    Err(err) => {
                                        error!("select data error :{}", err);
                                        sleep(Duration::from_secs(30)).await;
                                        continue 'work;
                                    }
                                };
                                if git_data.is_empty() {
                                    debug!("finish git doc notify");
                                    break 'work;
                                }
                                debug!("send task total:{}", git_data.len());
                                if let Some(last) = git_data.last() {
                                    start_id = last.id
                                }
                                for tmp in git_data {
                                    let tag_id = tmp.id;
                                    debug!("add git task [{}]", tag_id);
                                    if let Err(e) = channel_sender.send(tmp).await {
                                        error!("add git task fail[{}]:{}", tag_id, e);
                                    }
                                }
                            }
                        }
                        None => {
                            info!("git task is out");
                            break;
                        }
                    }
                }
            }
            None => {
                info!("double run dispatch")
            }
        }
    }
    //清理已被标记删除，但实际未被删掉的文件
    async fn clear_delete_clone(db: &Pool<MySql>, save_dir: &str) {
        let mut start_id = 0;
        loop {
            let mut clear_time = now_time().unwrap_or_default();
            if clear_time > 3600 {
                clear_time -= 3600;
            }
            let sql = sql_format!(
                " select id from {} where  id >{} and status={} and finish_time>={} order by id asc limit {} 
                ",
                DocGitCloneModel::table_name(),
                start_id,
                DocGitCloneStatus::Delete,
                clear_time,
                100,
            );
            let git_res = sqlx::query_scalar::<_, u64>(sql.as_str())
                .fetch_all(db)
                .await;
            match git_res {
                Ok(data) => {
                    if data.is_empty() {
                        break;
                    }
                    for tmp in data {
                        Self::delete_clone_dir(save_dir, &tmp).await;
                        start_id = tmp;
                    }
                }
                Err(sqlx::Error::RowNotFound) => {
                    break;
                }
                Err(err) => {
                    info!("clear delete data fail:{}", err);
                    break;
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
enum DocAction {
    Del { clone_id: u64 },
    Clone,
}

pub const REMOTE_NOTIFY_TYPE_DOC_TASK: u8 = 102;

impl GitTask {
    pub async fn remote_clone(&self) {
        if let Err(err) = self
            .remote_notify
            .call(
                REMOTE_NOTIFY_TYPE_DOC_TASK,
                DocAction::Clone,
                None,
                LocalExecType::RemoteExec,
                None,
            )
            .await
        {
            warn!(
                "notify clone fail:{}",
                err.to_fluent_message().default_format()
            )
        }
    }
    pub async fn remote_delete_clone(
        &self,
        clone_id: u64,
        host: &str,
        timeout: u64,
    ) -> GitDocResult<()> {
        if let Err(err) = self
            .remote_notify
            .call(
                REMOTE_NOTIFY_TYPE_DOC_TASK,
                DocAction::Del { clone_id },
                Some(host),
                LocalExecType::LocalExec,
                Some(ReplyWait {
                    max_node: 1,
                    timeout,
                }),
            )
            .await
        {
            warn!(
                "remote delete clone fail:{}",
                err.to_fluent_message().default_format()
            );
            return Err(GitDocError::Remote(fluent_message!(
                "doc-notify-call-fail",
                err.to_fluent_message()
            )));
        }
        Ok(())
    }
    pub async fn delete_clone_dir(save_dir: &str, clone_id: &u64) {
        // let config_dir = config!(app_core.config)
        //     .get_string("doc_git_dir")
        //     .unwrap_or_else(|_| env::temp_dir().to_string_lossy().to_string());
        let save_path = match git_doc_path(save_dir, clone_id, &None).await {
            Ok(set) => set,
            Err(err) => {
                warn!(
                    "{} doc save file dir :{}",
                    clone_id,
                    err.to_fluent_message().default_format()
                );
                return;
            }
        };
        if fs::metadata(&save_path).await.is_err() {
            return;
        }
        if let Err(err) = remove_dir_all(&save_path).await {
            warn!(
                "clear data {}({}) fail:{}",
                clone_id,
                save_path.to_string_lossy().to_string(),
                err
            );
        };
    }
    pub async fn delete_clone(&self, clone_id: &u64) -> GitDocResult<()> {
        let finish_time = now_time().unwrap_or_default();
        let status = DocGitCloneStatus::Delete as i8;
        let change = lsys_core::model_option_set!(DocGitCloneModelRef, {
            finish_time: finish_time,
            status:status
        });
        match Update::< DocGitCloneModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("id={}", clone_id,)),
                &self.db,
            )
            .await
        {
            Ok(_) => {
                Self::delete_clone_dir(&self.save_dir, clone_id).await;
            }
            Err(err) => {
                warn!("update clone bad status fail:{}", err);
            }
        }
        Ok(())
    }
}

use async_trait::async_trait;
/// 订阅远程通知清理本地缓存
pub struct GitRemoteTask {
    task: Arc<GitTask>,
}
impl GitRemoteTask {
    pub fn new(task: Arc<GitTask>) -> Self {
        GitRemoteTask { task }
    }
}

#[async_trait]
impl RemoteTask for GitRemoteTask {
    fn msg_type(&self) -> u8 {
        REMOTE_NOTIFY_TYPE_DOC_TASK
    }
    async fn run(&self, msg: MsgSendBody) -> Result<Option<Value>, String> {
        let action = serde_json::from_value::<DocAction>(msg.data).map_err(|e| e.to_string())?;
        match action {
            DocAction::Del { clone_id } => match self.task.delete_clone(&clone_id).await {
                Ok(_) => {
                    if let Err(err) = self.task.notify() {
                        warn!(
                            "delete clone after notify clone fail:{}",
                            err.to_fluent_message().default_format()
                        );
                    }
                }
                Err(err) => {
                    let err_str = err.to_fluent_message().default_format();
                    warn!("delete clone fail:{}", err_str);
                    return Err(err_str);
                }
            },
            DocAction::Clone => {
                if let Err(err) = self.task.notify() {
                    warn!(
                        "notify clone fail:{}",
                        err.to_fluent_message().default_format()
                    );
                }
            }
        }
        Ok(None)
    }
}
