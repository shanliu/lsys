use crate::model::{
    AccountIndexCat, AccountIndexModel, AccountIndexModelRef, AccountIndexStatus, AccountStatus,
};
use config::Map;
use lsys_core::{now_time, LimitParam};

use super::AccountResult;
use lsys_core::db::{Insert, ModelTableName, SqlExpr, SqlQuote, Update};
use lsys_core::{db_option_executor, model_option_set, sql_format};
use sqlx::{Acquire, MySql, Pool, Transaction};
pub struct AccountIndex {
    db: Pool<MySql>,
}

impl AccountIndex {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }
    //一个用户一个类型只能有一个记录
    pub async fn cat_one_add(
        &self,
        cat: AccountIndexCat,
        account_id: u64,
        index_data: &str,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) -> AccountResult<u64> {
        if index_data.is_empty() {
            return Ok(0);
        }
        let time = now_time()?;
        let index_cat = cat as u8;
        let status = AccountIndexStatus::Enable as i8;
        let index_data = index_data.to_string();
        let vdata = model_option_set!(AccountIndexModelRef,{
            index_cat:index_cat,
            index_data:index_data,
            account_id:account_id,
            status:status,
            change_time:time,
        });
        let change = lsys_core::model_option_set!(AccountIndexModelRef,{
            status:status,
            change_time:time
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Insert::<AccountIndexModel, _>::new(vdata)
            .execute_update(&Update::<AccountIndexModel, _>::new(change), &mut *db)
            .await;
        let addid = match tmp {
            Err(ie) => {
                db.rollback().await?;
                return Err(ie.into());
            }
            Ok(r) => r.last_insert_id(),
        };
        if addid > 0 {
            let del_status = AccountIndexStatus::Delete as i8;
            let change = lsys_core::model_option_set!(AccountIndexModelRef,{
                status:del_status,
                change_time:time
            });
            let tmp = Update::<AccountIndexModel, _>::new(change)
                .execute_by_where(
                    &lsys_core::db::WhereOption::Where(sql_format!(
                        "account_id={} and index_cat={} and id!={}",
                        account_id,
                        index_cat,
                        addid
                    )),
                    &mut *db,
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
    pub async fn add(
        &self,
        cat: AccountIndexCat,
        account_id: u64,
        index_data: &[&str],
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) -> AccountResult<u64> {
        if index_data.is_empty() {
            return Ok(0);
        }

        let time = now_time()?;
        let index_cat = cat as u8;
        let status = AccountIndexStatus::Enable as i8;
        let mut vdata = Vec::with_capacity(index_data.len());
        let tmp_data = index_data.iter().map(|e| e.to_string()).collect::<Vec<_>>();
        for t in tmp_data.iter() {
            vdata.push(model_option_set!(AccountIndexModelRef,{
                index_cat:index_cat,
                index_data:t,
                account_id:account_id,
                status:status,
                change_time:time,
            }));
        }
        let update = model_option_set!(AccountIndexModelRef,{
            status:status,
            change_time:time,
        });
        let res = db_option_executor!(
            db,
            {
                Insert::<AccountIndexModel, _>::new_vec(vdata)
                    .execute_update(
                        &Update::<AccountIndexModel, _>::new(update),
                        db.as_executor(),
                    )
                    .await?
            },
            transaction,
            &self.db
        );
        Ok(res.last_insert_id())
    }
    pub async fn del(
        &self,
        cat: AccountIndexCat,
        account_id: u64,
        index_data: &[&str],
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) -> AccountResult<u64> {
        if index_data.is_empty() {
            return Ok(0);
        }
        let index_cat = cat as u8;
        let time = now_time()?;
        let change = lsys_core::model_option_set!(AccountIndexModelRef,{
            status:AccountIndexStatus::Delete as i8,
            change_time:time,
        });
        let res = db_option_executor!(
            db,
            {
                Update::<AccountIndexModel, _>::new(change)
                    .execute_by_where(
                        &lsys_core::db::WhereOption::Where(sql_format!(
                            "index_data  in ({}) and index_cat={} and account_id={}",
                            index_data,
                            index_cat,
                            account_id
                        )),
                        db.as_executor(),
                    )
                    .await
            },
            transaction,
            &self.db
        )?;
        Ok(res.rows_affected())
    }
    pub async fn cat_del(
        &self,
        cat: AccountIndexCat,
        account_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) -> AccountResult<u64> {
        let index_cat = cat as u8;
        let time = now_time()?;
        let change = lsys_core::model_option_set!(AccountIndexModelRef,{
            status:AccountIndexStatus::Delete as i8,
            change_time:time,
        });
        let res = db_option_executor!(
            db,
            {
                Update::<AccountIndexModel, _>::new(change)
                    .execute_by_where(
                        &lsys_core::db::WhereOption::Where(sql_format!(
                            "index_cat={} and account_id={}",
                            index_cat,
                            account_id
                        )),
                        db.as_executor(),
                    )
                    .await
            },
            transaction,
            &self.db
        )?;
        Ok(res.rows_affected())
    }
    pub async fn account_del(
        &self,
        account_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) -> AccountResult<u64> {
        let time = now_time()?;
        let change = lsys_core::model_option_set!(AccountIndexModelRef,{
            status:AccountIndexStatus::Delete as i8,
            change_time:time,
        });
        let res = db_option_executor!(
            db,
            {
                Update::<AccountIndexModel, _>::new(change)
                    .execute_by_where(
                        &lsys_core::db::WhereOption::Where(sql_format!(
                            "account_id={}",
                            account_id
                        )),
                        db.as_executor(),
                    )
                    .await
            },
            transaction,
            &self.db
        )?;
        Ok(res.rows_affected())
    }
}

pub struct AccountItem {
    pub account_id: u64,
    pub cats: Map<AccountIndexCat, String>,
}
impl AccountIndex {
    ///往指定分类中搜索用户ID
    pub async fn search(
        &self,
        account_status: &[AccountStatus],
        key_word: &str,
        param: &[AccountIndexCat],
        limit: Option<&LimitParam>,
    ) -> AccountResult<(Vec<AccountItem>, Option<u64>)> {
        let account_status_data = if account_status.is_empty() {
            vec![AccountStatus::Enable, AccountStatus::Init]
                .into_iter()
                .map(|e| (e as i8).to_string())
                .collect::<Vec<_>>()
        } else {
            account_status
                .iter()
                .map(|e| (*e as i8).to_string())
                .collect::<Vec<_>>()
        };
        let key_word = key_word.trim();
        let mut sql = if key_word.is_empty() || param.is_empty() {
            sql_format!(
                "select distinct k.account_id,'' as cat_more
                FROM {} as k
                where k.status ={} and k.index_cat={} and k.index_data in ({}) ",
                AccountIndexModel::table_name(),
                AccountIndexStatus::Enable as i8,
                AccountIndexCat::AccountStatus as i8,
                account_status_data,
            )
        } else {
            let index_cat_data = param.iter().map(|e| *e as i8).collect::<Vec<_>>();
            sql_format!(
                "select distinct k.account_id,group_concat(k.index_cat,':',REPLACE(REPLACE(k.index_data,':',' '),',',' ')) as cat_more
                FROM {} as s
                inner join {} as k on s.account_id = k.account_id
                where  s.status ={} and s.index_cat={} and s.index_data in ({}) and k.status ={} and k.index_data like {} {}",
                AccountIndexModel::table_name(),
                AccountIndexModel::table_name(),
                AccountIndexStatus::Enable as i8,
                AccountIndexCat::AccountStatus as i8,
                account_status_data,
                format!("{}%",key_word),
                AccountIndexStatus::Enable as i8,
                if !index_cat_data.is_empty() {
                    SqlExpr(sql_format!("and k.index_cat in ({})", index_cat_data))
                } else {
                    SqlExpr("".to_string())
                },
            )
        };
        if let Some(page) = limit {
            sql = format!(
                "{} {} group by k.account_id order by {} {} ",
                sql,
                page.where_sql("k.account_id", Some("and")),
                page.order_sql("k.account_id"),
                page.limit_sql(),
            );
        } else {
            sql += " order by k.account_id desc";
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
                            if let Ok(cat) = AccountIndexCat::try_from(cat) {
                                if let Some(val) = tmp.next() {
                                    cats.insert(cat, val.to_string());
                                }
                            }
                        }
                    }
                }
                AccountItem {
                    account_id: e.0,
                    cats,
                }
            })
            .collect::<Vec<_>>();
        Ok((out, next))
    }
}
