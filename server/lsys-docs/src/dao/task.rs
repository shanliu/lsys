use git2::Repository;

use lsys_core::now_time;
use sqlx::{MySql, Pool};
use sqlx_model::{
    model_option_set, sql_format, Insert, ModelTableName, Select, SqlExpr, Update, WhereOption,
};
use tokio::{
    sync::{
        mpsc::{self, error::TryRecvError, Receiver, Sender},
        Mutex,
    },
    task::{AbortHandle, JoinError, JoinSet},
    time::sleep,
};
use tracing::{debug, error, info, warn};

use crate::model::{
    DocBuildModel, DocBuildModelRef, DocBuildStatus, DocCloneModel, DocCloneModelRef,
    DocCloneStatus, DocGitModel, DocGitModelRef, DocGitStatus, DocLogsModel, DocLogsModelRef,
    DocMenuModel, DocMenuModelRef, DocMenuStatus,
};
use sqlx_model::SqlQuote;
use std::{format, path::Path, println, time::Duration};

use super::GitDocResult;

// 发送任务抽象实现
pub struct GitTask {
    db: Pool<MySql>,
    tx: Sender<bool>,
    rx: Mutex<Option<Receiver<bool>>>,
    task_size: usize,
}

type BuildResult = Result<
    Vec<(
        DocMenuModel,
        DocBuildStatus, //构建结果,失败,部分成功,完全成功
        String,         //成功构建的菜单内容
        Vec<String>,    //根据菜单内容检测GIT文件失败消息
    )>,
    String,
>;

type TaskResult = (
    String, //hostname
    DocGitModel,
    BuildResult,
);

impl GitTask {
    pub fn new(db: Pool<MySql>) -> Self {
        let (tx, rx) = mpsc::channel::<bool>(1);
        Self {
            tx,
            db,
            rx: Mutex::new(Some(rx)),
            task_size: 10,
        }
    }
    /// 通知发送模块进行发送操作
    pub fn notify(&self) -> GitDocResult<()> {
        match self.tx.try_send(false) {
            Ok(_) => Ok(()),
            Err(err) => match err {
                mpsc::error::TrySendError::Full(_) => Ok(()),
                mpsc::error::TrySendError::Closed(_) => {
                    warn!("git task is close");
                    Err(super::GitDocError::System(err.to_string()))
                }
            },
        }
    }

