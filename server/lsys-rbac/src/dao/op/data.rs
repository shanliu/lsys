use crate::dao::result::RbacResult;
use crate::model::{
    RbacOpModel, RbacOpResModel, RbacOpResStatus, RbacOpStatus, RbacPermModel, RbacPermStatus,
};
use lsys_core::{
    impl_dao_fetch_one_by_one, string_clear, PageParam, StringClear, STRING_CLEAR_FORMAT,
};
use sqlx::Row;
use std::collections::HashMap;
use std::vec;

use lsys_core::db::{ModelTableName, SqlExpr, SqlQuote};
use lsys_core::sql_format;

use super::RbacOp;

//资源操作相关数据获取

impl RbacOp {
    impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        RbacOpModel,
        RbacResult<RbacOpModel>,
        id,
        "id={id} and status = {status}",
        status = RbacOpStatus::Enable
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        RbacOpModel,
        RbacResult<HashMap<u64, RbacOpModel>>,
        id,
        ids,
        "id in ({ids}) and  status = {status}",
        status = RbacOpStatus::Enable
    );
}

pub struct OpDataParam<'t> {
    pub user_id: u64,
    pub app_id: Option<u64>,
    pub op_name: Option<&'t str>,
    pub op_key: Option<&'t str>,
    pub ids: Option<&'t [u64]>,
}

//资源管理
impl RbacOp {
    fn op_sql(&self, field: &str, op_param: &OpDataParam<'_>) -> Option<String> {
        let mut sql = sql_format!(
            "select {} from {} where user_id = {} and status={}",
            SqlExpr(field),
            RbacOpModel::table_name(),
            op_param.user_id,
            RbacOpStatus::Enable,
        );
        if let Some(val) = op_param.app_id {
            sql += sql_format!(" and app_id = {}", val).as_str();
        }
        if let Some(val) = op_param.op_key {
            let val = string_clear(val, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            if val.is_empty() {
                return None;
            }
            sql += sql_format!(" and op_key = {}", val).as_str();
        }
        if let Some(val) = op_param.op_name {
            let val = string_clear(val, StringClear::LikeKeyWord, None);
            sql += sql_format!(" and op_name like {}", format!("%{}%", val)).as_str();
        }
        if let Some(rid) = op_param.ids {
            if rid.is_empty() {
                return None;
            } else {
                sql += &sql_format!(" and id in ({})", rid);
            }
        }
        Some(sql)
    }
    /// 获取指定条件的角色数量
    pub async fn op_count(&self, op_param: &OpDataParam<'_>) -> RbacResult<i64> {
        match self.op_sql("count(*) as total", op_param) {
            Some(sql) => {
                let query = sqlx::query_scalar::<_, i64>(&sql);
                let res = query.fetch_one(&self.db).await?;
                Ok(res)
            }
            None => Ok(0),
        }
    }
}
impl RbacOp {
    /// 获取资源关联的授权数量
    pub async fn op_res_type_count(&self, op_ids: &[u64]) -> RbacResult<Vec<(u64, i64)>> {
        if op_ids.is_empty() {
            return Ok(vec![]);
        }
        let sql = sql_format!(
                "select op_id,count(*) as total from {} where op_id in ({}) and status={} group by op_id",
                RbacOpResModel::table_name(),
                op_ids,
                RbacOpResStatus::Enable
            );
        let op_counts = sqlx::query(&sql)
            .try_map(|row: sqlx::mysql::MySqlRow| {
                let res_id = row.try_get::<u64, &str>("op_id").unwrap_or_default();
                let total = row.try_get::<i64, &str>("total").unwrap_or_default();
                Ok((res_id, total))
            })
            .fetch_all(&self.db)
            .await?;
        Ok(op_counts)
    }
    /// 是否有被角色关联
    pub async fn op_role_use(&self, op_ids: &[u64]) -> RbacResult<Vec<(u64, bool)>> {
        if op_ids.is_empty() {
            return Ok(vec![]);
        }
        let mut sql = Vec::with_capacity(op_ids.len());
        for oid in op_ids {
            sql.push(sql_format!(
                "select op_id from {} where op_id={} and status={} limit 1",
                RbacPermModel::table_name(),
                oid,
                RbacPermStatus::Enable
            ));
        }
        let op_counts = sqlx::query_scalar::<_, u64>(&sql.join(" union all"))
            .fetch_all(&self.db)
            .await?;
        Ok(op_ids
            .iter()
            .map(|e| (*e, op_counts.contains(e)))
            .collect::<Vec<_>>())
    }
}

#[derive(Default)]
pub struct OpDataAttrParam {
    //关联资源类型数量
    pub res_type_count: bool,
    //是否被角色使用
    pub check_role_use: bool,
}
#[derive(Default)]
pub struct RbacOpInfoData {
    pub res_type_count: i64,
    pub is_role_use: bool,
}
impl RbacOp {
    pub async fn op_info(
        &self,
        op_param: &OpDataParam<'_>,
        op_attr: &OpDataAttrParam,
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<(RbacOpModel, RbacOpInfoData)>> {
        let res = self.op_data(op_param, page).await?;
        let mut res_type_map: HashMap<u64, i64> = HashMap::new();
        if op_attr.res_type_count && !res.is_empty() {
            let perm_counts = self
                .op_res_type_count(&res.iter().map(|e| e.id).collect::<Vec<_>>())
                .await?;
            for (res_id, total) in perm_counts {
                res_type_map.insert(res_id, total);
            }
        }
        let mut role_map: HashMap<u64, bool> = HashMap::new();
        if op_attr.check_role_use && !res.is_empty() {
            let perm_counts = self
                .op_role_use(&res.iter().map(|e| e.id).collect::<Vec<_>>())
                .await?;
            for (res_id, total) in perm_counts {
                role_map.insert(res_id, total);
            }
        }
        Ok(res
            .into_iter()
            .map(|e| {
                let info = RbacOpInfoData {
                    res_type_count: *res_type_map.get(&e.id).unwrap_or(&0),
                    is_role_use: *role_map.get(&e.id).unwrap_or(&false),
                };
                (e, info)
            })
            .collect::<Vec<_>>())
    }
    /// 获取指定用户和ID的列表
    pub async fn op_data(
        &self,
        op_param: &OpDataParam<'_>,
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<RbacOpModel>> {
        match self.op_sql("*", op_param) {
            Some(mut sql) => {
                if let Some(pdat) = page {
                    sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
                }
                Ok(sqlx::query_as::<_, RbacOpModel>(&sql)
                    .fetch_all(&self.db)
                    .await?)
            }
            None => Ok(vec![]),
        }
    }
}
