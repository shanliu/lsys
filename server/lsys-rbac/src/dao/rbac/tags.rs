use lsys_core::{now_time, PageParam, RequestEnv};
use lsys_logger::dao::ChangeLogger;
use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{
    executor_option, model_option_set, sql_format, Insert, ModelTableName, Select, SqlQuote,
    Update, WhereOption,
};

use crate::model::{RbacTagsModel, RbacTagsModelRef, RbacTagsSource, RbacTagsStatus};

use super::{LogTag, UserRbacResult};
use std::sync::Arc;
pub struct RbacTags {
    db: Pool<MySql>,
    logger: Arc<ChangeLogger>,
}

impl RbacTags {
    pub fn new(db: Pool<MySql>, logger: Arc<ChangeLogger>) -> Self {
        Self { db, logger }
    }
    pub async fn count_by_name(
        &self,
        user_id: u64,
        name: &[String],
        from_ids: &Option<Vec<u64>>,
        sorce: RbacTagsSource,
        other_where: Option<&String>,
    ) -> UserRbacResult<i64> {
        if name.is_empty() {
            return Ok(0);
        }
        let mut sql = sql_format!(
            "select count(*) as total from {} where name IN ({}) and user_id = ? and  from_source=? and status=?",
            RbacTagsModel::table_name(),
            name
        );
        if let Some(fids) = from_ids {
            if fids.is_empty() {
                return Ok(0);
            }
            sql += &sql_format!(" and from_id IN ({})", fids);
        }
        if let Some(wsql) = other_where {
            sql += wsql;
        }
        let mut query = sqlx::query_scalar::<_, i64>(&sql);
        query = query
            .bind(user_id)
            .bind(sorce as i8)
            .bind(RbacTagsStatus::Enable as i8);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    /// 根据TAG 名获取列表
    pub async fn find_by_name(
        &self,
        user_id: u64,
        name: &[String],
        from_ids: &Option<Vec<u64>>,
        sorce: RbacTagsSource,
        page: &Option<PageParam>,
    ) -> UserRbacResult<Vec<RbacTagsModel>> {
        if name.is_empty() {
            return Ok(vec![]);
        }
        let mut sql = sql_format!(
            "name IN ({}) and user_id = ? and  from_source=? and status=?",
            name
        );
        if let Some(fids) = from_ids {
            if fids.is_empty() {
                return Ok(vec![]);
            }
            sql += &sql_format!(" and from_id IN ({})", fids);
        }
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }
        let data = Select::type_new::<RbacTagsModel>()
            .fetch_all_by_where_call::<RbacTagsModel, _, _>(
                &sql,
                |mut tmp, _| {
                    tmp = tmp
                        .bind(user_id)
                        .bind(sorce as i8)
                        .bind(RbacTagsStatus::Enable as i8);
                    tmp
                },
                &self.db,
            )
            .await?;
        Ok(data)
    }
    /// 根据目标ID获取列表
    pub async fn find_by_ids(
        &self,
        from_ids: &[u64],
        sorce: RbacTagsSource,
    ) -> UserRbacResult<Vec<RbacTagsModel>> {
        if from_ids.is_empty() {
            return Ok(vec![]);
        }
        let data = Select::type_new::<RbacTagsModel>()
            .fetch_all_by_where_call::<RbacTagsModel, _, _>(
                &sql_format!("from_id IN ({}) and from_source=? and status=?", from_ids),
                |tmp, _| tmp.bind(sorce as i8).bind(RbacTagsStatus::Enable as i8),
                &self.db,
            )
            .await?;
        Ok(data)
    }
    /// 根据目标ID获取列表
    pub async fn group_by_user_id(
        &self,
        user_id: u64,
        sorce: RbacTagsSource,
    ) -> UserRbacResult<Vec<(String, i64)>> {
        let sql = sql_format!(
            "select name,count(*) as total from {}
            where user_id = {} and from_source={} and status={} group by name",
            RbacTagsModel::table_name(),
            user_id,
            sorce as i8,
            RbacTagsStatus::Enable as i8,
        );
        let data = sqlx::query_as::<_, (String, i64)>(sql.as_str())
            .fetch_all(&self.db)
            .await?;
        Ok(data)
    }
    /// 删除指定目标的TAG
    pub async fn del_tags<'t>(
        &self,
        from_id: u64,
        source: RbacTagsSource,
        delete_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserRbacResult<()> {
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(RbacTagsModelRef,{
            change_time:time,
            change_user_id:delete_user_id,
            status:(RbacTagsStatus::Delete as i8)
        });
        executor_option!(
            {
                Update::<sqlx::MySql, RbacTagsModel, _>::new(change)
                    .execute_by_where_call(
                        "from_source=? and from_id=?",
                        |temp, _| temp.bind(source as i8).bind(from_id),
                        db,
                    )
                    .await?;
            },
            transaction,
            &self.db,
            db
        );

        self.logger
            .add(
                &LogTag {
                    action: "del",
                    tags: None,
                    from_source: source,
                },
                &Some(from_id),
                &None,
                &Some(delete_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
    /// 设置指定目标的TAG
    #[allow(clippy::too_many_arguments)]
    pub async fn set_tags<'t>(
        &self,
        from_id: u64,
        user_id: u64,
        source: RbacTagsSource,
        tags: &[String],
        change_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserRbacResult<()> {
        let from_source = source as i8;
        let time = now_time().unwrap_or_default();

        let ftags = Select::type_new::<RbacTagsModel>()
            .fetch_all_by_where_call::<RbacTagsModel, _, _>(
                "from_source=? and from_id=? and status=?",
                |tmp, _| {
                    tmp.bind(from_source)
                        .bind(from_id)
                        .bind(RbacTagsStatus::Enable as i8)
                },
                &self.db,
            )
            .await?;

        let mut del_tag = vec![];
        for itag in ftags.iter() {
            if !tags.contains(&itag.name) {
                del_tag.push(itag.id)
            }
        }
        let stag = ftags.into_iter().map(|e| e.name).collect::<Vec<_>>();
        let add_tag = tags
            .iter()
            .filter(|e| !stag.contains(e))
            .collect::<Vec<_>>();

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        if !add_tag.is_empty() {
            let mut idata = Vec::with_capacity(add_tag.len());
            for tag in add_tag.iter() {
                let mut item = model_option_set!(RbacTagsModelRef,{
                    from_source:from_source,
                    user_id:user_id,
                    from_id:from_id,
                    change_time:time,
                    change_user_id:change_user_id,
                    status:(RbacTagsStatus::Enable as i8),
                });
                item.name = Some(tag);
                idata.push(item);
            }
            let tmp = Insert::<sqlx::MySql, RbacTagsModel, _>::new_vec(idata)
                .execute(&mut db)
                .await;
            if let Err(e) = tmp {
                db.rollback().await?;
                return Err(e)?;
            };
        }

        if !del_tag.is_empty() {
            let ddata = model_option_set!(RbacTagsModelRef,{
                change_time:time,
                change_user_id:change_user_id,
                status:(RbacTagsStatus::Delete as i8),
            });
            let tmp = Update::<sqlx::MySql, RbacTagsModel, _>::new(ddata)
                .execute_by_where(
                    &WhereOption::Where(sql_format!("id in ({})", del_tag)),
                    &mut db,
                )
                .await;

            match tmp {
                Err(e) => {
                    db.rollback().await?;
                    return Err(e)?;
                }
                Ok(_) => {
                    db.commit().await?;
                }
            };
        } else {
            db.commit().await?;
        }

        self.logger
            .add(
                &LogTag {
                    action: "set",
                    tags: Some(tags.to_owned()),
                    from_source: source,
                },
                &Some(from_id),
                &Some(user_id),
                &Some(change_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
}
