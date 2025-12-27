use super::res_type::ResTypeParam;
use super::RbacRes;
use crate::dao::result::RbacResult;
use crate::model::{
    RbacOpModel, RbacOpResModel, RbacOpResStatus, RbacOpStatus, RbacPermModel, RbacPermStatus,
    RbacResModel, RbacResStatus,
};
use lsys_core::db::{ModelTableName, SqlExpr, SqlQuote};
use lsys_core::{impl_dao_fetch_one_by_one, PageParam, StringClear, STRING_CLEAR_FORMAT};
use lsys_core::{sql_format, string_clear};
use serde::Serialize;
use sqlx::Row;
use std::collections::{HashMap, HashSet};
use std::vec;
//RBAC中资源相关实现

//资源的数据获取

impl RbacRes {
    impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        RbacResModel,
        RbacResult<RbacResModel>,
        id,
        "id={id} and status = {status}",
        status = RbacResStatus::Enable
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        RbacResModel,
        RbacResult<HashMap<u64, RbacResModel>>,
        id,
        ids,
        "id in ({ids}) and  status = {status}",
        status = RbacResStatus::Enable
    );
}

pub struct ResDataParam<'t> {
    pub user_id: Option<u64>,
    pub app_id: Option<u64>,
    pub res_type: Option<&'t str>,
    pub res_data: Option<&'t str>,
    pub res_name: Option<&'t str>,
    pub ids: Option<&'t [u64]>,
}