    //清理已完成的任务
    //返回是否存在已完成任务
    async fn clean_task_ing(
        db: &Pool<MySql>,
        task_ing: Vec<(DocGitModel, String, AbortHandle)>,
        run_size: &mut usize,
    ) -> (Vec<(DocGitModel, String, AbortHandle)>, bool) {
        let mut finsih_pk = vec![];
        let on_task_ing = {
            let mut out = vec![];
            for tmp in task_ing {
                if tmp.2.is_finished() {
                    finsih_pk.push(tmp);
                } else {
                    out.push(tmp);
                }
            }
            out
        };
        //未查找到已完成任务,可能上一次已处理完.重新进入等待
        if finsih_pk.is_empty() {
            return (on_task_ing, false);
        }
        //存在处理完任务
        for (docgit, host, _) in finsih_pk {
            *run_size += 1;
            let status = DocCloneStatus::Ready as i8;
            let clone_time: u64 = now_time().unwrap_or_default();
            let vdata = model_option_set!(DocCloneModelRef,{
                doc_git_id:docgit.id,
                host:host,
                build_version:docgit.build_version,
                clone_time:clone_time,
                finish_time:0,
                clone_try:1,
                status:status,
            });
            let change = model_option_set!(DocCloneModelRef,{
                clone_time:clone_time,
                clone_try:2,
            });
            if let Err(err) = Insert::<sqlx::MySql, DocCloneModel, _>::new(vdata)
                .execute_update(&Update::<MySql, DocCloneModel, _>::new(change), db)
                .await
            {
                warn!("update clone data fail:{}", err);
            }
        }
        (on_task_ing, true)
    }
    //返回CLONE任务时更新相关记录及日志处理
    async fn finish_task(
        db: &Pool<MySql>,
        res: Result<
            TaskResult,
            JoinError, //panic 错误
        >,
    ) {
        match res {
            Ok((host, git_res, run_res)) => {
                match run_res {
                    //写doc_clone doc_build doc_logs 数据
                    Ok(build_res) => {
                        let status = DocCloneStatus::Cloned as i8;
                        let add_time = now_time().unwrap_or_default();
                        let vdata = model_option_set!(DocCloneModelRef,{
                            doc_git_id:git_res.id,
                            host:host,
                            build_version:git_res.build_version,
                            clone_time:add_time,
                            finish_time:add_time,
                            clone_try:0,
                            status:status,
                        });
                        let change = model_option_set!(DocCloneModelRef,{
                            status:status,
                        });

                        let mut trdb = match db.begin().await {
                            Ok(t) => t,
                            Err(err) => {
                                warn!("finish clone,but db error:{}", err);
                                return;
                            }
                        };

                        if let Err(err) = Insert::<sqlx::MySql, DocCloneModel, _>::new(vdata)
                            .execute_update(
                                &Update::<MySql, DocCloneModel, _>::new(change),
                                &mut trdb,
                            )
                            .await
                        {
                            let _ = trdb.rollback().await;
                            warn!("update clone data fail:{}", err);
                            return;
                        }

                        if git_res.finish_time == 0 {
                            let change = sqlx_model::model_option_set!(DocGitModelRef, {
                                finish_time: add_time
                            });
                            if let Err(err) = Update::<MySql, DocGitModel, _>::new(change)
                                .execute_by_pk(&git_res, &mut trdb)
                                .await
                            {
                                let _ = trdb.rollback().await;
                                warn!("update git data fail:{}", err);
                                return;
                            }
                        }

                        for (menu_res, status, build_data, _) in build_res.iter() {
                            let status = *status as i8;
                            let add_time = now_time().unwrap_or_default();
                            let vdata = model_option_set!(DocBuildModelRef,{
                                doc_git_id:git_res.id,
                                doc_menu_id:menu_res.id,
                                build_data:build_data,
                                host:host,
                                build_version:git_res.build_version,
                                finish_time:add_time,
                                status:status,
                            });
                            let change = model_option_set!(DocBuildModelRef,{
                                status:status,
                            });
                            if let Err(err) = Insert::<sqlx::MySql, DocBuildModel, _>::new(vdata)
                                .execute_update(
                                    &Update::<MySql, DocBuildModel, _>::new(change),
                                    &mut trdb,
                                )
                                .await
                            {
                                let _ = trdb.rollback().await;
                                warn!("set succ build fail:{}", err);
                                return;
                            }

                            if menu_res.finish_time == 0 {
                                let change = sqlx_model::model_option_set!(DocMenuModelRef, {
                                    finish_time: add_time
                                });
                                if let Err(err) = Update::<MySql, DocMenuModel, _>::new(change)
                                    .execute_by_pk(menu_res, db)
                                    .await
                                {
                                    let _ = trdb.rollback().await;
                                    warn!("update menu data fail:{}", err);
                                    return;
                                }
                            }
                        }
                        if let Err(err) = trdb.commit().await {
                            warn!("update data fail:{}", err);
                            return;
                        };
                        for (menu_res, _, _, err_msg) in build_res {
                            for tmp_err in err_msg {
                                let vdata = model_option_set!(DocLogsModelRef, {
                                    doc_git_id:git_res.id,
                                    host:host,
                                    doc_menu_id:menu_res.id,
                                    build_version:git_res.build_version,
                                    message:tmp_err,
                                    add_time:add_time,
                                });
                                if let Err(err) = Insert::<sqlx::MySql, DocLogsModel, _>::new(vdata)
                                    .execute(db)
                                    .await
                                {
                                    warn!("add git build log fail:{}", err);
                                }
                            }
                        }
                        //TODO  判断 全部已经完成 状态OK或大于n次 且 已经存在失败 且 存在成功
                        //
                    }
                    Err(err) => {
                        let add_time = now_time().unwrap_or_default();

                        let vdata = model_option_set!(DocLogsModelRef, {
                            doc_git_id:git_res.id,
                            host:host,
                            build_version:git_res.build_version,
                            message:err,
                            add_time:add_time,
                        });
                        if let Err(err) = Insert::<sqlx::MySql, DocLogsModel, _>::new(vdata)
                            .execute(db)
                            .await
                        {
                            warn!("add git clone log fail:{}", err);
                        }
                    }
                }
            }
            Err(err) => {
                //有任务PANIC了,非稳定版没法捕捉到任务ID,等TOKIO升级后在修改...
                error!("git task error:{:?}", err);
            }
        }
    }
    //进行GIT CLONE任务及 检查目录中文件是否存在
    fn run_task(
        _host_name: &str,
        git_res: &DocGitModel,
        menu_res: &Vec<DocMenuModel>,
        _clone_res: &Option<DocCloneModel>,
        //build_res: Vec<DocBuildModel>,
    ) -> BuildResult {
        //TODO 这里继续...
        let _repo = if Path::new(&git_res.url).is_dir() {
            match Repository::open("./git") {
                Ok(repo) => repo,
                Err(e) => return Err(e.to_string()),
            }
            //  repo.remote_add_fetch(name, spec)
        } else {
            match Repository::clone(&git_res.url, "./git") {
                Ok(repo) => repo,
                Err(e) => return Err(e.to_string()),
            }
        };
        for tmp in menu_res {
            println!("{:?}", tmp)
        }
        //对比当前的版本跟 _clone_res 差异，异常时重新clone
        //处理menu，最后返回所有处理结果
        //组装结果返回
        //     repo.branch("ddd", target, force);
        Ok(vec![])
    }
    //检查任务ID是否正常,并执行任务
    //返回是否正常添加任务
    async fn add_task(
        db: &Pool<MySql>,
        task_set: &mut JoinSet<TaskResult>,
        task_ing: &mut Vec<(DocGitModel, String, AbortHandle)>,
        v: u32,
        run_size: &mut usize,
    ) -> bool {
        if let Some(in_id) = task_ing.iter().find(|e| e.0.id == v) {
            info!("task {} is runing", in_id.0.id);
            return false;
        }
        let host_name = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let res = Select::type_new::<DocGitModel>()
            .fetch_one_by_scalar_pk::<DocGitModel, _, _>(v, db)
            .await;
        let git_data = match res {
            Ok(data) => data,
            Err(err) => {
                warn!("{} add task fail:{}", v, err);
                return false;
            }
        };
        //TODO  检测GIT 路径.及必要数据
        //目录数据
        let res = Select::type_new::<DocMenuModel>()
            .fetch_all_by_where_call::<DocMenuModel, _, _>(
                "doc_git_id =? and build_version=? and status=?",
                |tmp, _| {
                    tmp.bind(git_data.id)
                        .bind(git_data.build_version.clone())
                        .bind(DocMenuStatus::Enable as i8)
                },
                db,
            )
            .await;
        let menu_res = match res {
            Ok(data) => data,
            Err(err) => {
                warn!("{} select clone fail:{}", git_data.id, err);
                return false;
            }
        };
        if menu_res.is_empty() {
            let add_time = now_time().unwrap_or_default();
            let message = "menu is empty,skip clone".to_string();
            let vdata = model_option_set!(DocLogsModelRef, {
                doc_git_id:git_data.id,
                host:host_name,
                build_version:git_data.build_version,
                message:message,
                add_time:add_time,
            });
            if let Err(err) = Insert::<sqlx::MySql, DocLogsModel, _>::new(vdata)
                .execute(db)
                .await
            {
                warn!("add git clone log fail:{}", err);
            }
            return false;
        }
        let build_sql = sql_format!(
            "doc_git_id ={} and host={} and build_version={} and doc_menu_id in ({}) ",
            git_data.id,
            host_name,
            git_data.build_version,
            menu_res.iter().map(|e| e.id).collect::<Vec<_>>()
        );
        let res = Select::type_new::<DocBuildModel>()
            .fetch_all_by_where::<DocBuildModel, _>(&WhereOption::Where(build_sql), db)
            .await;
        match res {
            Ok(data) => {
                let mut all_finish = true;
                for tmp in menu_res.iter() {
                    if let Some(dat) = data.iter().find(|e| e.doc_menu_id == tmp.id) {
                        if !DocBuildStatus::Finish.eq(dat.status)
                            && !DocBuildStatus::Succ.eq(dat.status)
                        {
                            all_finish = false;
                            break;
                        }
                    } else {
                        all_finish = false;
                        break;
                    }
                }
                if all_finish {
                    //is finish
                    info!("{} double run build", git_data.id);
                    return false;
                }
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                warn!("{} select clone fail:{}", git_data.id, err);
                return false;
            }
        };

        let res = Select::type_new::<DocCloneModel>()
            .fetch_one_by_where_call::<DocCloneModel, _, _>(
                "doc_git_id =? and host=? ",
                |tmp, _| tmp.bind(git_data.id).bind(git_data.build_version.clone()),
                db,
            )
            .await;
        let clone_res = match res {
            Ok(data) => {
                if DocCloneStatus::Cloned.eq(data.status) {
                    info!("git[{}] is clone ", git_data.id);
                }
                Some(data)
            }
            Err(sqlx::Error::RowNotFound) => None,
            Err(err) => {
                warn!("{} select clone fail:{}", git_data.id, err);
                None
            }
        };
        let task_git_data = git_data.to_owned();
        let task_host_name = host_name.clone();
        *run_size -= 1; //添加任务前先减值
        let abort = task_set.spawn_blocking(move || {
            let res = Self::run_task(&task_host_name, &task_git_data, &menu_res, &clone_res);
            (task_host_name, task_git_data, res)
        });
        debug!("add async task end :{}", git_data.id);
        task_ing.push((git_data, host_name, abort));
        true
    }
    /// 获得发送中任务信息
    /// * `app_core` - 公共APP句柄,用于创建REDIS
    /// * `task_reader` - 任务读取实现
    /// * `task_executioner` - 任务发送实现
    pub async fn dispatch(&self) {
        if self.task_size == 0 {
            error!("task can't is 0");
            return;
        }

        let db = self.db.clone();
        let max_size = self.task_size;
        match self.rx.lock().await.take() {
            Some(mut rx) => {
                let (channel_sender, mut channel_receiver) = mpsc::channel::<u32>(max_size);
                tokio::spawn(async move {
                    //连接REDIS
                    let mut run_size = max_size;
                    let mut task_empty;
                    let mut task_set = JoinSet::new(); //进行中任务,没法将任务数据在这关联,所以用 task_ing 关联
                    let mut task_ing = vec![]; //任务数据跟任务处理关联数组
                    let exe_db = db.clone();
                    'task_main: loop {
                        debug!("start git doc task");
                        //从channel 获取任务,不阻塞
                        task_empty = match channel_receiver.try_recv() {
                            Ok(v) => {
                                //获取到任务,执行任务
                                if !Self::add_task(
                                    &exe_db,
                                    &mut task_set,
                                    &mut task_ing,
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
                            //查找已完成任务列表
                            task_ing = Self::clean_task_ing(&exe_db, task_ing, &mut run_size)
                                .await
                                .0;
                            //异步阻塞等待任务
                            'recv: loop {
                                if task_set.is_empty() {
                                    //无进行中任务,只监听新增
                                    match channel_receiver.recv().await {
                                        Some(v) => {
                                            if !Self::add_task(
                                                &exe_db,
                                                &mut task_set,
                                                &mut task_ing,
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
                                                        &mut task_set,
                                                        &mut task_ing,
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
                                                Self::finish_task(&exe_db,res).await;
                                                task_ing=Self::clean_task_ing(&exe_db, task_ing, &mut run_size).await.0;
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
                                Self::finish_task(&exe_db, res).await;
                                let find_finish;
                                (task_ing, find_finish) =
                                    Self::clean_task_ing(&exe_db, task_ing, &mut run_size).await;
                                if !find_finish {
                                    continue;
                                }
                                break;
                                //退出任务完成检测,进入任务处理流程
                            }
                        }
                    }
                });
                let _ = self.tx.try_send(true); //启动时检查未初始化的记录
                loop {
                    match rx.recv().await {
                        Some(all) => {
                            let mut id = 0;
                            loop {
                                let hostname = hostname::get()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                let sql = sql_format!(
                                    "select id  from {} where id not in (
                                    select doc_git_id from {} where host={} and status!={} 
                                ) and status={} and id>{} {} order by id asc limit {}",
                                    DocGitModel::table_name(),
                                    DocCloneModel::table_name(),
                                    hostname,
                                    DocCloneStatus::Cloned,
                                    DocGitStatus::Enable,
                                    id,
                                    if !all {
                                        SqlExpr(" and finish_time>0 ")
                                    } else {
                                        SqlExpr("")
                                    },
                                    max_size
                                );
                                let git_res =
                                    sqlx::query_scalar::<_, u32>(&sql).fetch_all(&self.db).await;
                                let git_data = match git_res {
                                    Ok(res) => res,
                                    Err(sqlx::Error::RowNotFound) => {
                                        break;
                                    }
                                    Err(err) => {
                                        error!("select data error :{}", err);
                                        sleep(Duration::from_secs(30)).await;
                                        continue;
                                    }
                                };
                                if let Some(last) = git_data.last() {
                                    id = last.to_owned()
                                }
                                for tmp in git_data {
                                    if let Err(e) = channel_sender.send(tmp).await {
                                        error!("add git task fail[{}]:{}", tmp, e);
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
}
