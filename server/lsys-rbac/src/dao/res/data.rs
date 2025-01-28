use super::RbacRes;
use crate::dao::result::RbacResult;
use crate::model::{RbacResModel, RbacResStatus};
use lsys_core::db::{ModelTableName, SqlExpr, SqlQuote};
use lsys_core::sql_format;
use lsys_core::{impl_dao_fetch_one_by_one, PageParam};
use std::collections::HashMap;
use std::vec;
//RBAC中资源相关实现

use crate::model::{RbacOpModel, RbacOpResModel, RbacOpResStatus, RbacOpStatus};

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
        if let Some(val) = res_param.user_id {
            sql += sql_format!(" and user_id = {}", val).as_str();
        }
        if let Some(val) = res_param.res_type {
            sql += sql_format!(" and res_type = {}", val).as_str();
        }
        if let Some(val) = res_param.res_data {
            sql += sql_format!(" and res_data = {}", val).as_str();
        }
        if let Some(val) = res_param.res_name {
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
    /// 获取指定用户和ID的列表
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
                Ok(sqlx::query_as::<_, RbacResModel>(&sql)
                    .fetch_all(&self.db)
                    .await?)
            }
            None => Ok(vec![]),
        }
    }

    //获取某资源可用操作
    pub async fn res_op_data(
        &self,
        res: &RbacResModel,
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<RbacOpModel>> {
        let sql=sql_format!(
            "select op.* from {} as op
                join {} as op_res on op.id=op_res.op_id
                where op.user_id={} and op.status={} and op_res.user_id={} and op_res.res_type={} and op_res.status={}",
            RbacOpModel::table_name(),
            RbacOpResModel::table_name(),
            res.user_id,
            RbacOpStatus::Enable,
            res.user_id,
            res.res_type,
            RbacOpResStatus::Enable,
        );
        let sql = if let Some(pdat) = page {
            format!(
                "{} order by op_res.change_time desc limit {} offset {} ",
                sql, pdat.limit, pdat.offset
            )
        } else {
            format!("{} order by op_res.change_time desc ", sql)
        };
        let res = sqlx::query_as::<_, RbacOpModel>(&sql)
            .fetch_all(&self.db)
            .await?;
        Ok(res)
    }
    pub async fn res_op_count(&self, res: &RbacResModel) -> RbacResult<i64> {
        let sql=sql_format!(
            "select count(*) as total from {} as op
                join {} as op_res on op.id=op_res.op_id
                where op.user_id={} and op.status={} and op_res.user_id={} and op_res.res_type={} and op_res.status={}",
            RbacOpModel::table_name(),
            RbacOpResModel::table_name(),
            res.user_id,
            RbacOpStatus::Enable,
            res.user_id,
            res.res_type,
            RbacOpResStatus::Enable,
        );
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.db)
            .await?)
    }
}
