use crate::model::{AccountIndexCat, AccountIndexModel, AccountIndexStatus, AccountStatus};
use config::Map;
use lsys_core::utils::{now_time, string_clear, StringClear};

use super::AccountResult;
use lsys_core::db::{
    BatchInsert, CursorPageData, CursorPageParam, Insert, QueryBuilderExt, TableMeta,
    Update, OptionTxExecutor
};
use sqlx::{Acquire, MySql, Pool, QueryBuilder, Transaction};
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
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Insert::<_,AccountIndexModel>::new()
            .set(AccountIndexModel::INDEX_CAT, index_cat)
            .set(AccountIndexModel::INDEX_DATA, &index_data)
            .set(AccountIndexModel::ACCOUNT_ID, account_id)
            .set(AccountIndexModel::STATUS, status)
            .set(AccountIndexModel::CHANGE_TIME, time)
            .execute_update(
                Update::<_,AccountIndexModel>::new()
                    .set(AccountIndexModel::STATUS, status)
                    .set(AccountIndexModel::CHANGE_TIME, time),
                &mut *db,
            )
            .await;
        let addid = match tmp {
            Err(ie) => {
                db.rollback().await?;
                return Err(ie.into());
            }
            Ok(row) => {
                if row.last_insert_id() == 0 {
                    sqlx::query_scalar::<_, u64>(&format!(
                        "select id from {} where account_id=? and index_cat=? and index_data=?",
                        AccountIndexModel::table_name(),
                    ))
                    .bind(account_id)
                    .bind(index_cat)
                    .bind(&index_data)
                    .fetch_one(&self.db)
                    .await?
                } else {
                    row.last_insert_id()
                }
            }
        };
        if addid > 0 {
            let del_status = AccountIndexStatus::Delete as i8;
            let tmp = Update::<_,AccountIndexModel>::new()
                .set(AccountIndexModel::STATUS, del_status)
                .set(AccountIndexModel::CHANGE_TIME, time)
                .execute(&mut *db, |qb| {
                    qb.push_where().field_eq("account_id", account_id).push_and().field_eq("index_cat", index_cat).push_and().field_ne("id", addid);
                })
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
    ) -> AccountResult<()> {
        if index_data.is_empty() {
            return Ok(());
        }
        let time = now_time()?;
        let index_cat = cat as u8;
        let status = AccountIndexStatus::Enable as i8;
        let tmp_data = index_data.iter().map(|e| e.to_string()).collect::<Vec<_>>();
        let mut batch = BatchInsert::<_,AccountIndexModel>::with_capacity(tmp_data.len());
        for t in tmp_data.iter() {
            batch = batch.push(
                Insert::<_,AccountIndexModel>::new()
                    .set(AccountIndexModel::INDEX_CAT, index_cat)
                    .set(AccountIndexModel::INDEX_DATA, t)
                    .set(AccountIndexModel::ACCOUNT_ID, account_id)
                    .set(AccountIndexModel::STATUS, status)
                    .set(AccountIndexModel::CHANGE_TIME, time),
            );
        }
        batch
            .execute_update(
                Update::<_,AccountIndexModel>::new()
                    .set(AccountIndexModel::STATUS, status)
                    .set(AccountIndexModel::CHANGE_TIME, time),
                OptionTxExecutor::new(transaction, &self.db),
            )
            .await?;
        Ok(())
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
        let res = Update::<_,AccountIndexModel>::new()
            .set(AccountIndexModel::STATUS, AccountIndexStatus::Delete as i8)
            .set(AccountIndexModel::CHANGE_TIME, time)
            .execute(OptionTxExecutor::new(transaction, &self.db), |qb| {
                qb.push_where().field_in_string("index_data", index_data);
                qb.push_and().field_eq("index_cat", index_cat);
                qb.push_and().field_eq("account_id", account_id);
            })
            .await?;
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
        let res = Update::<_,AccountIndexModel>::new()
            .set(AccountIndexModel::STATUS, AccountIndexStatus::Delete as i8)
            .set(AccountIndexModel::CHANGE_TIME, time)
            .execute(OptionTxExecutor::new(transaction, &self.db), |qb| {
                qb.push_where().field_eq("index_cat", index_cat).push_and().field_eq("account_id", account_id);
            })
            .await?;
        Ok(res.rows_affected())
    }
    pub async fn account_del(
        &self,
        account_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) -> AccountResult<u64> {
        let time = now_time()?;
        let res = Update::<_,AccountIndexModel>::new()
            .set(AccountIndexModel::STATUS, AccountIndexStatus::Delete as i8)
            .set(AccountIndexModel::CHANGE_TIME, time)
            .execute(OptionTxExecutor::new(transaction, &self.db), |qb| {
                qb.push_where().field_eq("account_id", account_id);
            })
            .await?;
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
        limit: &CursorPageParam<u64>,
    ) -> AccountResult<(Vec<AccountItem>, CursorPageData<u64>)> {
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
        let key_word = string_clear(key_word, StringClear::LikeKeyWord, None);
        let mut qb = QueryBuilder::<MySql>::new("");
        if key_word.is_empty() || param.is_empty() {
            qb.push(format!(
                "select distinct k.account_id,'' as cat_more FROM {} as k",
                AccountIndexModel::table_name(),
            ));
            qb.push_where().field_eq("k.status", AccountIndexStatus::Enable as i8);
            qb.push_and().field_eq("k.index_cat", AccountIndexCat::AccountStatus as i8);
            qb.push_and().field_in_string("k.index_data", &account_status_data);
            qb.push(" ");
        } else {
            let index_cat_data = param.iter().map(|e| *e as i8).collect::<Vec<_>>();
            qb.push(format!(
                "select distinct k.account_id,group_concat(k.index_cat,':',REPLACE(REPLACE(k.index_data,':',' '),',',' ')) as cat_more FROM {} as s inner join {} as k on s.account_id = k.account_id",
                AccountIndexModel::table_name(),
                AccountIndexModel::table_name(),
            ));
            qb.push_where().field_eq("s.status", AccountIndexStatus::Enable as i8);
            qb.push_and().field_eq("s.index_cat", AccountIndexCat::AccountStatus as i8);
            qb.push_and().field_in_string("s.index_data", &account_status_data);
            qb.push_and().field_eq("k.status", AccountIndexStatus::Enable as i8);
            qb.push_and().field_like("k.index_data", format!("{}%", key_word));
            if !index_cat_data.is_empty() {
                qb.push_and().field_in_copied("k.index_cat", &index_cat_data);
            }
            qb.push(" ");
        }
        let query_limit = limit.page_query("k.account_id");
        if query_limit.has_cursor() {
            qb.push_and();
            query_limit.push_where(&mut qb);
            qb.push(" ");
        }
        qb.push(" group by k.account_id HAVING cat_more IS NOT NULL ");
        query_limit.push_order_by(&mut qb);
        query_limit.push_limit(&mut qb);
        let mut res: Vec<(u64, String)> = qb.build_query_as::<(u64, String)>()
            .fetch_all(&self.db)
            .await?;
        let next = query_limit.finalize(&mut res, |row, cursor| row.0 == *cursor, |row| row.0);
        let out = res
            .into_iter()
            .map(|e| {
                let mut cats = Map::new();
                for item in e.1.split(',') {
                    let mut tmp = item.split(':');
                    if let Some(cat) = tmp.next()
                        && let Ok(cat) = cat.parse::<i8>()
                            && let Ok(cat) = AccountIndexCat::try_from(cat)
                                && let Some(val) = tmp.next() {
                                    cats.insert(cat, val.to_string());
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
