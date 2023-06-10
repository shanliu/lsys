use crate::model::{UserIndexCat, UserIndexModel, UserIndexModelRef, UserIndexStatus, UserStatus};
use config::Map;
use lsys_core::{now_time, LimitParam};

use super::UserAccountResult;
use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{
    executor_option, model_option_set, sql_array_str, sql_format, Insert, ModelTableName, SqlQuote,
    Update,
};

pub struct UserIndex {
    db: Pool<MySql>,
}

impl UserIndex {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }
    //一个用户一个类型只能有一个记录
    pub async fn cat_one_add<'t>(
        &self,
        cat: UserIndexCat,
        user_id: u64,
        index_data: &str,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        if index_data.is_empty() {
            return Ok(0);
        }
        let time = now_time()?;
        let index_cat = cat as u8;
        let status = UserIndexStatus::Enable as i8;
        let index_data = index_data.to_string();
        let vdata = model_option_set!(UserIndexModelRef,{
            index_cat:index_cat,
            index_data:index_data,
            user_id:user_id,
            status:status,
            change_time:time,
        });
        let change = sqlx_model::model_option_set!(UserIndexModelRef,{
            status:status,
            change_time:time
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Insert::<sqlx::MySql, UserIndexModel, _>::new(vdata)
            .execute_update(&Update::<MySql, UserIndexModel, _>::new(change), &mut db)
            .await;
        let addid = match tmp {
            Err(ie) => {
                db.rollback().await?;
                return Err(ie.into());
            }
            Ok(r) => r.last_insert_id(), //@todo 重复更新 ID是否返回
        };
        if addid > 0 {
            let del_status = UserIndexStatus::Delete as i8;
            let change = sqlx_model::model_option_set!(UserIndexModelRef,{
                status:del_status,
                change_time:time
            });
            let tmp = Update::<sqlx::MySql, UserIndexModel, _>::new(change)
                .execute_by_where_call(
                    "user_id=? and index_cat=? and id!=?",
                    |res, _| res.bind(user_id).bind(index_cat).bind(addid),
                    &mut db,
                )
                .await;
            if let Err(ie) = tmp {
                db.rollback().await?;
                return Err(ie.into());
            }
        }
        db.commit().await?;
        Ok(addid)
    }
    pub async fn add<'t>(
        &self,
        cat: UserIndexCat,
        user_id: u64,
        index_data: &[String],
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        if index_data.is_empty() {
            return Ok(0);
        }

        let time = now_time()?;
        let index_cat = cat as u8;
        let status = UserIndexStatus::Enable as i8;
        let mut vdata = Vec::with_capacity(index_data.len());
        for t in index_data.iter() {
            vdata.push(model_option_set!(UserIndexModelRef,{
                index_cat:index_cat,
                index_data:t,
                user_id:user_id,
                status:status,
                change_time:time,
            }));
        }
        let update = model_option_set!(UserIndexModelRef,{
            status:status,
            change_time:time,
        });
        let res = executor_option!(
            {
                Insert::<sqlx::MySql, UserIndexModel, _>::new_vec(vdata)
                    .execute_update(&Update::<MySql, UserIndexModel, _>::new(update), db)
                    .await?
            },
            transaction,
            &self.db,
            db
        );
        Ok(res.last_insert_id())
    }
    pub async fn del<'t>(
        &self,
        cat: UserIndexCat,
        user_id: u64,
        index_data: &[String],
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        if index_data.is_empty() {
            return Ok(0);
        }
        let index_cat = cat as u8;
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(UserIndexModelRef,{
            status:UserIndexStatus::Delete as i8,
            change_time:time,
        });
        let res = executor_option!(
            {
                Update::<sqlx::MySql, UserIndexModel, _>::new(change)
                    .execute_by_where_call(
                        &sql_format!(
                            "index_data  in ({}) and index_cat=? and user_id=?",
                            index_data
                        ),
                        |mut res, _| {
                            res = res.bind(index_cat);
                            res = res.bind(user_id);

                            res
                        },
                        db,
                    )
                    .await
            },
            transaction,
            &self.db,
            db
        )?;
        Ok(res.rows_affected())
    }
    pub async fn cat_del<'t>(
        &self,
        cat: UserIndexCat,
        user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        let index_cat = cat as u8;
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(UserIndexModelRef,{
            status:UserIndexStatus::Delete as i8,
            change_time:time,
        });
        let res = executor_option!(
            {
                Update::<sqlx::MySql, UserIndexModel, _>::new(change)
                    .execute_by_where_call(
                        &sql_format!("index_cat=? and user_id=?"),
                        |mut res, _| {
                            res = res.bind(index_cat);
                            res = res.bind(user_id);
                            res
                        },
                        db,
                    )
                    .await
            },
            transaction,
            &self.db,
            db
        )?;
        Ok(res.rows_affected())
    }
    pub async fn user_del<'t>(
        &self,
        user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(UserIndexModelRef,{
            status:UserIndexStatus::Delete as i8,
            change_time:time,
        });
        let res = executor_option!(
            {
                Update::<sqlx::MySql, UserIndexModel, _>::new(change)
                    .execute_by_where_call(
                        "user_id=?",
                        |mut res, _| {
                            res = res.bind(user_id);
                            res
                        },
                        db,
                    )
                    .await
            },
            transaction,
            &self.db,
            db
        )?;
        Ok(res.rows_affected())
    }

    //     ///查找符合条件用户ID
    //     ///指定索引类型+索引包含数据
    //     pub async fn find_user(
    //         &self,
    //         user_status: &[UserStatus],
    //         param: &[(UserIndexCat, &str)],
    //         limit: &Option<LimitParam>,
    //     ) -> UserAccountResult<(Vec<u64>, Option<u64>)> {
    //         let user_status_data = if user_status.is_empty() {
    //             vec![UserStatus::Enable as i8, UserStatus::Init as i8]
    //         } else {
    //             user_status.iter().map(|e| *e as i8).collect::<Vec<_>>()
    //         };
    //         let mut sql = Vec::with_capacity(param.len());
    //         for (i, tmp) in param.iter().enumerate() {
    //             sql.push((
    //                 format!(
    //                     "{} as t{i} on t.user_id=t{i}.user_id",
    //                     UserIndexModel::table_name()
    //                 ),
    //                 sql_format!(
    //                     "t{i}.status={} and t{i}.index_cat={} and t{i}.index_data= {}",
    //                     UserIndexStatus::Enable as i8,
    //                     tmp.0,
    //                     format!("{}%", tmp.1)
    //                 ),
    //             ));
    //         }
    //         let mut sql = sql_format!(
    //             "select distinct t.user_id
    //             FROM {} as t inner join  {}
    //             where t.status ={} and t.index_cat={} and t.index_data in ({}) and {}  ",
    //             UserIndexModel::table_name(),
    //             sql.iter()
    //                 .map(|e| e.0.to_owned())
    //                 .collect::<Vec<_>>()
    //                 .join(" inner join "),
    //             UserIndexStatus::Enable as i8,
    //             UserIndexCat::UserStatus as i8,
    //             user_status_data,
    //             sql.iter()
    //                 .map(|e| e.1.to_owned())
    //                 .collect::<Vec<_>>()
    //                 .join(" and "),
    //         );
    //         if let Some(page) = limit {
    //             sql = format!(
    //                 "{} {} group by user_id order by {} {} ",
    //                 sql,
    //                 page.where_sql("t.user_id",Some("and")),
    //                 page.order_sql("t.user_id"),
    //                 page.limit_sql(),
    //             );
    //         } else {
    //             sql += " order by t.user_id desc";
    //         }
    //         let mut res = sqlx::query_scalar::<_, u64>(sql.as_str())
    //             .fetch_all(&self.db)
    //             .await?;

    //         let next = limit
    //             .as_ref()
    //             .map(|page| page.tidy(&mut res))
    //             .unwrap_or_default();
    //         Ok((res, next))
    //     }
}

