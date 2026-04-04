use std::collections::HashSet;
use std::sync::Arc;

use crate::model::{SenderTplConfigModel, SenderTplConfigStatus, SenderType};

use super::logger::LogAppConfig;
use super::{SenderError, SenderResult};
use lsys_core::db::{Insert, QueryBuilderExt, TableMeta, Update, WhereClause};
use lsys_core::db::OffsetPageParam;
use lsys_core::fluent_message;
use sqlx::{MySql, QueryBuilder};
use lsys_core::utils::{
    now_time, string_clear, RequestEnv, StringClear, STRING_CLEAR_FORMAT,
};
use lsys_core::valid_key;
use lsys_core::valid_param::{
    ValidError, ValidNumber, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen,
};
use lsys_logger::dao::ChangeLoggerDao;
use lsys_setting::dao::SettingDao;
use lsys_setting::model::SettingModel;
use serde::Serialize;
use serde_json::json;
use sqlx::Pool;

//发送模板跟发送接口配置

pub struct SenderTplConfig {
    db: Pool<sqlx::MySql>,
    setting: Arc<SettingDao>,
    send_type: SenderType,
    logger: Arc<ChangeLoggerDao>,
}

impl SenderTplConfig {
    pub fn new(
        db: Pool<sqlx::MySql>,
        setting: Arc<SettingDao>,
        logger: Arc<ChangeLoggerDao>,
        send_type: SenderType,
    ) -> Self {
        Self {
            db,
            setting,
            send_type,
            logger,
        }
    }
    //检查配置id是否被使用
    pub async fn check_setting_id_used(&self, setting_id: u64) -> SenderResult<()> {
        match sqlx::query_scalar::<_, i64>(&format!(
            "select count(*) as total from {} where setting_id=? and status=?",
            SenderTplConfigModel::table_name()
        ))
        .bind(setting_id)
        .bind(SenderTplConfigStatus::Enable as i8)
        .fetch_one(&self.db)
        .await
        {
            Ok(total) => {
                if total > 0 {
                    return Err(SenderError::System(fluent_message!("sender-setting-used",{
                        "total":total
                    })));
                }
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => return Err(err)?,
        };
        Ok(())
    }

    pub async fn find_by_id(&self, id: u64) -> SenderResult<SenderTplConfigModel> {
        let data = sqlx::query_as::<_, SenderTplConfigModel>(&format!(
            "select * from {} where sender_type=? and id=? and status=?",
            SenderTplConfigModel::table_name()
        ))
        .bind(self.send_type as i8)
        .bind(id)
        .bind(SenderTplConfigStatus::Enable as i8)
        .fetch_one(&self.db)
        .await?;

        Ok(data)
    }
    pub async fn count_config(
        &self,
        id: Option<u64>,
        user_id: Option<u64>,
        app_id: Option<u64>,
        tpl_key: Option<&str>,
        like_tpl_key: Option<&str>,
    ) -> SenderResult<i64> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select count(*) as total from {}",
            SenderTplConfigModel::table_name(),
        ));
        let res = {
            let mut wb = WhereClause::new(&mut qb);
            self.push_list_where(&mut wb, id, user_id, app_id, tpl_key, like_tpl_key).is_none()
        }; if res {
            return Ok(0);
        }
        Ok(qb.build_query_scalar::<i64>().fetch_one(&self.db).await?)
    }
    fn push_list_where<'a, 'args>(
        &self,
        wb: &mut WhereClause<'a, 'args, MySql>,
        id: Option<u64>,
        user_id: Option<u64>,
        app_id: Option<u64>,
        tpl_key: Option<&str>,
        like_tpl_key: Option<&str>,
    ) -> Option<()> {
        wb.and().field_eq("sender_type", self.send_type as i8);
        wb.and().field_eq("status", SenderTplConfigStatus::Enable as i8);
        if let Some(aid) = id {
            wb.and().field_eq("id", aid);
        }
        if let Some(aid) = app_id {
            wb.and().field_eq("app_id", aid);
        }
        if let Some(uid) = user_id {
            wb.and().field_eq("user_id", uid);
        }
        if let Some(tpl) = tpl_key {
            let tpl = string_clear(tpl, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            if tpl.is_empty() {
                return None;
            }
            wb.and().field_eq("tpl_key", tpl);
        }
        if let Some(ref tmp) = like_tpl_key {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            if tmp.is_empty() {
                return None;
            }
            wb.and().field_like("tpl_key", format!("{}%", tmp));
        }
        Some(())
    }
    pub async fn list_config(
        &self,
        id: Option<u64>,
        user_id: Option<u64>,
        app_id: Option<u64>,
        tpl_key: Option<&str>,
        like_tpl_key: Option<&str>,
        page: &OffsetPageParam,
    ) -> SenderResult<Vec<(SenderTplConfigModel, Option<SettingModel>)>> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select * from {}",
            SenderTplConfigModel::table_name(),
        ));
        let res = {
            let mut wb = WhereClause::new(&mut qb);
            self.push_list_where(&mut wb, id, user_id, app_id, tpl_key, like_tpl_key).is_none()
        }; if res {
            return Ok(vec![]);
        }
        qb.push(" order by id desc");
        page.push_limit(&mut qb);
        let res = qb.build_query_as::<SenderTplConfigModel>()
            .fetch_all(&self.db)
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
    async fn add_config_param_valid(
        &self,
        name: &str,
        app_id: u64,
        setting_id: u64,
        tpl_key: &str,
    ) -> SenderResult<()> {
        ValidParam::default()
            .add(
                valid_key!("config_name"),
                &name,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(3, 24)),
            )
            .add(
                valid_key!("app_id"),
                &app_id,
                &ValidParamCheck::default().add_rule(ValidNumber::id()),
            )
            .add(
                valid_key!("sms_setting_id"),
                &setting_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, 32)),
            )
            .add(
                valid_key!("tpl_key"),
                &tpl_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, 32)),
            )
            .check()?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_config<C: Serialize>(
        &self,
        name: &str,
        app_id: u64,
        setting_id: u64,
        tpl_key: &str,
        config_data: C,
        user_id: u64,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.add_config_param_valid(name, app_id, setting_id, tpl_key)
            .await?;

        let sql = format!(
            "select id from {} where sender_type=? and app_id=? and user_id=? and status=? and name=?",
            SenderTplConfigModel::table_name()
        );
        let find_res = sqlx::query_scalar::<_, u64>(&sql)
            .bind(self.send_type as i8)
            .bind(app_id)
            .bind(user_id)
            .bind(SenderTplConfigStatus::Enable as i8)
            .bind(name)
            .fetch_one(&self.db)
            .await;
        match find_res {
            Ok(id) => {
                return Err(SenderError::Vaild(ValidError::message(
                    valid_key!("tpl_name"),
                    fluent_message!("tpl-name-exits",{
                        "id":id,
                        "name":name
                    }),
                )))
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => return Err(err)?,
        }

        let name = name.to_owned();
        let tpl_key = tpl_key.to_owned();
        let config_data = json!(config_data).to_string();
        let time = now_time().unwrap_or_default();
        let send_type = self.send_type as i8;

        let id = Insert::<_,SenderTplConfigModel>::new()
            .set(SenderTplConfigModel::NAME, &name)
            .set(SenderTplConfigModel::SENDER_TYPE, send_type)
            .set(SenderTplConfigModel::APP_ID, app_id)
            .set(SenderTplConfigModel::TPL_KEY, &tpl_key)
            .set(SenderTplConfigModel::CONFIG_DATA, &config_data)
            .set(SenderTplConfigModel::CHANGE_TIME, time)
            .set(SenderTplConfigModel::USER_ID, user_id)
            .set(SenderTplConfigModel::CHANGE_USER_ID, add_user_id)
            .set(SenderTplConfigModel::SETTING_ID, setting_id)
            .set(SenderTplConfigModel::STATUS, SenderTplConfigStatus::Enable as i8)
            .execute(&self.db)
            .await
            .map(|e| e.last_insert_id())?;

        self.logger
            .add(
                &LogAppConfig {
                    action: "add",
                    sender_type: self.send_type as u8,
                    app_id: app_id.to_owned(),
                    name: &name,
                    tpl_key: &tpl_key,
                    user_id,
                    setting_id,
                    config_data: &config_data,
                },
                Some(id),
                Some(add_user_id.to_owned()),
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
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        if SenderTplConfigStatus::Delete.eq(config.status) {
            return Ok(0);
        }
        let time = now_time().unwrap_or_default();
        let res = Update::<_,SenderTplConfigModel>::new()
            .set(SenderTplConfigModel::STATUS, SenderTplConfigStatus::Delete as i8)
            .set(SenderTplConfigModel::CHANGE_TIME, time)
            .set(SenderTplConfigModel::CHANGE_USER_ID, user_id)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", config.id);
            })
            .await;

        self.logger
            .add(
                &LogAppConfig {
                    action: "del",
                    sender_type: self.send_type as u8,
                    app_id: config.app_id,
                    name: config.name.as_str(),
                    tpl_key: config.tpl_key.as_str(),
                    user_id: config.change_user_id,
                    setting_id: config.setting_id,
                    config_data: config.config_data.as_str(),
                },
                Some(config.id),
                Some(user_id.to_owned()),
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