//资源管理
impl RbacRes {
    fn res_sql(&self, filed: &str, res_param: &ResDataParam<'_>) -> Option<String> {
        let mut sql = sql_format!(
            "select {} from {} where status={}",
            SqlExpr(filed),
            RbacResModel::table_name(),
            RbacResStatus::Enable,
        );
        if let Some(val) = res_param.app_id {
            sql += sql_format!(" and app_id = {}", val).as_str();
        }
        if let Some(val) = res_param.user_id {
            sql += sql_format!(" and user_id = {}", val).as_str();
        }
        if let Some(val) = res_param.res_type {
            let val = string_clear(val, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            if val.is_empty() {
                return None;
            }
            sql += sql_format!(" and res_type = {}", val).as_str();
        }
        if let Some(val) = res_param.res_data {
            let val = string_clear(val, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            sql += sql_format!(" and res_data = {}", val).as_str();
        }
        if let Some(val) = res_param.res_name {
            let val = string_clear(val, StringClear::LikeKeyWord, None);
            sql += sql_format!(" and res_name like {}", format!("%{}%", val)).as_str();
        }
        if let Some(rid) = res_param.ids {
            if rid.is_empty() {
                return None;
            } else {
                sql += &sql_format!(" and id in ({})", rid);
            }
        }
        Some(sql)
    }
    /// 获取指定条件的角色数量
    pub async fn res_count(&self, res_param: &ResDataParam<'_>) -> RbacResult<i64> {
        match self.res_sql("count(*) as total", res_param) {
            Some(sql) => {
                let query = sqlx::query_scalar::<_, i64>(&sql);
                let res = query.fetch_one(&self.db).await?;
                Ok(res)
            }
            None => Ok(0),
        }
    }
}

#[derive(Default)]
pub struct ResDataAttrParam {
    //关联的操作数量
    pub op_count: bool,
    //关联授权数量
    pub perm_count: bool,
}
#[derive(Default)]
pub struct RbacResInfoData {
    pub op_count: i64,
    pub perm_count: i64,
}
impl RbacRes {
    /// 获取资源关联的授权数量
    pub async fn res_perm_count(&self, res_ids: &[u64]) -> RbacResult<Vec<(u64, i64)>> {
        if res_ids.is_empty() {
            return Ok(vec![]);
        }
        let sql = sql_format!(
                            "select res_id,count(*) as total from {} where res_id in ({}) and status={} group by res_id",
                            RbacPermModel::table_name(),
                            res_ids,
                            RbacPermStatus::Enable
                        );
        let perm_counts = sqlx::query(&sql)
            .try_map(|row: sqlx::mysql::MySqlRow| {
                let res_id = row.try_get::<u64, &str>("res_id").unwrap_or_default();
                let total = row.try_get::<i64, &str>("total").unwrap_or_default();
                Ok((res_id, total))
            })
            .fetch_all(&self.db)
            .await?;
        Ok(perm_counts)
    }

    /// 获取指定用户和ID的列表
    pub async fn res_info(
        &self,
        res_param: &ResDataParam<'_>,
        res_attr: &ResDataAttrParam,
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<(RbacResModel, RbacResInfoData)>> {
        let res = self.res_data(res_param, page).await?;
        let mut op_count_map: HashMap<u64, i64> = HashMap::new();
        if res_attr.op_count && !res.is_empty() {
            let mut res_sql = Vec::with_capacity(res.len());
            let mut uniq_key = HashSet::new();
            for e in &res {
                let uniq_id = format!("{}_{}_{}", e.user_id, e.app_id, e.res_type);
                if uniq_key.contains(&uniq_id) {
                    continue;
                }
                uniq_key.insert(uniq_id);
                res_sql.push(sql_format!(
                    "select res_type,user_id,app_id,count(*) as total from {} where 
                                    status={}  and res_type = {} and
                                    user_id = {} and app_id = {}
                                    group by res_type",
                    RbacOpResModel::table_name(),
                    RbacOpResStatus::Enable,
                    e.res_type,
                    e.user_id,
                    e.app_id,
                ));
            }
            let sql = res_sql.join(" union all ");
            let op_counts = sqlx::query(&sql)
                .try_map(|row: sqlx::mysql::MySqlRow| {
                    let res_type = row.try_get::<String, &str>("res_type").unwrap_or_default();
                    let user_id = row.try_get::<u64, &str>("user_id").unwrap_or_default();
                    let app_id = row.try_get::<u64, &str>("app_id").unwrap_or_default();
                    let total = row.try_get::<i64, &str>("total").unwrap_or_default();
                    Ok((res_type, user_id, app_id, total))
                })
                .fetch_all(&self.db)
                .await?;
            for e in &res {
                let mut set_total = 0;
                for (res_type, user_id, app_id, total) in op_counts.iter() {
                    if e.res_type == *res_type && e.user_id == *user_id && e.app_id == *app_id {
                        set_total = *total;

                        break;
                    }
                }
                op_count_map.insert(e.id, set_total);
            }
        }
        let mut perm_count_map: HashMap<u64, i64> = HashMap::new();
        if res_attr.perm_count && !res.is_empty() {
            let perm_counts = self
                .res_perm_count(&res.iter().map(|e| e.id).collect::<Vec<_>>())
                .await?;
            for (res_id, total) in perm_counts {
                perm_count_map.insert(res_id, total);
            }
        }
        Ok(res
            .into_iter()
            .map(|e| {
                let info = RbacResInfoData {
                    op_count: *op_count_map.get(&e.id).unwrap_or(&0),
                    perm_count: *perm_count_map.get(&e.id).unwrap_or(&0),
                };
                (e, info)
            })
            .collect::<Vec<_>>())
    }
    pub async fn res_data(
        &self,
        res_param: &ResDataParam<'_>,
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<RbacResModel>> {
        match self.res_sql("*", res_param) {
            Some(mut sql) => {
                if let Some(pdat) = page {
                    sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
                }
                let res = sqlx::query_as::<_, RbacResModel>(&sql)
                    .fetch_all(&self.db)
                    .await?;
                Ok(res)
            }
            None => Ok(vec![]),
        }
    }
}

pub struct ResTypeListParam<'t> {
    pub user_id: Option<u64>,
    pub app_id: Option<u64>,
    pub res_type: Option<&'t str>,
}

#[derive(Serialize)]
pub struct ResTypeListRecord {
    pub user_id: u64,
    pub app_id: u64,
    pub res_type: String,
    pub res_total: u64,
}

impl RbacRes {
    fn res_type_sql_where(&self, res_param: &ResTypeListParam<'_>) -> Option<(String, String)> {
        let mut where_sql = vec![sql_format!("status={}", RbacResStatus::Enable,)];
        let mut group_sql = vec![];
        if let Some(val) = res_param.user_id {
            where_sql.push(sql_format!("  user_id = {}", val));
        } else {
            group_sql.push("user_id");
        }
        if let Some(val) = res_param.app_id {
            where_sql.push(sql_format!("  app_id = {}", val));
        } else {
            group_sql.push("app_id");
        }
        if let Some(val) = res_param.res_type {
            if val.is_empty() {
                return None;
            }
            where_sql.push(sql_format!(" res_type = {}", val));
        }
        group_sql.push("res_type");
        Some((
            if where_sql.is_empty() {
                "".to_string()
            } else {
                format!("where {}", where_sql.join(" and "))
            },
            if group_sql.is_empty() {
                "".to_string()
            } else {
                format!("group by  {}", group_sql.join(","))
            },
        ))
    }
    //获取某资源类型数据
    pub async fn res_type_data(
        &self,
        res_param: &ResTypeListParam<'_>,
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<ResTypeListRecord>> {
        let (where_sql, group_sql) = match self.res_type_sql_where(res_param) {
            Some(op_dat) => op_dat,
            None => return Ok(vec![]),
        };
        let sql = sql_format!(
            "select user_id,app_id,res_type,CONVERT(count(*),UNSIGNED) as total from {} {} {} ",
            RbacResModel::table_name(),
            SqlExpr(where_sql),
            SqlExpr(group_sql)
        );
        let sql = if let Some(pdat) = page {
            format!("{} limit {} offset {} ", sql, pdat.limit, pdat.offset)
        } else {
            sql
        };
        let res = sqlx::query(&sql)
            .try_map(|row: sqlx::mysql::MySqlRow| {
                let user_id = row.try_get::<u64, &str>("user_id").unwrap_or_default();
                let app_id = row.try_get::<u64, &str>("app_id").unwrap_or_default();
                let res_type = row.try_get::<String, &str>("res_type").unwrap_or_default();
                let res_total = row.try_get::<u64, &str>("total").unwrap_or_default();
                Ok(ResTypeListRecord {
                    user_id,
                    app_id,
                    res_type,
                    res_total,
                })
            })
            .fetch_all(&self.db)
            .await;
        Ok(res?)
    }
    //获取某资源类型总数
    pub async fn res_type_count(&self, res_param: &ResTypeListParam<'_>) -> RbacResult<i64> {
        let (where_sql, group_sql) = match self.res_type_sql_where(res_param) {
            Some(op_dat) => op_dat,
            None => return Ok(0),
        };
        let sql = sql_format!(
            "select count(*) as total from (select res_type  from {} {} {}) as t ",
            RbacResModel::table_name(),
            SqlExpr(where_sql),
            SqlExpr(group_sql)
        );
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.db)
            .await?)
    }
}
#[derive(Serialize)]
pub struct RbacOpResRecord {
    pub op_res: RbacOpResModel,
    pub op_data: Option<RbacOpModel>,
}
impl RbacRes {
    //获取某资源可用操作
    pub async fn res_type_op_data(
        &self,
        res_type_data: &ResTypeParam<'_>,
        op_key: Option<&[&str]>,
        fetch_op_data: bool,
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<RbacOpResRecord>> {
        let op_sql = match op_key {
            Some(op_dat) => {
                if op_dat.is_empty() {
                    return Ok(vec![]);
                }
                let op_dat = op_dat
                    .iter()
                    .map(|e| string_clear(e, StringClear::Option(STRING_CLEAR_FORMAT), Some(33)))
                    .collect::<Vec<String>>();
                sql_format!("and op.op_key in ({})", op_dat)
            }
            None => "".to_string(),
        };
        let res_type = string_clear(
            res_type_data.res_type,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(33),
        );
        let sql = sql_format!(
            "select op_res.* from {} as op
                join {} as op_res on op.id=op_res.op_id
                where op.user_id={} and op.status={} and op_res.user_id={} 
                and op_res.app_id={}  and op_res.res_type={}
                and op_res.status={} {}",
            RbacOpModel::table_name(),
            RbacOpResModel::table_name(),
            res_type_data.user_id,
            RbacOpStatus::Enable,
            res_type_data.user_id,
            res_type_data.app_id,
            res_type,
            RbacOpResStatus::Enable,
            SqlExpr(op_sql)
        );
        let sql = if let Some(pdat) = page {
            format!(
                "{} order by op_res.change_time desc limit {} offset {} ",
                sql, pdat.limit, pdat.offset
            )
        } else {
            format!("{} order by op_res.change_time desc ", sql)
        };
        let mut res = sqlx::query_as::<_, RbacOpResModel>(&sql)
            .fetch_all(&self.db)
            .await?
            .into_iter()
            .map(|op_res| RbacOpResRecord {
                op_res,
                op_data: None,
            })
            .collect::<Vec<_>>();

        if fetch_op_data && !res.is_empty() {
            let op_ids = res.iter().map(|e| e.op_res.op_id).collect::<Vec<_>>();
            let sql = sql_format!(
                "select * from {}  where id in ({})",
                RbacOpModel::table_name(),
                op_ids
            );
            let tmp_res = sqlx::query_as::<_, RbacOpModel>(&sql)
                .fetch_all(&self.db)
                .await?;
            res = res
                .into_iter()
                .map(|mut e| {
                    e.op_data = tmp_res
                        .iter()
                        .find(|c| c.id == e.op_res.op_id)
                        .map(|t| t.to_owned());
                    e
                })
                .collect::<Vec<_>>();
        }
        Ok(res)
    }
    pub async fn res_type_op_count(&self, res_type_data: &ResTypeParam<'_>) -> RbacResult<i64> {
        let res_type = string_clear(
            res_type_data.res_type,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(33),
        );
        let sql = sql_format!(
            "select count(*) as total from {} as op
                join {} as op_res on op.id=op_res.op_id
                where op.user_id={} and op.status={} and op_res.user_id={} 
                and op_res.app_id={}  and op_res.res_type={} and op_res.status={}",
            RbacOpModel::table_name(),
            RbacOpResModel::table_name(),
            res_type_data.user_id,
            RbacOpStatus::Enable,
            res_type_data.user_id,
            res_type_data.app_id,
            res_type,
            RbacOpResStatus::Enable,
        );
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.db)
            .await?)
    }
}
