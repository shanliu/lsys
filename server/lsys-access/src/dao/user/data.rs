use std::collections::HashMap;

use crate::model::{SessionModel, SessionStatus};
use crate::{dao::AccessResult, model::UserModel};

use super::AccessUser;
use lsys_core::db::{ModelTableName, SqlExpr, SqlQuote};
use lsys_core::{now_time, sql_format, LimitParam};
use serde::Serialize;
impl AccessUser {
    //通过ID获取用户
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        UserModel,
        AccessResult<UserModel>,
        id,
        "id = {id} "
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        UserModel,
        AccessResult<HashMap<u64, UserModel>>,
        id,
        ids,
        "id in ({ids}) "
    );
}
impl AccessUser {
    //通过登录数据查询用户
    pub async fn find_by_data(&self, app_id: u64, data: &str) -> AccessResult<UserModel> {
        let data = sqlx::query_as::<_, UserModel>(&sql_format!(
            "select * from {} where app_id={} and user_data={}",
            UserModel::table_name(),
            app_id,
            data
        ))
        .fetch_one(&self.db)
        .await?;
        Ok(data)
    }
}

pub struct UserDataParam<'t> {
    pub app_id: Option<u64>,
    pub user_data: Option<&'t str>,
    pub user_account: Option<&'t str>,
    pub user_any: Option<&'t str>,
}

impl AccessUser {
    //通过登录数据查询用户
    fn user_data_where(&self, param: &UserDataParam<'_>) -> Option<Vec<String>> {
        let mut sql_vec = vec![];
        if let Some(ref tmp) = param.app_id {
            sql_vec.push(sql_format!("app_id = {}", tmp));
        };
        if let Some(tmp) = param.user_any {
            sql_vec.push(sql_format!(
                " ( user_data = {} or user_account = {} ) ",
                tmp,
                tmp
            ));
        }
        if let Some(tmp) = param.user_data {
            sql_vec.push(sql_format!("user_data = {}", tmp));
        }
        if let Some(ref tmp) = param.user_account {
            sql_vec.push(sql_format!("user_account = {}", tmp));
        }
        Some(sql_vec)
    }
    //用户数据
    pub async fn user_data(
        &self,
        param: &UserDataParam<'_>,
        limit: Option<&LimitParam>,
    ) -> AccessResult<(Vec<UserModel>, Option<u64>)> {
        let where_sql = match self.user_data_where(param) {
            Some(sql) => sql,
            None => return Ok((vec![], None)),
        };
        let where_sql = if let Some(page) = limit {
            let page_where = page.where_sql(
                "id",
                if where_sql.is_empty() {
                    None
                } else {
                    Some("and")
                },
            );
            format!(
                "{} {} {} order by {} {} ",
                if !where_sql.is_empty() || !page_where.is_empty() {
                    "where "
                } else {
                    ""
                },
                where_sql.join(" and "),
                page_where,
                page.order_sql("id"),
                page.limit_sql(),
            )
        } else {
            format!(
                "{} {}  order by id desc",
                if where_sql.is_empty() { "where " } else { "" },
                where_sql.join(" and ")
            )
        };
        let mut out_data = sqlx::query_as::<_, UserModel>(&sql_format!(
            "select * from {} {}",
            UserModel::table_name(),
            SqlExpr(where_sql)
        ))
        .fetch_all(&self.db)
        .await?;
        let next = limit
            .as_ref()
            .map(|page| page.tidy(&mut out_data))
            .unwrap_or_default()
            .map(|e| e.id);
        Ok((out_data, next))
    }
    pub async fn user_count(&self, param: &UserDataParam<'_>) -> AccessResult<i64> {
        let where_sql = match self.user_data_where(param) {
            Some(sql) => sql,
            None => return Ok(0),
        };
        let out_total = sqlx::query_scalar::<_, i64>(&sql_format!(
            "select count(*) as total from {} {}",
            UserModel::table_name(),
            SqlExpr(format!(
                "{} {}",
                if where_sql.is_empty() { "where " } else { "" },
                where_sql.join(" and ")
            ))
        ))
        .fetch_one(&self.db)
        .await?;
        Ok(out_total)
    }
}

pub struct SessionDataParam {
    pub app_id: Option<u64>,
    pub oauth_app_id: Option<u64>,
    pub user_id: Option<u64>,
    pub is_enable: Option<bool>,
}