pub struct UserItem {
    pub user_id: u64,
    pub cats: Map<UserIndexCat, String>,
}
impl UserIndex {
    ///往指定分类中搜索用户ID
    pub async fn search_user(
        &self,
        user_status: &[UserStatus],
        key_word: &str,
        param: &[UserIndexCat],
        limit: &Option<LimitParam>,
    ) -> UserAccountResult<(Vec<UserItem>, Option<u64>)> {
        let user_status_data = if user_status.is_empty() {
            vec![UserStatus::Enable, UserStatus::Init]
                .into_iter()
                .map(|e| (e as i8).to_string())
                .collect::<Vec<_>>()
        } else {
            user_status
                .iter()
                .map(|e| (*e as i8).to_string())
                .collect::<Vec<_>>()
        };
        let key_word = key_word.trim();
        let mut sql = if key_word.is_empty() || param.is_empty() {
            sql_format!(
                "select distinct k.user_id,'' as cat_more
                FROM {} as k 
                where k.status ={} and k.index_cat={} and k.index_data in ({}) ",
                UserIndexModel::table_name(),
                UserIndexStatus::Enable as i8,
                UserIndexCat::UserStatus as i8,
                user_status_data,
            )
        } else {
            sql_format!(
                "select distinct k.user_id,group_concat(k.index_cat,':',REPLACE(REPLACE(k.index_data,':',' '),',',' ')) as cat_more
                FROM {} as s
                inner join {} as k on s.user_id = k.user_id
                where  s.status ={} and s.index_cat={} and s.index_data in ({}) and k.status ={} and {} and k.index_data like {} ",
                UserIndexModel::table_name(),
                UserIndexModel::table_name(),
                UserIndexStatus::Enable as i8,
                UserIndexCat::UserStatus as i8,
                user_status_data,
                UserIndexStatus::Enable as i8,
                sql_array_str!(" k.index_cat in ({}) ",param.iter().map(|e|*e as i8).collect::<Vec<_>>()),
                format!("{}%",key_word),
            )
        };
        if let Some(page) = limit {
            sql = format!(
                "{} {} group by k.user_id order by {} {} ",
                sql,
                page.where_sql("k.user_id", Some("and")),
                page.order_sql("k.user_id"),
                page.limit_sql(),
            );
        } else {
            sql += " order by k.user_id desc";
        }
        let mut res = sqlx::query_as::<_, (u64, String)>(sql.as_str())
            .fetch_all(&self.db)
            .await?;
        if let Some(LimitParam::Next { .. }) = limit {
            res.reverse();
        };
        let next = limit
            .as_ref()
            .map(|page| page.tidy(&mut res))
            .unwrap_or_default()
            .map(|e| e.0);
        let out = res
            .into_iter()
            .map(|e| {
                let mut cats = Map::new();
                for item in e.1.split(',') {
                    let mut tmp = item.split(':');
                    if let Some(cat) = tmp.next() {
                        if let Ok(cat) = cat.parse::<i8>() {
                            if let Ok(cat) = UserIndexCat::try_from(cat) {
                                if let Some(val) = tmp.next() {
                                    cats.insert(cat, val.to_string());
                                }
                            }
                        }
                    }
                }
                UserItem { user_id: e.0, cats }
            })
            .collect::<Vec<_>>();
        Ok((out, next))
    }
}
