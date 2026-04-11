use std::collections::HashMap;

use crate::model::{SessionModel, SessionStatus};
use crate::{dao::AccessResult, model::UserModel};

use super::AccessUser;
use lsys_core::db::{
    CursorPageData, CursorPageParam, QueryBuilderExt, TableMeta, TotalParam, TotalRow, WhereClause,
};
use lsys_core::utils::{STRING_CLEAR_FORMAT, StringClear, now_time, string_clear};
use lsys_core::valid_param::{ValidParam, ValidParamCheck, ValidPattern, ValidStrlen};
use lsys_core::{db::utils::FetchField, valid_key};
use serde::Serialize;
use sqlx::{MySql, QueryBuilder};
impl AccessUser {
    //通过ID获取用户
    pub async fn find_by_id(&self, id: &u64) -> AccessResult<UserModel> {
        Ok(
            lsys_core::db::utils::Fetch::<MySql, UserModel>::one(&self.db, |qb| {
                qb.field_eq("id", *id);
            })
            .await?,
        )
    }
    pub async fn find_by_ids(&self, ids: &[u64]) -> AccessResult<HashMap<u64, UserModel>> {
        Ok(lsys_core::db::utils::Fetch::<MySql, UserModel>::map(
            &self.db,
            |qb| {
                qb.field_in_copied("id", ids);
            },
            |v| v.id,
        )
        .await?)
    }
}
impl AccessUser {
    async fn find_by_data_param_valid(&self, user_data: &str) -> AccessResult<()> {
        let user_data = user_data.to_string();
        let user_data_max = FetchField::new(&self.db)
            .string_max::<UserModel>(&UserModel::USER_DATA)
            .await
            .len_or(32);

        ValidParam::default()
            .add(
                valid_key!("user_data"),
                &user_data,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, user_data_max)),
            )
            .check()?;
        Ok(())
    }
    //通过登录数据查询用户
    pub async fn find_by_data(&self, app_id: u64, user_data: &str) -> AccessResult<UserModel> {
        self.find_by_data_param_valid(user_data).await?;
        Ok(sqlx::query_as::<_, UserModel>(&format!(
            "select * from {} where app_id=? and user_data=?",
            UserModel::table_name(),
        ))
        .bind(app_id)
        .bind(user_data)
        .fetch_one(&self.db)
        .await?)
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
    fn user_data_where<'a, 'args>(
        &self,
        wb: &mut WhereClause<'a, 'args, MySql>,
        param: &UserDataParam<'_>,
    ) -> bool {
        if let Some(ref tmp) = param.app_id {
            wb.and().field_eq("app_id", *tmp);
        };
        if let Some(tmp) = param.user_any {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(129));
            if tmp.is_empty() {
                return false;
            }
            let qb = wb.and();
            qb.push("(");
            qb.field_eq("user_data", tmp.clone());
            qb.push_or().field_eq("user_account", tmp);
            qb.push(")");
        }
        if let Some(tmp) = param.user_data {
            let tmp = string_clear(tmp, StringClear::Ident, Some(33));
            if tmp.is_empty() {
                return false;
            }
            wb.and().field_eq("user_data", tmp);
        }
        if let Some(ref tmp) = param.user_account {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(129));
            if tmp.is_empty() {
                return false;
            }
            wb.and().field_eq("user_account", tmp);
        }
        true
    }
    //用户数据
    pub async fn user_data(
        &self,
        param: &UserDataParam<'_>,
        limit: &CursorPageParam<u64>,
    ) -> AccessResult<(Vec<UserModel>, CursorPageData<u64>)> {
        let query_limit = limit.page_query("id");
        let mut qb =
            QueryBuilder::<MySql>::new(format!("select * from {}", UserModel::table_name()));
        {
            let mut wb = WhereClause::new(&mut qb);
            if !self.user_data_where(&mut wb, param) {
                return Ok((vec![], CursorPageData::default()));
            }
            if query_limit.has_cursor() {
                query_limit.push_where(wb.and());
            }
        }
        query_limit.push_order_by(&mut qb);
        query_limit.push_limit(&mut qb);
        let mut out_data = qb.build_query_as::<UserModel>().fetch_all(&self.db).await?;
        let next = query_limit.finalize(&mut out_data, |c, d| *d == c.id, |c| c.id);
        Ok((out_data, next))
    }
    pub async fn user_count(
        &self,
        param: &UserDataParam<'_>,
        total_param: &TotalParam,
    ) -> AccessResult<TotalRow> {
        let query = total_param.total_count_query();
        let out_total = if query.is_threshold_mode() {
            let mut qb = QueryBuilder::<MySql>::new(format!(
                "select count(*) as total from (select 1 from {}",
                UserModel::table_name()
            ));
            {
                let mut wb = WhereClause::new(&mut qb);
                if !self.user_data_where(&mut wb, param) {
                    return Ok(TotalRow::Exact(0));
                }
            }
            query.push_limit(&mut qb);
            qb.push(") as t");
            qb.build_query_scalar::<i64>().fetch_one(&self.db).await? as u64
        } else {
            let mut qb = QueryBuilder::<MySql>::new(format!(
                "select count(*) as total from {}",
                UserModel::table_name()
            ));
            {
                let mut wb = WhereClause::new(&mut qb);
                if !self.user_data_where(&mut wb, param) {
                    return Ok(TotalRow::Exact(0));
                }
            }
            qb.build_query_scalar::<i64>().fetch_one(&self.db).await? as u64
        };
        Ok(query.finalize(out_total))
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
    pub id: u64,
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
    fn session_data_where<'a, 'args>(
        &self,
        wb: &mut WhereClause<'a, 'args, MySql>,
        param: &SessionDataParam,
    ) {
        if let Some(ref tmp) = param.app_id {
            wb.and().field_eq("user_app_id", *tmp);
        };
        if let Some(ref tmp) = param.oauth_app_id {
            wb.and().field_eq("oauth_app_id", *tmp);
        };
        if let Some(ref tmp) = param.user_id {
            wb.and().field_eq("user_id", *tmp);
        };
        if let Some(ref tmp) = param.is_enable {
            let ntime = now_time().unwrap_or_default();
            if *tmp {
                let qb = wb.and();
                qb.field_eq("status", SessionStatus::Enable as i8);
                qb.push_and().field_gt("expire_time", ntime);
            } else {
                let qb = wb.and();
                qb.push("(");
                qb.field_ne("status", SessionStatus::Enable as i8);
                qb.push_or().field_lte("expire_time", ntime);
                qb.push(")");
            }
        };
    }
    // 用户登录数据
    pub async fn session_data(
        &self,
        param: &SessionDataParam,
        limit: &CursorPageParam<u64>,
    ) -> AccessResult<(Vec<SessionDataRecord>, CursorPageData<u64>)> {
        let query_limit = limit.page_query("id");
        let mut qb =
            QueryBuilder::<MySql>::new(format!("select * from {}", SessionModel::table_name()));
        {
            let mut wb = WhereClause::new(&mut qb);
            self.session_data_where(&mut wb, param);
            if query_limit.has_cursor() {
                query_limit.push_where(wb.and());
            }
        }
        query_limit.push_order_by(&mut qb);
        query_limit.push_limit(&mut qb);
        let mut out_data = qb
            .build_query_as::<SessionModel>()
            .fetch_all(&self.db)
            .await?;

        let next = query_limit.finalize(&mut out_data, |c, d| *d == c.id, |c| c.id);
        let ntime = now_time().unwrap_or_default();
        Ok((
            out_data
                .into_iter()
                .map(|e| SessionDataRecord {
                    id: e.id,
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
    pub async fn session_count(
        &self,
        param: &SessionDataParam,
        total_param: &TotalParam,
    ) -> AccessResult<TotalRow> {
        let query = total_param.total_count_query();
        let out_total = if query.is_threshold_mode() {
            let mut qb = QueryBuilder::<MySql>::new(format!(
                "select count(*) as total from (select 1 from {}",
                SessionModel::table_name()
            ));
            {
                let mut wb = WhereClause::new(&mut qb);
                self.session_data_where(&mut wb, param);
            }
            query.push_limit(&mut qb);
            qb.push(") as t");
            qb.build_query_scalar::<i64>().fetch_one(&self.db).await? as u64
        } else {
            let mut qb = QueryBuilder::<MySql>::new(format!(
                "select count(*) as total from {}",
                SessionModel::table_name()
            ));
            {
                let mut wb = WhereClause::new(&mut qb);
                self.session_data_where(&mut wb, param);
            }
            qb.build_query_scalar::<i64>().fetch_one(&self.db).await? as u64
        };
        Ok(query.finalize(out_total))
    }
}
