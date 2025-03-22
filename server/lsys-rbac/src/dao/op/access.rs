//RBAC中资源相关实现

use crate::dao::result::RbacResult;
use crate::model::{RbacOpModel, RbacOpStatus};
use lsys_core::db::{ModelTableName, SqlExpr, SqlQuote};
use lsys_core::sql_format;
use std::vec;

use super::cache::OpCacheKey;
use super::{RbacOp, RbacOpCache};

pub struct OpInfo<'t> {
    pub op_key: &'t str, //资源数据
    pub user_id: u64,    //资源用户ID
    pub app_id: u64,
}

//授权检查跟资源操作相关实现

//资源管理
impl RbacOp {
    /// 根据资源KEY获取资源
    pub async fn find_vec_by_info<'a>(
        &self,
        keys: &[&'a OpInfo<'a>],
    ) -> RbacResult<Vec<(&'a OpInfo<'a>, Option<RbacOpModel>)>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        let mut where_sql = Vec::with_capacity(keys.len());
        for rkey in keys {
            where_sql.push(sql_format!(
                "(op_key ={}  and user_id={} and app_id={})",
                rkey.op_key,
                rkey.user_id,
                rkey.app_id,
            ));
        }
        let sql = sql_format!(
            "select * from {} where
            ({}) and status ={}",
            RbacOpModel::table_name(),
            SqlExpr(where_sql.join(" or ")),
            RbacOpStatus::Enable as i8
        );
        let res = sqlx::query_as::<_, RbacOpModel>(sql.as_str())
            .fetch_all(&self.db)
            .await?;
        Ok(keys
            .iter()
            .map(|e| {
                (
                    *e,
                    res.iter()
                        .find(|f| {
                            f.op_key == e.op_key && f.user_id == e.user_id && f.app_id == e.app_id
                        })
                        .map(|f| f.to_owned()),
                )
            })
            .collect())
    }
    /// 根据资源KEY获取资源
    pub async fn find_one_by_info<'a>(&self, rkey: &'a OpInfo<'a>) -> RbacResult<RbacOpModel> {
        let sql = sql_format!(
            "select * from {} where
            op_key ={}  and user_id={} and app_id={} and status ={}",
            RbacOpModel::table_name(),
            rkey.op_key,
            rkey.user_id,
            rkey.app_id,
            RbacOpStatus::Enable as i8
        );
        Ok(sqlx::query_as::<_, RbacOpModel>(sql.as_str())
            .fetch_one(&self.db)
            .await?)
    }
}

impl RbacOpCache<'_> {
    pub async fn find_vec_by_info<'a>(
        &self,
        keys: &'a [OpInfo<'a>],
    ) -> RbacResult<Vec<(&'a OpInfo<'a>, Option<RbacOpModel>)>> {
        let mut get = vec![];
        let mut out = vec![];
        for tmp in keys {
            match self
                .op
                .cache_op_data
                .get(&OpCacheKey {
                    op_key: tmp.op_key.to_owned(),
                    user_id: tmp.user_id,
                    app_id: tmp.app_id,
                })
                .await
            {
                Some(data) => {
                    out.push((tmp, data));
                }
                None => {
                    get.push(tmp);
                }
            }
        }
        if !get.is_empty() {
            match self.op.find_vec_by_info(&get).await {
                Ok(datas) => {
                    for (tinfo, tmod) in datas {
                        out.push((tinfo, tmod))
                    }
                }
                Err(err) => return Err(err),
            }
        }
        Ok(out)
    }
}
