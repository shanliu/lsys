use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Display,
    hash::Hash,
    sync::atomic::AtomicU32,
};

use async_trait::async_trait;

use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use lsys_core::{IntoFluentMessage, TaskAcquisition, TaskItem};
use lsys_setting::model::SettingModel;
use redis::{FromRedisValue, ToRedisArgs};

use tracing::warn;

use crate::model::SenderTplConfigModel;

use super::{SenderExecError, SenderTaskResult, SenderTaskResultItem, SenderTplConfig};

//在发送任务封装下
//增加多个发送适配支持

pub(crate) fn index_get(index_store: &AtomicU32, len: usize) -> usize {
    if len > 1 {
        let mut now = index_store.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as usize;
        if now + 1 >= len {
            index_store.store(0, std::sync::atomic::Ordering::Relaxed);
            now = 0
        }
        now
    } else {
        0
    }
}

pub trait SenderTaskItem<I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display>:
    TaskItem<I>
{
    fn app_id(&self) -> u64;
    fn tpl_id(&self) -> String;
}

pub trait SenderTaskData {
    fn to_pks(&self) -> Vec<u64>;
}

#[async_trait]
pub trait SenderTaskAcquisition<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display,
    G: SenderTaskItem<I>,
    D: SenderTaskData,
>: TaskAcquisition<I, G>
{
    async fn read_send_record(
        &self,
        record: &G,
        sending_data: &[u64],
        limit: u16,
    ) -> Result<D, String>;
    async fn task_send_success(
        &self,
        setting: &SettingModel,
        task: &G,
        record: &D,
        res_items: &[SenderTaskResultItem],
    );
    async fn task_record_send_fail(
        &self,
        setting: &SettingModel,
        task: &G,
        record: &D,
        error: &SenderExecError,
    );
    async fn task_send_fail(
        &self,
        task: &G,
        in_task_id: &[u64],
        error: &SenderExecError,
        setting: Option<&SettingModel>,
    );
}

#[async_trait]
pub trait SenderTaskExecutor<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display,
    G: SenderTaskItem<I>,
    D: SenderTaskData,
>: Sync + Send + 'static
{
    //发送适配器的标识
    fn setting_key(&self) -> String;
    //该适配器最大一次课发送的短信数量
    async fn limit(
        &self,
        setting: &SettingModel, //短信发送配置
    ) -> u16;
    //执行发送的实现
    async fn exec(
        &self,
        val: &G,                           //短信内容
        data: &D,                          //短信接收方，多个
        tpl_config: &SenderTplConfigModel, //短信模版
        setting: &SettingModel,            //短信发送配置
    ) -> SenderTaskResult;
}

pub(crate) type SenderTaskExecutorBox<I, G, D> = (Box<dyn SenderTaskExecutor<I, G, D>>, AtomicU32);

// 执行批量发送
pub(crate) async fn group_exec<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display + 'static,
    G: SenderTaskItem<I> + TaskItem<I> + std::marker::Sync + 'static,
    D: SenderTaskData + std::marker::Send + std::marker::Sync + 'static,
    R: SenderTaskAcquisition<I, G, D> + std::marker::Sync,
