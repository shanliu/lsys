use std::{
    fmt::{Display, Formatter},
    hash::Hash,
    sync::atomic::AtomicU32,
};

use async_trait::async_trait;

use lsys_setting::model::SettingModel;
use redis::{FromRedisValue, ToRedisArgs};
use tracing::{info, warn};

use crate::model::SenderTplConfigModel;

use super::{SenderTplConfig, TaskItem};

//在发送任务封装下
//增加多个发送适配支持

#[derive(Debug)]
pub enum SenderExecError {
    Finish(String),
    Next(String),
}

impl Display for SenderExecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Finish(e) => write!(f, "{:?}", e),
            Self::Next(e) => write!(f, "{:?}", e),
        }
    }
}

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

#[async_trait]
pub trait SenderTaskExecutor<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display,
    G: SenderTaskItem<I>,
>: Sync + Send + 'static
{
    async fn exec(
        &self,
        val: &G,
        tpl_config: &SenderTplConfigModel,
        setting: &SettingModel,
    ) -> Result<String, SenderExecError>;
    fn setting_key(&self) -> String;
}

pub(crate) type SenderTaskExecutorBox<I, G> = (Box<dyn SenderTaskExecutor<I, G>>, AtomicU32);

pub(crate) async fn group_exec<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display + 'static,
    G: SenderTaskItem<I> + 'static,
>(
    val: &G,
    index_store: &AtomicU32,
    tpl_config: &SenderTplConfig,
    inner: &[SenderTaskExecutorBox<I, G>],
) -> Result<(String, SenderTplConfigModel, SettingModel), String> {
    if !inner.is_empty() {
        let len = inner.len();
        let start = index_get(index_store, len);
        let config = tpl_config
            .list_config(
                &None,
                &None,
                &Some(val.app_id()),
                &Some(val.tpl_id()),
                &None,
            )
            .await
            .map_err(|e| e.to_string())?;
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
                        match exe_er.exec(val, tpl_config, setting).await {
                            Ok(send_note) => {
                                return Ok((
                                    send_note,
                                    tpl_config.to_owned().clone(),
                                    setting.to_owned().clone(),
                                ));
                            }
                            Err(err) => match err {
                                SenderExecError::Finish(err) => {
                                    return Err(err);
                                }
                                SenderExecError::Next(err) => {
                                    info!("exec error:{} on:{}", err, val.to_task_pk());
                                    if exec_index + 1 == exec_len && index + 1 == len {
                                        return Err(err);
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
    Err(format!(
        "not find any exec on message :{}",
        val.to_task_pk()
    ))
}
