use lsys_core::{now_time, PageParam, VecStringJoin};
use sqlx::{MySql, Pool};
use sqlx_model::{model_option_set, sql_format, Insert, ModelTableName, Select, SqlQuote, Update};

use tracing::error;

use crate::model::{UserLoginModel, UserLoginModelRef};

use super::UserAccountResult;

pub struct UserLogin {
    db: Pool<MySql>,
}

impl UserLogin {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }
    /// 登陆历史
    pub async fn history_data(
        &self,
        user_id: Option<u64>,
        login_account: Option<String>,
        is_login: Option<i8>,
        login_type: Option<String>,
        page: &Option<PageParam>,
    ) -> UserAccountResult<Vec<UserLoginModel>> {
        let mut where_sql = vec![];
        if user_id.is_some() {
            where_sql.push("user_id=?")
        }
        if let Some(ref tmp) = login_account {
            if !tmp.is_empty() {
                where_sql.push("login_account=?")
            }
        }
        if is_login.is_some() {
            where_sql.push("is_login=?")
        }
        if login_type.is_some() {
            where_sql.push("login_type=?")
        }
        let mut sql = where_sql.string_join(" and ") + "  order by id desc ";
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }
        let user_res = Select::type_new::<UserLoginModel>()
            .fetch_all_by_where_call::<UserLoginModel, _, _>(
                &sql,
                |mut res, _| {
                    if let Some(tmp) = user_id {
                        res = res.bind(tmp);
                    }
                    if let Some(tmp) = login_account {
                        if !tmp.is_empty() {
                            res = res.bind(tmp);
                        }
                    }
                    if let Some(tmp) = is_login {
                        res = res.bind(tmp);
                    }
                    if let Some(tmp) = login_type {
                        res = res.bind(tmp);
                    }
                    res
                },
                &self.db.clone(),
            )
            .await?;
        Ok(user_res)
    }
    /// 登陆历史数量
    pub async fn history_count(
        &self,
        user_id: Option<u64>,
        login_account: Option<String>,
        is_login: Option<i8>,
        login_type: Option<String>,
    ) -> UserAccountResult<i64> {
        let mut where_sql = vec![];
        if let Some(tmp) = user_id {
            where_sql.push(sql_format!("user_id={}", tmp));
        }
        if let Some(tmp) = login_account {
            if !tmp.is_empty() {
                where_sql.push(sql_format!("login_account={}", tmp));
            }
        }
        if let Some(tmp) = is_login {
            where_sql.push(sql_format!("is_login={}", tmp));
        }
        if let Some(tmp) = login_type {
            where_sql.push(sql_format!("login_type={}", tmp));
        }
        let wsql = if where_sql.is_empty() {
            "".to_string()
        } else {
            format!("where {}", where_sql.string_join(" and "))
        };
        let sql = format!(
            "select count(*) as total from {} {}",
            UserLoginModel::table_name(),
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
        login_account: String,
        login_type: String,
        login_ip: String,
        login_city: String,
    ) -> UserAccountResult<u64> {
        let time = now_time()?;
        let new_data = model_option_set!(UserLoginModelRef,{
            login_type:login_type,
            login_account:login_account,
            login_ip:login_ip,
            user_id: 0,
            is_login: 0,
            login_city:login_city,
            add_time: time,
        });
        let login_res = Insert::<sqlx::MySql, UserLoginModel, _>::new(new_data)
            .execute(&self.db)
            .await?;
        Ok(login_res.last_insert_id())
    }
    /// 设置用户信息
    pub async fn finish_history(
        &self,
        login_id: u64,
        is_login: i8,
        user_id: u64,
        login_msg: String,
        login_token: String,
    ) -> UserAccountResult<()> {
        let change = sqlx_model::model_option_set!(UserLoginModelRef,{
            is_login:is_login,
            user_id:user_id,
            login_msg:login_msg,
            login_token:login_token,
        });
        let ures = Update::<sqlx::MySql, UserLoginModel, _>::new(change)
            .execute_by_scalar_pk(login_id, &self.db)
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
