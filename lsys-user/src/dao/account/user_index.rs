use crate::model::{UserIndexCat, UserIndexModel, UserIndexModelRef, UserIndexStatus};
use lsys_core::now_time;
use sqlx::{MySql, Pool, Transaction};
use sqlx_model::{executor_option, model_option_set, sql_format, Insert, SqlQuote, Update};

use super::UserAccountResult;

pub struct UserIndex {
    db: Pool<MySql>,
}

impl UserIndex {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
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
                status:status,
                index_data:t,
                user_id:user_id,
                add_time:time,
            }));
        }
        let res = executor_option!(
            {
                Insert::<sqlx::MySql, UserIndexModel, _>::new_vec(vdata)
                    .execute(db)
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
            delete_time:time,
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
    pub async fn user_del<'t>(
        &self,
        user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(UserIndexModelRef,{
            status:UserIndexStatus::Delete as i8,
            delete_time:time,
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
}
