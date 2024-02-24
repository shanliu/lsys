use std::collections::HashSet;
use std::sync::Arc;

use crate::model::{
    SenderTplConfigModel, SenderTplConfigModelRef, SenderTplConfigStatus, SenderType,
};

use super::logger::LogAppConfig;
use super::SenderResult;
use lsys_core::{now_time, PageParam, RequestEnv};
use lsys_logger::dao::ChangeLogger;
use lsys_setting::dao::Setting;
use lsys_setting::model::SettingModel;
use serde::Serialize;
use serde_json::json;
use sqlx::Pool;
use sqlx_model::{sql_format, Insert, ModelTableName, Update};
use sqlx_model::{Select, SqlQuote};
//发送模板跟发送接口配置

pub struct SenderTplConfig {
    db: Pool<sqlx::MySql>,
    setting: Arc<Setting>,
    send_type: SenderType,
    logger: Arc<ChangeLogger>,
}

impl SenderTplConfig {
    pub fn new(
        db: Pool<sqlx::MySql>,
        setting: Arc<Setting>,
        logger: Arc<ChangeLogger>,
        send_type: SenderType,
    ) -> Self {
        Self {
            db,
            setting,
            send_type,
            logger,
        }
    }
    pub async fn find_by_id(&self, id: &u64) -> SenderResult<SenderTplConfigModel> {
        let data = sqlx_model::Select::type_new::<SenderTplConfigModel>()
            .fetch_one_by_where::<SenderTplConfigModel, _>(
                &sqlx_model::WhereOption::Where(sqlx_model::sql_format!(
                    "sender_type={} and id={} and status={}",
                    self.send_type,
                    id,
                    SenderTplConfigStatus::Enable as i8
                )),
                &self.db,
            )
            .await?;
        Ok(data)
    }
    pub async fn count_config(
        &self,
        id: &Option<u64>,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        tpl_id: &Option<String>,
    ) -> SenderResult<i64> {
        let mut sqlwhere = vec![sql_format!(
            "sender_type={} and status ={}",
            self.send_type as i8,
            SenderTplConfigStatus::Enable as i8
        )];
        if let Some(aid) = id {
            sqlwhere.push(sql_format!("id = {}  ", aid));
        }
        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("app_id = {}  ", aid));
        }
        if let Some(uid) = user_id {
            sqlwhere.push(sql_format!("user_id={} ", uid));
        }
        if let Some(tpl) = tpl_id {
            sqlwhere.push(sql_format!("tpl_id={} ", tpl));
        }
        let sql = format!(
            "select count(*) as total from {}
            where {}",
            SenderTplConfigModel::table_name(),
            sqlwhere.join(" and "),
        );
        Ok(sqlx::query_scalar::<_, i64>(sql.as_str())
            .fetch_one(&self.db)
            .await?)
    }

    pub async fn list_config(
        &self,
        id: &Option<u64>,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        tpl_id: &Option<String>,
        page: &Option<PageParam>,
    ) -> SenderResult<Vec<(SenderTplConfigModel, Option<SettingModel>)>> {
        let mut sqlwhere = vec![sql_format!(
            "sender_type={} and status ={}",
            self.send_type as i8,
            SenderTplConfigStatus::Enable as i8
        )];
        if let Some(aid) = id {
            sqlwhere.push(sql_format!("id = {}  ", aid));
        }
        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("app_id = {}  ", aid));
        }
        if let Some(uid) = user_id {
            sqlwhere.push(sql_format!("user_id={} ", uid));
        }
        if let Some(tpl) = tpl_id {
            sqlwhere.push(sql_format!("tpl_id={} ", tpl));
        }
        let mut sql = format!("{}  order by id desc", sqlwhere.join(" and "));
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }
        let res = Select::type_new::<SenderTplConfigModel>()
            .fetch_all_by_where::<SenderTplConfigModel, _>(
                &sqlx_model::WhereOption::Where(sql),
                &self.db,
            )
            .await?;
        if res.is_empty() {
            return Ok(vec![]);
        }
        let ids = res
            .iter()
            .flat_map(|e| {
                if e.setting_id > 0 {
                    Some(e.setting_id)
                } else {
                    None
                }
            })
            .collect::<HashSet<u64>>()
            .iter()
            .map(|e| e.to_owned())
            .collect::<Vec<u64>>();
        let ali_res = self.setting.find_by_ids(&ids).await?;
        Ok(res
            .into_iter()
            .map(|e| {
                let tmp = ali_res.get(&e.setting_id).map(|e| e.to_owned());
                (e, tmp)
            })
            .collect::<Vec<_>>())
    }

    //关联发送跟aliyun短信的配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_config<C: Serialize>(
        &self,
        name: &str,
        app_id: &u64,
        setting_id: &u64,
        tpl_id: &str,
        config_data: C,
        user_id: &u64,
        add_user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let name = name.to_owned();
        let tpl_id = tpl_id.to_owned();
        let config_data = json!(config_data).to_string();
        let time = now_time().unwrap_or_default();
        let send_type = self.send_type as i8;
        let add = sqlx_model::model_option_set!(SenderTplConfigModelRef,{
            name:name,
            sender_type:send_type,
            app_id:app_id,
            tpl_id:tpl_id,
            config_data:config_data,
            change_time:time,
            user_id:user_id,
            change_user_id:add_user_id,
            setting_id:setting_id,
            status:SenderTplConfigStatus::Enable as i8,
        });

        let id = Insert::<sqlx::MySql, SenderTplConfigModel, _>::new(add)
            .execute(&self.db)
            .await
            .map(|e| e.last_insert_id())?;

        self.logger
            .add(
                &LogAppConfig {
                    action: "add",
                    sender_type: self.send_type as u8,
                    app_id: app_id.to_owned(),
                    name,
                    tpl_id,
                    setting_id: *setting_id,
                    config_data,
                },
                &Some(id),
                &Some(user_id.to_owned()),
                &Some(add_user_id.to_owned()),
                None,
                env_data,
            )
            .await;
        Ok(id)
    }
    //删除发送跟aliyun短信的配置
    pub async fn del_config(
        &self,
        config: &SenderTplConfigModel,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(SenderTplConfigModelRef,{
            status:SenderTplConfigStatus::Delete as i8,
            change_time:time,
            change_user_id:user_id
        });
        let res = Update::<sqlx::MySql, SenderTplConfigModel, _>::new(change)
            .execute_by_pk(config, &self.db)
            .await;

        self.logger
            .add(
                &LogAppConfig {
                    action: "del",
                    sender_type: self.send_type as u8,
                    app_id: config.app_id,
                    name: config.name.to_owned(),
                    tpl_id: config.tpl_id.to_owned(),
                    setting_id: config.setting_id,
                    config_data: config.config_data.to_owned(),
                },
                &Some(config.id),
                &Some(config.change_user_id.to_owned()),
                &Some(user_id.to_owned()),
                None,
                env_data,
            )
            .await;

        match res {
            Err(e) => Err(e)?,
            Ok(mr) => {
                //清理缓存
                Ok(mr.rows_affected())
            }
        }
    }
}
