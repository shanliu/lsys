use lsys_core::db::{Insert, ModelTableName, SqlExpr, SqlQuote, Update, WhereOption};
use lsys_core::{model_option_set, now_time, sql_format, LimitParam, VecStringJoin};

use sqlx::{MySql, Pool};

use tracing::error;

use crate::model::{AccountLoginModel, AccountLoginModelRef};

use super::AccountResult;

pub struct AccountLoginHistory {
    db: Pool<MySql>,
}

impl AccountLoginHistory {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }
    fn history_where(
        &self,
        account_id: Option<u64>,
        login_account: Option<&str>,
        is_login: Option<i8>,
        login_type: Option<&str>,
        login_ip: Option<&str>,
    ) -> Vec<String> {
        let mut where_sql = vec![];
        if let Some(tmp) = account_id {
            where_sql.push(sql_format!("account_id={}", tmp))
        }
        if let Some(tmp) = login_account {
            if !tmp.is_empty() {
                where_sql.push(sql_format!("login_account={}", tmp))
            }
        }
        if let Some(tmp) = is_login {
            where_sql.push(sql_format!("is_login={}", tmp))
        }
        if let Some(tmp) = login_ip {
            where_sql.push(sql_format!("login_ip={}", tmp))
        }
        if let Some(tmp) = login_type {
            where_sql.push(sql_format!("login_type={}", tmp))
        }
        where_sql
    }
    /// 登陆历史
    pub async fn history_data(
        &self,
        account_id: Option<u64>,
        login_account: Option<&str>,
        is_login: Option<i8>,
        login_type: Option<&str>,
        login_ip: Option<&str>,
        limit: Option<&LimitParam>,
    ) -> AccountResult<(Vec<AccountLoginModel>, Option<u64>)> {
        let sqlwhere =
            self.history_where(account_id, login_account, is_login, login_type, login_ip);

        let tmp = if let Some(page) = limit {
            if sqlwhere.is_empty() {
                format!(
                    " {} order by {} {} ",
                    page.where_sql("id", None),
                    page.order_sql("id"),
                    page.limit_sql(),
                )
            } else {
                format!(
                    "{} {} order by {} {} ",
                    sqlwhere.join(" and "),
                    page.where_sql("id", Some("and")),
                    page.order_sql("id"),
                    page.limit_sql(),
                )
            }
        } else {
            format!("{}  order by id desc", sqlwhere.join(" and "))
        };

        let mut data = sqlx::query_as::<_, AccountLoginModel>(&sql_format!(
            "select * from {} {}",
            AccountLoginModel::table_name(),
            if !sqlwhere.is_empty()
                || limit
                    .as_ref()
                    .map(|e| e.pos())
                    .unwrap_or_default()
                    .is_some()
            {
                SqlExpr(format!(" where {}", tmp))
            } else {
                SqlExpr(tmp)
            }
        ))
        .fetch_all(&self.db)
        .await?;

        let next = limit
            .as_ref()
            .map(|page| page.tidy(&mut data))
            .unwrap_or_default()
            .map(|e| e.id);
        Ok((data, next))
    }
    /// 登陆历史数量
    pub async fn history_count(
        &self,
        account_id: Option<u64>,
        login_account: Option<&str>,
        is_login: Option<i8>,
        login_type: Option<&str>,
        login_ip: Option<&str>,
    ) -> AccountResult<i64> {
        let where_sql =
            self.history_where(account_id, login_account, is_login, login_type, login_ip);

        let wsql = if where_sql.is_empty() {
            "".to_string()
        } else {
            format!("where {}", where_sql.string_join(" and "))
        };
        let sql = format!(
            "select count(*) as total from {} {}",
            AccountLoginModel::table_name(),
            wsql,
        );
        let res = sqlx::query_scalar::<_, i64>(sql.as_str())
            .fetch_one(&self.db)
            .await?;
        Ok(res)
    }
    /// 设置用户信息
    pub async fn create_history(
        &self,
        login_account: &str,
        login_type: &str,
        login_ip: &str,
        login_city: &str,
    ) -> AccountResult<u64> {
        let time = now_time()?;
        let login_account = login_account.to_string();
        let login_type = login_type.to_string();
        let login_ip = login_ip.to_string();
        let login_city = login_city.to_string();
        let new_data = model_option_set!(AccountLoginModelRef,{
            login_type:login_type,
            login_account:login_account,
            login_ip:login_ip,
            account_id: 0,
            is_login: 0,
            login_city:login_city,
            add_time: time,
        });
        let login_res = Insert::<AccountLoginModel, _>::new(new_data)
            .execute(&self.db)
            .await?;
        Ok(login_res.last_insert_id())
    }
    /// 设置用户信息
    pub async fn finish_history(
        &self,
        login_id: u64,
        is_login: i8,
        account_id: u64,
        login_msg: impl ToString,
    ) -> AccountResult<()> {
        let login_msg = login_msg.to_string();
        let change = lsys_core::model_option_set!(AccountLoginModelRef,{
            is_login:is_login,
            account_id:account_id,
            login_msg:login_msg,

        });
        let ures = Update::< AccountLoginModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("id={}", login_id)),
                &self.db,
            )
            .await;
        if let Err(err) = ures {
            error!(
                "update login success status fail {} in login id: {}",
                err.to_string(),
                login_id
            );
        }
        Ok(())
    }
}
