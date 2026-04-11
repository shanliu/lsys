use lsys_core::db::{
    CursorPageData, CursorPageParam, Insert, QueryBuilderExt, TableMeta, TotalParam, TotalRow,
    Update, WhereClause,
};
use lsys_core::utils::{STRING_CLEAR_FORMAT, StringClear, now_time, string_clear};

use sqlx::{MySql, Pool, QueryBuilder};

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
    fn history_where<'a, 'args>(
        &self,
        wb: &mut WhereClause<'a, 'args, MySql>,
        account_id: Option<u64>,
        login_account: Option<&str>,
        is_login: Option<i8>,
        login_type: Option<&str>,
        login_ip: Option<&str>,
    ) {
        if let Some(tmp) = account_id {
            wb.and().field_eq("account_id", tmp);
        }
        if let Some(tmp) = login_account {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(129));
            wb.and().field_eq("login_account", tmp);
        }
        if let Some(tmp) = is_login {
            wb.and().field_eq("is_login", tmp);
        }
        if let Some(tmp) = login_ip {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(47));
            wb.and().field_eq("login_ip", tmp);
        }
        if let Some(tmp) = login_type {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            wb.and().field_eq("login_type", tmp);
        }
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
        let query_limit = limit.page_query("id");
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select * from {}",
            AccountLoginModel::table_name()
        ));
        {
            let mut wb = WhereClause::new(&mut qb);
            self.history_where(
                &mut wb,
                account_id,
                login_account,
                is_login,
                login_type,
                login_ip,
            );
            if query_limit.has_cursor() {
                query_limit.push_where(wb.and());
            }
        }
        query_limit.push_order_by(&mut qb);
        query_limit.push_limit(&mut qb);

        let mut data = qb
            .build_query_as::<AccountLoginModel>()
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
        total_param: &TotalParam,
    ) -> AccountResult<TotalRow> {
        let query = total_param.total_count_query();
        let mut qb = if query.is_threshold_mode() {
            QueryBuilder::<MySql>::new(format!(
                "select count(*) as total from (select 1 from {}",
                AccountLoginModel::table_name()
            ))
        } else {
            QueryBuilder::<MySql>::new(format!(
                "select count(*) as total from {}",
                AccountLoginModel::table_name()
            ))
        };
        {
            let mut wb = WhereClause::new(&mut qb);
            self.history_where(
                &mut wb,
                account_id,
                login_account,
                is_login,
                login_type,
                login_ip,
            );
        }
        if query.is_threshold_mode() {
            query.push_limit(&mut qb);
            qb.push(") as t");
        }
        let count = qb
            .build_query_scalar()
            .fetch_one(&self.db)
            .await
            .unwrap_or(0i64) as u64;
        Ok(query.finalize(count))
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
        let login_res = Insert::<_, AccountLoginModel>::new()
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
        let ures = Update::<_, AccountLoginModel>::new()
            .set(AccountLoginModel::IS_LOGIN, is_login)
            .set(AccountLoginModel::ACCOUNT_ID, account_id)
            .set(AccountLoginModel::LOGIN_MSG, login_msg)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", login_id);
            })
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
