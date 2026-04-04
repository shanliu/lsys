use std::sync::Arc;

use crate::dao::{SenderError, SenderResult};
use crate::model::{SenderTplBodyModel, SenderTplBodyStatus, SenderType};
use lsys_core::db::OffsetPageParam;
use lsys_core::fluent_message;
use lsys_core::utils::{
    now_time, string_clear, RequestEnv, StringClear, STRING_CLEAR_FORMAT,
};
use lsys_core::valid_key;
use lsys_core::valid_param::{
    ValidNumber, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen,
};

use lsys_core::db::{Insert, QueryBuilderExt, TableMeta, Update, WhereClause};
use sqlx::{MySql, QueryBuilder};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::Pool;
use tera::{Context, Template, Tera};
use tokio::sync::RwLock;
use tracing::{debug, trace};

use super::logger::LogMessageTpls;
//公用模板
pub struct MessageTpls {
    db: Pool<sqlx::MySql>,
    tera: RwLock<Tera>,
    logger: Arc<ChangeLoggerDao>,
}

impl MessageTpls {
    pub fn new(db: Pool<sqlx::MySql>, logger: Arc<ChangeLoggerDao>, tera: Tera) -> Self {
        Self {
            db,
            tera: RwLock::new(tera),
            logger,
        }
    }
    pub async fn find_by_id(&self, id: &u64) -> SenderResult<SenderTplBodyModel> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, SenderTplBodyModel>::one(
            &self.db,
            |qb| { qb.field_eq("id", *id); },
        ).await?)
    }

    async fn add_param_valid(&self, app_id: u64, tpl_id: &str, tpl_data: &str) -> SenderResult<()> {
        ValidParam::default()
            .add(
                valid_key!("app_id"),
                &app_id,
                &ValidParamCheck::default().add_rule(ValidNumber::min(1)),
            )
            .add(
                valid_key!("tpl_id"),
                &tpl_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, 32)),
            )
            .add(
                valid_key!("tpl_data"),
                &tpl_data,
                &ValidParamCheck::default().add_rule(ValidStrlen::range(1, 20000)),
            )
            .check()?;
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn add(
        &self,
        app_id: u64,
        sender_type: SenderType,
        tpl_id: &str,
        tpl_data: &str,
        user_id: u64,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.add_param_valid(app_id, tpl_id, tpl_data).await?;
        let sender_type = sender_type as i8;
        Template::new(&self.tpl_key(sender_type, tpl_id), None, tpl_data)
            .map_err(SenderError::Tera)?;
        let tpl_id = tpl_id.to_owned();
        let user_id = user_id.to_owned();
        let time = now_time().unwrap_or_default();

        let tpl_data = tpl_data.to_owned();
        let status = SenderTplBodyStatus::Enable as i8;

        let res = sqlx::query_as::<_, SenderTplBodyModel>(&format!(
            "select * from {} where app_id=? and tpl_id=? and status=? and user_id=?",
            SenderTplBodyModel::table_name()
        ))
        .bind(app_id)
        .bind(&tpl_id)
        .bind(SenderTplBodyStatus::Enable as i8)
        .bind(user_id)
        .fetch_one(&self.db)
        .await;

        match res {
            Ok(tpl) => {
                if tpl.user_id == user_id && tpl_data.trim() == tpl.tpl_data.trim() {
                    return Ok(tpl.id);
                } else {
                    return Err(SenderError::System(fluent_message!("tpl-exits",
                        {"tpl_id":tpl_id,"id":tpl.id }//"tpl {$tpl_id} bind in other tpl [{$id}]",
                    )));
                }
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }
        let id = Insert::<_,SenderTplBodyModel>::new()
            .set(SenderTplBodyModel::APP_ID, app_id)
            .set(SenderTplBodyModel::SENDER_TYPE, sender_type)
            .set(SenderTplBodyModel::TPL_ID, &tpl_id)
            .set(SenderTplBodyModel::TPL_DATA, &tpl_data)
            .set(SenderTplBodyModel::USER_ID, user_id)
            .set(SenderTplBodyModel::CHANGE_TIME, time)
            .set(SenderTplBodyModel::CHANGE_USER_ID, add_user_id)
            .set(SenderTplBodyModel::STATUS, status)
            .execute(&self.db)
            .await?
            .last_insert_id();

        self.logger
            .add(
                &LogMessageTpls {
                    action: "add",
                    app_id,
                    sender_type,
                    tpl_id: &tpl_id,
                    tpl_data: &tpl_data,
                    user_id,
                },
                Some(id),
                Some(add_user_id.to_owned()),
                None,
                env_data,
            )
            .await;
        Ok(id)
    }
    async fn edit_param_valid(&self, tpl_data: &str) -> SenderResult<()> {
        ValidParam::default()
            .add(
                valid_key!("tpl_data"),
                &tpl_data,
                &ValidParamCheck::default().add_rule(ValidStrlen::range(1, 20000)),
            )
            .check()?;
        Ok(())
    }
    //可取消发送的数据
    pub async fn edit(
        &self,
        tpl: &SenderTplBodyModel,
        tpl_data: &str,
        change_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.edit_param_valid(tpl_data).await?;
        let tkey = self.tpl_key(tpl.sender_type, &tpl.tpl_id);
        Template::new(&tkey, None, tpl_data)?;
        let change_user_id = change_user_id.to_owned();
        let time = now_time().unwrap_or_default();
        let tpl_data = tpl_data.to_owned();

        let row = Update::<_,SenderTplBodyModel>::new()
            .set(SenderTplBodyModel::TPL_DATA, &tpl_data)
            .set(SenderTplBodyModel::CHANGE_USER_ID, change_user_id)
            .set(SenderTplBodyModel::CHANGE_TIME, time)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", tpl.id);
            })
            .await?
            .rows_affected();

        self.tera.write().await.add_raw_template(&tkey, &tpl_data)?;

        self.logger
            .add(
                &LogMessageTpls {
                    action: "edit",
                    sender_type: tpl.sender_type,
                    app_id: tpl.app_id,
                    tpl_id: &tpl.tpl_id,
                    tpl_data: &tpl_data,
                    user_id: tpl.user_id,
                },
                Some(tpl.id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;

        Ok(row)
    }
    //可取消发送的数据
    pub async fn del(
        &self,
        tpl: &SenderTplBodyModel,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        if SenderTplBodyStatus::Delete.eq(tpl.status) {
            return Ok(0);
        }

        let user_id = user_id.to_owned();
        let time = now_time().unwrap_or_default();
        let status = SenderTplBodyStatus::Delete as i8;
        let row = Update::<_,SenderTplBodyModel>::new()
            .set(SenderTplBodyModel::STATUS, status)
            .set(SenderTplBodyModel::USER_ID, user_id)
            .set(SenderTplBodyModel::CHANGE_TIME, time)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", tpl.id);
            })
            .await?
            .rows_affected();
        let tkey = self.tpl_key(tpl.sender_type, &tpl.tpl_id);
        self.tera.write().await.templates.remove(&tkey);
        self.tera.write().await.build_inheritance_chains()?;

        self.logger
            .add(
                &LogMessageTpls {
                    action: "del",
                    sender_type: tpl.sender_type,
                    tpl_id: &tpl.tpl_id,
                    app_id: tpl.app_id,
                    tpl_data: &tpl.tpl_data,
                    user_id: tpl.user_id,
                },
                Some(tpl.id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok(row)
    }
    fn tpl_key(&self, send_type: i8, tpl_id: &str) -> String {
        format!("type:{}-{}", send_type, tpl_id)
    }
    //渲染指定模板内容
    pub async fn render(
        &self,
        app_id: u64,
        sender_type: SenderType,
        tpl_id: &str,
        context: &Context,
    ) -> SenderResult<String> {
        let sender_type = sender_type as i8;
        let tkey = &self.tpl_key(sender_type, tpl_id);
        if !self.tera.read().await.templates.contains_key(tkey) {
            let tpl = sqlx::query_as::<_, SenderTplBodyModel>(&format!(
                "select * from {} where app_id=? and sender_type=? and tpl_id=? and status=?",
                SenderTplBodyModel::table_name()
            ))
            .bind(app_id)
            .bind(sender_type)
            .bind(tpl_id)
            .bind(SenderTplBodyStatus::Enable as i8)
            .fetch_one(&self.db)
            .await?;

            self.tera
                .write()
                .await
                .add_raw_template(tkey, &tpl.tpl_data)?;
            debug!("sender init tpl key {}", tkey);
        };

        trace!(
            "cache tpl {}:{:?}",
            tkey,
            self.tera.read().await.get_template(tkey)
        );

        let data = self.tera.read().await.render(tkey, context)?;
        Ok(data)
    }
    fn push_list_where<'a, 'args>(
        &self,
        wb: &mut WhereClause<'a, 'args, MySql>,
        app_id: u64,
        sender_type: Option<SenderType>,
        id: Option<u64>,
        tpl_id: Option<&str>,
        tpl_id_like: Option<&str>,
    ) -> Option<()> {
        wb.and().field_eq("app_id", app_id);
        wb.and().field_eq("status", SenderTplBodyStatus::Enable as i8);
        if let Some(s) = sender_type {
            wb.and().field_eq("sender_type", s as i8);
        }
        if let Some(s) = id {
            wb.and().field_eq("id", s);
        }
        if let Some(s) = tpl_id {
            wb.and().field_eq("tpl_id", s.to_owned());
        }
        if let Some(tmp) = tpl_id_like {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(128));
            if tmp.is_empty() {
                return None;
            }
            wb.and().field_like("tpl_id", format!("{}%", tmp));
        }
        Some(())
    }
    pub async fn list_data(
        &self,
        app_id: u64,
        sender_type: Option<SenderType>,
        id: Option<u64>,
        tpl_id: Option<&str>,
        tpl_id_like: Option<&str>,
        page: &OffsetPageParam,
    ) -> SenderResult<Vec<SenderTplBodyModel>> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select * from {}",
            SenderTplBodyModel::table_name(),
        ));
        let res = {
            let mut wb = WhereClause::new(&mut qb);
            self.push_list_where(&mut wb, app_id, sender_type, id, tpl_id, tpl_id_like).is_none()
        }; if res {
            return Ok(vec![]);
        }
        qb.push(" order by id desc");
        page.push_limit(&mut qb);
        Ok(qb.build_query_as::<SenderTplBodyModel>()
            .fetch_all(&self.db)
            .await?)
    }
    pub async fn list_count(
        &self,
        app_id: u64,
        sender_type: Option<SenderType>,
        id: Option<u64>,
        tpl_id: Option<&str>,
        tpl_id_like: Option<&str>,
    ) -> SenderResult<i64> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select count(*) as total from {}",
            SenderTplBodyModel::table_name(),
        ));
        let res = {
            let mut wb = WhereClause::new(&mut qb);
            self.push_list_where(&mut wb, app_id, sender_type, id, tpl_id, tpl_id_like).is_none()
        }; if res {
            return Ok(0);
        }
        Ok(qb.build_query_scalar::<i64>().fetch_one(&self.db).await?)
    }
}