>(
    acquisition: &R,                          //任务读取器
    val: &G,                                  //任务记录
    index_store: &AtomicU32,                  //发送计数
    tpl_config: &SenderTplConfig,             //发送配置
    inner: &[SenderTaskExecutorBox<I, G, D>], //可用发送适配器数组
) -> Result<(), String> {
    if inner.is_empty() {
        let msg = "can't find any sender apapter".to_string();
        acquisition
            .task_send_fail(val, &[], &SenderExecError::Finish(msg.clone()), None)
            .await;
        return Err(msg);
    }
    let len = inner.len();
    let start = index_get(index_store, len);
    //获取任务消息可用的发送模板.
    let config = match tpl_config
        .list_config(None, None, Some(val.app_id()), Some(&val.tpl_id()), None)
        .await
        .map_err(|e| e.to_fluent_message().default_format())
    {
        Ok(t) => t,
        Err(err) => {
            acquisition
                .task_send_fail(val, &[], &SenderExecError::Next(err.clone()), None)
                .await;
            return Err(err);
        }
    };

    //根据历史发送整理发送模板数据
    type SendItem<'a, I, G, D> = Vec<(
        &'a Box<dyn SenderTaskExecutor<I, G, D>>,
        &'a SenderTplConfigModel,
        &'a SettingModel,
    )>;
    let mut send_group: HashMap<String, SendItem<I, G, D>> = HashMap::new();
    for index in 0..len {
        if let Some((exe_er, exe_index)) = inner.get((start + index) % len) {
            let exec_config = config
                .iter()
                .flat_map(|(tpl_config, tpl_index)| match &tpl_index {
                    Some(setting) => {
                        if exe_er.setting_key() != setting.setting_key {
                            None
                        } else {
                            Some((tpl_config, setting))
                        }
                    }
                    None => {
                        warn!("config id {} load apatar config fail", tpl_config.id);
                        None
                    }
                })
                .collect::<Vec<_>>();
            if exec_config.is_empty() {
                continue;
            }
            let exec_len = exec_config.len();
            let exe_start = index_get(exe_index, exec_len);
            for exec_index in 0..exec_len {
                if let Some((tpl_config, setting)) =
                    exec_config.get((exec_index + exe_start) % exec_len)
                {
                    let exe_type = exe_er.setting_key();
                    match send_group.entry(exe_type) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().push((
                                exe_er,
                                tpl_config.to_owned(),
                                setting.to_owned(),
                            ));
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(vec![(exe_er, tpl_config.to_owned(), setting.to_owned())]);
                        }
                    };
                }
            }
        }
    }

    if send_group.is_empty() {
        let msg = format!("can't find any tpl config on tpl :{}", val.tpl_id());
        acquisition
            .task_send_fail(val, &[], &SenderExecError::Next(msg.clone()), None)
            .await;
        return Err(msg);
    }

    let mut futures = FuturesUnordered::new();
    let mut send_ids = HashMap::<u64, (&str, i16, Vec<u64>)>::new();
    let mut all_read = false; //是否已经读取指定任务的所有发送记录
    for exec_group in send_group.values() {
        for (exec_er, tpl_config, setting) in exec_group.iter() {
            if all_read {
                send_ids.insert(tpl_config.id, (&setting.setting_key, 0, vec![]));
                continue;
            }
            let limit = exec_er.limit(setting).await;
            let in_ids = send_ids
                .iter()
                .flat_map(|e| e.1 .2.to_owned())
                .collect::<Vec<u64>>();
            let res = match acquisition.read_send_record(val, &in_ids, limit).await {
                Ok(r) => r,
                Err(err) => {
                    let msg = format!("read send record fail :{}", err);
                    acquisition
                        .task_send_fail(
                            val,
                            &[],
                            &SenderExecError::Next(msg.clone()),
                            Some(setting),
                        )
                        .await;
                    return Err(msg);
                }
            };

            let pks = res.to_pks();
            if pks.is_empty() {
                send_ids.insert(tpl_config.id, (&setting.setting_key, 0, vec![]));
                all_read = true;
            } else {
                send_ids.insert(tpl_config.id, (&setting.setting_key, 1, pks));
                futures.push(
                    async move {
                        let exec_res = exec_er.exec(val, &res, tpl_config, setting).await;
                        let is_ok = exec_res.is_ok();
                        match exec_res {
                            Ok(res_items) => {
                                acquisition
                                    .task_send_success(setting, val, &res, &res_items)
                                    .await;
                            }
                            Err(err) => {
                                acquisition
                                    .task_record_send_fail(setting, val, &res, &err)
                                    .await;
                            }
                        }
                        (is_ok, setting.setting_key.clone(), tpl_config.id)
                    }
                    .boxed(),
                );
            }
        }
    }

    loop {
        // select! 宏可以等待多个操作中的任何一个完成
        tokio::select! {
            // 当有future完成时
            Some((is_ok,setting_key,tpl_config_id)) = futures.next() => {
                let mut self_tmp=None;
                let mut tar_tmp=None;
                if let Some(tmp)=send_ids.get_mut(&tpl_config_id){
                    tmp.2=vec![];
                    if !is_ok{
                        tmp.1 = -1;
                    }
                    let now_num=tmp.1;
                    //查找同类型,且非当前执行的配置
                    for exec_group in send_group.values() {
                        let mut end_loop=false;
                        let mut min_num=-1;
                        for (exec_er, tpl_config, setting) in exec_group.iter() {
                            if setting.setting_key==setting_key{
                                if tpl_config_id!=tpl_config.id{
                                    if let Some(r_tmp)=send_ids.get(&setting.id){
                                        if r_tmp.1>=0 && (min_num<0 ||r_tmp.1<=min_num) {
                                            min_num=r_tmp.1;
                                            tar_tmp=Some((exec_er, tpl_config, setting));
                                        }
                                    }
                                }else if now_num>=0{
                                    self_tmp=Some((exec_er, tpl_config, setting))
                                }
                                end_loop=true;
                            }
                        }
                        if end_loop {break;}
                    }
                }
                if let Some((exec_er, tpl_config, setting)) = tar_tmp.or(self_tmp){
                    let in_ids = send_ids
                        .iter()
                        .flat_map(|e| e.1 .2.to_owned())
                        .collect::<Vec<u64>>();
                    if let Some(tmp)=send_ids.get_mut(&tpl_config.id){
                        let limit = exec_er.limit(setting).await;
                        match acquisition.read_send_record(val, &in_ids, limit).await {
                            Ok(res) => {
                                let pks=res.to_pks();
                                if pks.is_empty() {
                                    tmp.2=vec![];
                                } else {
                                    tmp.2=pks;
                                    tmp.1 += 1;
                                    futures.push(
                                        async move {
                                            let exec_res = exec_er.exec(val, &res, tpl_config, setting).await;
                                            let is_ok = exec_res.is_ok();
                                            match exec_res {
                                                Ok(res_items) => {
                                                    acquisition
                                                        .task_send_success(setting, val, &res, &res_items)
                                                        .await;
                                                }
                                                Err(err) => {
                                                    acquisition
                                                        .task_record_send_fail(setting, val, &res, &err)
                                                        .await;
                                                }
                                            }
                                            (is_ok, setting.setting_key.clone(),tpl_config.id)
                                        }
                                        .boxed(),
                                    );
                                }
                            },
                            Err(err) => {
                                let msg = format!("read send record fail [in task] :{}", err);
                                acquisition
                                    .task_send_fail(
                                        val,
                                        &in_ids,
                                        &SenderExecError::Next(msg.clone()),
                                        Some(setting),
                                    )
                                    .await;
                            }
                        }
                    }
                }
            }
            // 如果没有其他的操作，就结束循环
            else => {
                break
            },
        }
    }
    Ok(())
}
