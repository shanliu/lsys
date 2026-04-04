use crate::dao::result::RbacResult;
use crate::model::{
    RbacOpModel, RbacOpResModel, RbacOpResStatus, RbacOpStatus, RbacPermModel, RbacPermStatus,
};
use lsys_core::db::{OffsetPageParam, QueryBuilderExt};
use lsys_core::utils::{string_clear, StringClear, STRING_CLEAR_FORMAT};
use sqlx::Row;
use std::collections::HashMap;
use std::vec;

use lsys_core::db::TableMeta;
use sqlx::{MySql, QueryBuilder};

use super::RbacOp;

//资源操作相关数据获取

impl RbacOp {
    pub async fn find_by_id(&self, id: &u64) -> RbacResult<RbacOpModel> {
        Ok(lsys_core::db::utils::Fetch::<MySql, RbacOpModel>::one(
            &self.db,
            |qb| {
                qb.field_eq("id", *id)
                  .push_and()
                  .field_eq("status", RbacOpStatus::Enable as i8);
            },
        ).await?)
    }
    pub async fn find_by_ids(&self, ids: &[u64]) -> RbacResult<HashMap<u64, RbacOpModel>> {
        Ok(lsys_core::db::utils::Fetch::<MySql, RbacOpModel>::map(
            &self.db,
            |qb| {
                qb.field_in_copied("id", ids)
                  .push_and()
                  .field_eq("status", RbacOpStatus::Enable as i8);
            },
            |v| v.id,
        ).await?)
    }
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
    fn op_sql(&self, field: &str, op_param: &OpDataParam<'_>) -> Option<QueryBuilder<'static, MySql>> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select {} from {}",
            field,
            RbacOpModel::table_name(),
        ));
        qb.push_where().field_eq("user_id", op_param.user_id);
        qb.push_and().field_eq("status", RbacOpStatus::Enable as i8);
        if let Some(val) = op_param.app_id {
            qb.push_and().field_eq("app_id", val);
        }
        if let Some(val) = op_param.op_key {
            let val = string_clear(val, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            if val.is_empty() {
                return None;
            }
            qb.push_and().field_eq("op_key", val);
        }
        if let Some(val) = op_param.op_name {
            let val = string_clear(val, StringClear::LikeKeyWord, None);
            qb.push_and().field_like("op_name", format!("%{}%", val));
        }
        if let Some(rid) = op_param.ids {
            if rid.is_empty() {
                return None;
            } else {
                qb.push_and().field_in_copied("id", rid);
            }
        }
        Some(qb)
    }
    /// 获取指定条件的角色数量
    pub async fn op_count(&self, op_param: &OpDataParam<'_>) -> RbacResult<i64> {
        match self.op_sql("count(*) as total", op_param) {
            Some(mut qb) => {
                let res = qb.build_query_scalar::<i64>().fetch_one(&self.db).await?;
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
        let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select op_id,count(*) as total from {}",
                RbacOpResModel::table_name(),
            ));
        qb.push_where().field_in_copied("op_id", op_ids);
        qb.push_and().field_eq("status", RbacOpResStatus::Enable as i8).push(" group by op_id");
        let op_counts = qb.build()
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
        let mut qb = QueryBuilder::<MySql>::new(
            "select * from ((",
        );
        for (i, oid) in op_ids.iter().enumerate() {
            if i > 0 {
                qb.push(") union all (");
            }
            qb.push(format!(
                "select op_id from {}",
                RbacPermModel::table_name(),
            ));
            qb.push_where().field_eq("op_id", *oid);
            qb.push_and().field_eq("status", RbacPermStatus::Enable as i8);
            qb.push(" limit 1");
        }
        qb.push(")) as t");
        let op_counts = qb.build_query_scalar::<u64>()
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
        page: &OffsetPageParam,
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
        page: &OffsetPageParam,
    ) -> RbacResult<Vec<RbacOpModel>> {
        match self.op_sql("*", op_param) {
            Some(mut qb) => {
                qb.push(" order by id desc");
                page.push_limit(&mut qb);
                Ok(qb.build_query_as::<RbacOpModel>()
                    .fetch_all(&self.db)
                    .await?)
            }
            None => Ok(vec![]),
        }
    }
}
