use lsys_core::db::{
    CursorPageData, CursorPageParam, Insert, SqlExpr, SqlQuote, SqlSuffix, TableMeta, Update,
};
use lsys_core::utils::{
    now_time, string_clear, StringClear, VecStringJoin, STRING_CLEAR_FORMAT,
};
use lsys_core::sql_format;

use sqlx::{MySql, Pool};

use tracing::error;

use crate::model::{AccountLoginModel, AccountLoginStatus};

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
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(129));
            where_sql.push(sql_format!("login_account={}", tmp))
        }
        if let Some(tmp) = is_login {
            where_sql.push(sql_format!("is_login={}", tmp))
        }
        if let Some(tmp) = login_ip {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(47));
            where_sql.push(sql_format!("login_ip={}", tmp))
        }
        if let Some(tmp) = login_type {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
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
        limit: &CursorPageParam<u64>,
    ) -> AccountResult<(Vec<AccountLoginModel>, CursorPageData<u64>)> {
        let sqlwhere =
            self.history_where(account_id, login_account, is_login, login_type, login_ip);

        let query_limit = limit.page_query("id");
        let where_str = sqlwhere.join(" and ");
        let suff_sql = query_limit.build_query_sql(if sqlwhere.is_empty() {
            None
        } else {
            Some(&where_str)
        });

        let mut data = sqlx::query_as::<_, AccountLoginModel>(&sql_format!(
            "select * from {} {}",
            AccountLoginModel::table_name(),
            SqlExpr(suff_sql)
        ))
        .fetch_all(&self.db)
        .await?;

        let next = query_limit.finalize(&mut data, |c, d| *d == c.id, |c| c.id);
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
        let login_account = string_clear(
            login_account,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(128),
        );
        let login_type = string_clear(
            login_type,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(32),
        );
        let login_ip = string_clear(login_ip, StringClear::Option(STRING_CLEAR_FORMAT), Some(46));
        let login_city = string_clear(
            login_city,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(100),
        );
        let login_res = Insert::<_,AccountLoginModel>::new()
            .set(AccountLoginModel::LOGIN_TYPE, login_type)
            .set(AccountLoginModel::LOGIN_ACCOUNT, login_account)
            .set(AccountLoginModel::LOGIN_IP, login_ip)
            .set(AccountLoginModel::ACCOUNT_ID, 0_u64)
            .set(AccountLoginModel::IS_LOGIN, 0_i8)
            .set(AccountLoginModel::LOGIN_CITY, login_city)
            .set(AccountLoginModel::ADD_TIME, time)
            .execute(&self.db)
            .await?;
        Ok(login_res.last_insert_id())
    }
    /// 设置用户信息
    pub async fn finish_history(
        &self,
        login_id: u64,
        is_login: AccountLoginStatus,
        account_id: u64,
        login_msg: impl ToString,
    ) -> AccountResult<()> {
        let login_msg = login_msg.to_string();
        let is_login = is_login as i8;
        let ures = Update::<_,AccountLoginModel>::new()
            .set(AccountLoginModel::IS_LOGIN, is_login)
            .set(AccountLoginModel::ACCOUNT_ID, account_id)
            .set(AccountLoginModel::LOGIN_MSG, login_msg)
            .execute(SqlSuffix::Where(&sql_format!("id={}", login_id)), &self.db)
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
