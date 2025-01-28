use std::sync::Arc;

use crate::dao::{SenderError, SenderResult};
use crate::model::{SenderTplBodyModel, SenderTplBodyModelRef, SenderTplBodyStatus, SenderType};
use lsys_core::{fluent_message, now_time, PageParam, RequestEnv};

use lsys_core::db::{Insert, ModelTableName, SqlExpr, SqlQuote, Update};
use lsys_core::{model_option_set, sql_format};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::Pool;
use tera::{Context, Template, Tera};
use tokio::sync::RwLock;

use super::logger::LogMessageTpls;
//公用模板
pub struct MessageTpls {
    db: Pool<sqlx::MySql>,
    tera: RwLock<Tera>,
    logger: Arc<ChangeLoggerDao>,
}

impl MessageTpls {
    pub fn new(db: Pool<sqlx::MySql>, logger: Arc<ChangeLoggerDao>) -> Self {
        Self {
            db,
            tera: RwLock::new(Tera::default()),

            logger,
        }
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        SenderTplBodyModel,
        SenderResult<SenderTplBodyModel>,
        id,
        "id={id}"
    );
    pub async fn add(
        &self,
        sender_type: SenderType,
        tpl_id: &str,
        tpl_data: &str,
        user_id: u64,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let sender_type = sender_type as i8;
        Template::new(&self.tpl_key(sender_type, tpl_id), None, tpl_data)
            .map_err(SenderError::Tera)?;
        let tpl_id = tpl_id.to_owned();
        let user_id = user_id.to_owned();
        let time = now_time().unwrap_or_default();

        let tpl_data = tpl_data.to_owned();
        let status = SenderTplBodyStatus::Enable as i8;

        let res = sqlx::query_as::<_, SenderTplBodyModel>(&sql_format!(
            "select * from {} where tpl_id={} and status = {} and user_id = {} ",
            SenderTplBodyModel::table_name(),
            tpl_id,
            SenderTplBodyStatus::Enable,
            user_id
        ))
        .fetch_one(&self.db)
        .await;

        match res {
            Ok(tpl) => {
                if tpl.user_id == user_id {
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
        let idata = model_option_set!(SenderTplBodyModelRef,{
            sender_type:sender_type,
            tpl_id:tpl_id,
            tpl_data:tpl_data,
            user_id:user_id,
            change_time:time,
            change_user_id:add_user_id,
            status:status,
        });
        let id = Insert::<sqlx::MySql, SenderTplBodyModel, _>::new(idata)
            .execute(&self.db)
            .await?
            .last_insert_id();

        self.logger
            .add(
                &LogMessageTpls {
                    action: "add",
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
    //可取消发送的数据
    pub async fn edit(
        &self,
        tpl: &SenderTplBodyModel,
        tpl_data: &str,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let tkey = self.tpl_key(tpl.sender_type, &tpl.tpl_id);
        Template::new(&tkey, None, tpl_data)?;
        let user_id = user_id.to_owned();
        let time = now_time().unwrap_or_default();
        let tpl_data = tpl_data.to_owned();

        let change = model_option_set!(SenderTplBodyModelRef,{
            tpl_data:tpl_data,
            change_user_id:user_id,
            change_time:time,
        });
        let row = Update::<sqlx::MySql, SenderTplBodyModel, _>::new(change)
            .execute_by_pk(tpl, &self.db)
            .await?
            .rows_affected();
        self.tera.write().await.add_raw_template(&tkey, &tpl_data)?;

        self.logger
            .add(
                &LogMessageTpls {
                    action: "edit",
                    sender_type: tpl.sender_type,
                    tpl_id: &tpl.tpl_id,
                    tpl_data: &tpl_data,
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
    //可取消发送的数据
    pub async fn del(
        &self,
        tpl: &SenderTplBodyModel,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let user_id = user_id.to_owned();
        let time = now_time().unwrap_or_default();
        let status = SenderTplBodyStatus::Delete as i8;
        let change = model_option_set!(SenderTplBodyModelRef,{
            status:status,
            user_id:user_id,
            change_time:time,
        });
        let row = Update::<sqlx::MySql, SenderTplBodyModel, _>::new(change)
            .execute_by_pk(tpl, &self.db)
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
        format!("{}-{}", send_type, tpl_id)
    }
    //渲染指定模板内容
    pub async fn render(
        &self,
        sender_type: SenderType,
        tpl_id: &str,
        context: &Context,
    ) -> SenderResult<String> {
        let sender_type = sender_type as i8;
        let tkey = &self.tpl_key(sender_type, tpl_id);
        if !self.tera.read().await.templates.contains_key(tkey) {
            let tpl = sqlx::query_as::<_, SenderTplBodyModel>(&sql_format!(
                "select * from {} where sender_type={} and tpl_id={} and status = {}",
                SenderTplBodyModel::table_name(),
                sender_type,
                tpl_id,
                SenderTplBodyStatus::Enable
            ))
            .fetch_one(&self.db)
            .await?;

            self.tera
                .write()
                .await
                .add_raw_template(tkey, &tpl.tpl_data)?;
        };
        let data = self.tera.read().await.render(tkey, context)?;
        Ok(data)
    }
    fn list_where_sql(
        &self,
        user_id: u64,
        sender_type: Option<SenderType>,
        id: Option<u64>,
        tpl_id: Option<&str>,
    ) -> String {
        let mut sqlwhere = vec![sql_format!(
            "user_id={} and status ={}",
            user_id,
            SenderTplBodyStatus::Enable
        )];
        if let Some(s) = sender_type {
            sqlwhere.push(sql_format!("sender_type={} ", s));
        }
        if let Some(s) = id {
            sqlwhere.push(sql_format!("id={} ", s));
        }
        if let Some(s) = tpl_id {
            sqlwhere.push(sql_format!("tpl_id={} ", s));
        }
        sqlwhere.join(" and ")
    }
    pub async fn list_data(
        &self,
        user_id: u64,
        sender_type: Option<SenderType>,
        id: Option<u64>,
        tpl_id: Option<&str>,
        page: Option<&PageParam>,
    ) -> SenderResult<Vec<SenderTplBodyModel>> {
        let sqlwhere = self.list_where_sql(user_id, sender_type, id, tpl_id);
        // let page_sql = if let Some(pdat) = page {
        //     format!(
        //         " order by id desc limit {} offset {} ",
        //         pdat.limit, pdat.offset
        //     )
        // } else {
        //     " ".to_string()
        // };
        let sql = sql_format!(
            "select * from {} {} order by id desc {}",
            SenderTplBodyModel::table_name(),
            if !sqlwhere.is_empty() {
                SqlExpr(format!("where {}", sqlwhere))
            } else {
                SqlExpr("".to_string())
            },
            match page {
                Some(pdat) => SqlExpr(format!("limit {} offset {}", pdat.limit, pdat.offset)),
                None => SqlExpr("".to_string()),
            }
        );
        Ok(sqlx::query_as::<_, SenderTplBodyModel>(&sql)
            .fetch_all(&self.db)
            .await?)
    }
    pub async fn list_count(
        &self,
        user_id: u64,
        sender_type: Option<SenderType>,
        id: Option<u64>,
        tpl_id: Option<&str>,
    ) -> SenderResult<i64> {
        let sqlwhere = self.list_where_sql(user_id, sender_type, id, tpl_id);
        let sql = sql_format!(
            "select count(*) as total from {} where {}",
            SenderTplBodyModel::table_name(),
            SqlExpr(sqlwhere)
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
}
