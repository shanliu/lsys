use std::sync::Arc;

use crate::dao::{SenderError, SenderResult};
use crate::model::{SenderTplStatus, SenderTplsModel, SenderTplsModelRef, SenderType};
use lsys_core::{get_message, now_time, FluentMessage, PageParam};

use sqlx::Pool;
use sqlx_model::{
    model_option_set, sql_format, Insert, ModelTableName, Select, Update, WhereOption,
};
use sqlx_model::{SqlExpr, SqlQuote};
use tera::{Context, Template, Tera};
use tokio::sync::RwLock;
//公用模板
pub struct MessageTpls {
    db: Pool<sqlx::MySql>,
    tera: RwLock<Tera>,
    fluent: Arc<FluentMessage>,
}

impl MessageTpls {
    pub fn new(db: Pool<sqlx::MySql>, fluent: Arc<FluentMessage>) -> Self {
        Self {
            db,
            tera: RwLock::new(Tera::default()),
            fluent,
        }
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        SenderTplsModel,
        SenderResult<SenderTplsModel>,
        id,
        "id={id}"
    );
    pub async fn add(
        &self,
        sender_type: SenderType,
        tpl_id: &str,
        tpl_data: &str,
        user_id: &u64,
        add_user_id: &u64,
    ) -> SenderResult<u64> {
        let sender_type = sender_type as i8;
        Template::new(&self.tpl_key(sender_type, tpl_id), None, tpl_data)
            .map_err(SenderError::Tpl)?;
        let tpl_id = tpl_id.to_owned();
        let user_id = user_id.to_owned();
        let time = now_time().unwrap_or_default();

        let tpl_data = tpl_data.to_owned();
        let status = SenderTplStatus::Enable as i8;
        let res = Select::type_new::<SenderTplsModel>()
            .fetch_one_by_where_call::<SenderTplsModel, _, _>(
                " tpl_id=? and status = ? and user_id = ?",
                |mut res, _| {
                    res = res.bind(tpl_id.clone());
                    res = res.bind(SenderTplStatus::Enable as i8);
                    res = res.bind(user_id);
                    res
                },
                &self.db,
            )
            .await;
        match res {
            Ok(tpl) => {
                if tpl.user_id == user_id {
                    return Ok(tpl.id);
                } else {
                    return Err(SenderError::System(get_message!(&self.fluent,
                        "tpl-exits","tpl {$tpl_id} bind in other tpl [{$id}]",
                        ["tpl_id"=>tpl_id,"id"=>tpl.id ]
                    )));
                }
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }
        let idata = model_option_set!(SenderTplsModelRef,{
            sender_type:sender_type,
            tpl_id:tpl_id,
            tpl_data:tpl_data,
            user_id:user_id,
            change_time:time,
            change_user_id:add_user_id,
            status:status,
        });
        Ok(Insert::<sqlx::MySql, SenderTplsModel, _>::new(idata)
            .execute(&self.db)
            .await?
            .last_insert_id())
    }
    //可取消发送的数据
    pub async fn edit(
        &self,
        tpl: &SenderTplsModel,

        tpl_data: &str,
        user_id: &u64,
    ) -> SenderResult<u64> {
        let tkey = self.tpl_key(tpl.sender_type, &tpl.tpl_id);
        Template::new(&tkey, None, tpl_data)?;
        let user_id = user_id.to_owned();
        let time = now_time().unwrap_or_default();
        let tpl_data = tpl_data.to_owned();

        let change = model_option_set!(SenderTplsModelRef,{

            tpl_data:tpl_data,
            user_id:user_id,
            change_time:time,
        });
        let row = Update::<sqlx::MySql, SenderTplsModel, _>::new(change)
            .execute_by_pk(tpl, &self.db)
            .await?
            .rows_affected();
        self.tera.write().await.templates.remove(&tkey);
        self.tera.write().await.build_inheritance_chains()?;
        Ok(row)
    }
    //可取消发送的数据
    pub async fn del(&self, tpl: &SenderTplsModel, user_id: &u64) -> SenderResult<u64> {
        let user_id = user_id.to_owned();
        let time = now_time().unwrap_or_default();
        let status = SenderTplStatus::Delete as i8;
        let change = model_option_set!(SenderTplsModelRef,{
            status:status,
            user_id:user_id,
            change_time:time,
        });
        let row = Update::<sqlx::MySql, SenderTplsModel, _>::new(change)
            .execute_by_pk(tpl, &self.db)
            .await?
            .rows_affected();
        let tkey = self.tpl_key(tpl.sender_type, &tpl.tpl_id);
        self.tera.write().await.templates.remove(&tkey);
        self.tera.write().await.build_inheritance_chains()?;
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
        if self.tera.read().await.templates.get(tkey).is_none() {
            let tpl = Select::type_new::<SenderTplsModel>()
                .fetch_one_by_where_call::<SenderTplsModel, _, _>(
                    "sender_type=? and tpl_id=? and status = ?",
                    |mut res, _| {
                        res = res.bind(sender_type);
                        res = res.bind(tpl_id.to_owned());
                        res = res.bind(SenderTplStatus::Enable as i8);
                        res
                    },
                    &self.db,
                )
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
        user_id: &u64,
        sender_type: &Option<SenderType>,
        id: &Option<u64>,
        tpl_id: &Option<String>,
    ) -> String {
        let mut sqlwhere = vec![sql_format!(
            "user_id={} and status ={}",
            user_id,
            SenderTplStatus::Enable
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
        user_id: &u64,
        sender_type: &Option<SenderType>,
        id: &Option<u64>,
        tpl_id: &Option<String>,
        page: &Option<PageParam>,
    ) -> SenderResult<Vec<SenderTplsModel>> {
        let sqlwhere = self.list_where_sql(user_id, sender_type, id, tpl_id);
        let page_sql = if let Some(pdat) = page {
            format!(
                " order by id desc limit {} offset {} ",
                pdat.limit, pdat.offset
            )
        } else {
            " order by id desc".to_string()
        };
        let sql = if !sqlwhere.is_empty() {
            WhereOption::Where(sqlwhere + page_sql.as_str())
        } else {
            WhereOption::None
        };
        Ok(Select::type_new::<SenderTplsModel>()
            .fetch_all_by_where::<SenderTplsModel, _>(&sql, &self.db)
            .await?)
    }
    pub async fn list_count(
        &self,
        user_id: &u64,
        sender_type: &Option<SenderType>,
        id: &Option<u64>,
        tpl_id: &Option<String>,
    ) -> SenderResult<i64> {
        let sqlwhere = self.list_where_sql(user_id, sender_type, id, tpl_id);
        let sql = sql_format!(
            "select count(*) as total from {} where {}",
            SenderTplsModel::table_name(),
            SqlExpr(sqlwhere)
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
}