#[derive(Serialize, Debug)]
pub struct SessionDataRecord {
    pub token_data: String,
    pub user_id: u64,
    pub app_id: u64,
    pub oauth_app_id: u64,
    pub login_type: String,
    pub login_ip: String,
    pub device_id: String,
    pub device_name: String,
    pub status: i8,
    pub add_time: u64,
    pub expire_time: u64,
    pub logout_time: u64,
}

impl AccessUser {
    //通过登录数据查询用户
    fn session_data_where(&self, param: &SessionDataParam) -> Option<Vec<String>> {
        let mut sql_vec = vec![];
        if let Some(ref tmp) = param.app_id {
            sql_vec.push(sql_format!("user_app_id = {}", tmp));
        };
        if let Some(ref tmp) = param.oauth_app_id {
            sql_vec.push(sql_format!("oauth_app_id = {}", tmp));
        };
        if let Some(ref tmp) = param.user_id {
            sql_vec.push(sql_format!("user_id = {}", tmp));
        };
        if let Some(ref tmp) = param.is_enable {
            let ntime = now_time().unwrap_or_default();
            if *tmp {
                sql_vec.push(sql_format!(
                    "status = {} and expire_time>{}",
                    SessionStatus::Enable as i8,
                    ntime
                ));
            } else {
                sql_vec.push(sql_format!(
                    "(status != {} or expire_time<={})",
                    SessionStatus::Enable as i8,
                    ntime
                ));
            }
        };
        Some(sql_vec)
    }
    // 用户登录数据
    pub async fn session_data(
        &self,
        param: &SessionDataParam,
        limit: Option<&LimitParam>,
    ) -> AccessResult<(Vec<SessionDataRecord>, Option<u64>)> {
        let where_sql = match self.session_data_where(param) {
            Some(sql) => sql,
            None => return Ok((vec![], None)),
        };
        let where_sql = if let Some(page) = limit {
            let page_where = page.where_sql(
                "id",
                if where_sql.is_empty() {
                    None
                } else {
                    Some("and")
                },
            );
            format!(
                "{} {} {} order by {} {} ",
                if !where_sql.is_empty() || !page_where.is_empty() {
                    "where "
                } else {
                    ""
                },
                where_sql.join(" and "),
                page_where,
                page.order_sql("id"),
                page.limit_sql(),
            )
        } else {
            format!(
                "{} {}  order by id desc",
                if where_sql.is_empty() { "where " } else { "" },
                where_sql.join(" and ")
            )
        };
        let mut out_data = sqlx::query_as::<_, SessionModel>(&sql_format!(
            "select * from {} {}",
            SessionModel::table_name(),
            SqlExpr(where_sql)
        ))
        .fetch_all(&self.db)
        .await?;

        let next = limit
            .as_ref()
            .map(|page| page.tidy(&mut out_data))
            .unwrap_or_default()
            .map(|e| e.id);
        let ntime = now_time().unwrap_or_default();
        Ok((
            out_data
                .into_iter()
                .map(|e| SessionDataRecord {
                    token_data: e.token_data,
                    user_id: e.user_id,
                    app_id: e.user_app_id,
                    oauth_app_id: e.oauth_app_id,
                    login_type: e.login_type,
                    login_ip: e.login_ip,
                    device_id: e.device_id,
                    device_name: e.device_name,
                    status: if SessionStatus::Enable.eq(e.status) {
                        if e.expire_time > ntime {
                            SessionStatus::Enable as i8
                        } else {
                            SessionStatus::Delete as i8
                        }
                    } else {
                        SessionStatus::Delete as i8
                    },
                    add_time: e.add_time,
                    expire_time: e.expire_time,
                    logout_time: e.logout_time,
                })
                .collect::<Vec<_>>(),
            next,
        ))
    }
    pub async fn session_count(&self, param: &SessionDataParam) -> AccessResult<i64> {
        let where_sql = match self.session_data_where(param) {
            Some(sql) => sql,
            None => return Ok(0),
        };
        let out_total = sqlx::query_scalar::<_, i64>(&sql_format!(
            "select count(*) as total from {} {}",
            SessionModel::table_name(),
            SqlExpr(format!(
                "{} {}",
                if where_sql.is_empty() { "where " } else { "" },
                where_sql.join(" and ")
            ))
        ))
        .fetch_one(&self.db)
        .await?;
        Ok(out_total)
    }
}
