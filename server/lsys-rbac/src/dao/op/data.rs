use crate::dao::result::RbacResult;
use crate::model::{RbacOpModel, RbacOpStatus};
use std::collections::HashMap;
use std::vec;

use lsys_core::{impl_dao_fetch_one_by_one, PageParam};

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
            sql += sql_format!(" and op_key = {}", val).as_str();
        }
        if let Some(val) = op_param.op_name {
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
