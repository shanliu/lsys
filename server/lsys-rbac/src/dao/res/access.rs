//RBAC中资源相关实现

use crate::model::{RbacResModel, RbacResStatus};
use lsys_core::db::{ModelTableName, SqlExpr, SqlQuote};
use lsys_core::sql_format;
use std::vec;

use super::{RbacRes, ResCacheKey};
use crate::dao::res::RbacResCache;
use crate::dao::result::RbacResult;

//资源的授权检查的相关实现

pub struct ResInfo<'t> {
    pub res_type: &'t str, //资源类型
    pub res_data: &'t str, //资源数据
    pub user_id: u64,      //资源用户ID
}

//资源管理
impl RbacRes {
    /// 根据资源KEY获取资源
    pub async fn find_vec_by_info<'a>(
        &self,
        keys: &[&'a ResInfo<'a>],
    ) -> RbacResult<Vec<(&'a ResInfo<'a>, Option<RbacResModel>)>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        let mut where_sql = Vec::with_capacity(keys.len());
        for rkey in keys {
            where_sql.push(sql_format!(
                "(res_type ={} and res_data={} and user_id={})",
                rkey.res_type,
                rkey.res_data,
                rkey.user_id,
            ));
        }
        let sql = sql_format!(
            "select * from {} where
            ({}) and status ={}",
            RbacResModel::table_name(),
            SqlExpr(where_sql.join(" or ")),
            RbacResStatus::Enable as i8
        );
        let res = sqlx::query_as::<_, RbacResModel>(sql.as_str())
            .fetch_all(&self.db)
            .await?;
        let out = keys
            .iter()
            .map(|e| {
                (
                    *e,
                    res.iter()
                        .find(|f| {
                            f.res_type.as_str() == e.res_type
                                && f.res_data.as_str() == e.res_data
                                && f.user_id == e.user_id
                        })
                        .map(|f| f.to_owned()),
                )
            })
            .collect();
        Ok(out)
    }
    /// 根据资源KEY获取资源
    pub async fn find_one_by_info<'a>(&self, rkey: &'a ResInfo<'a>) -> RbacResult<RbacResModel> {
        let sql = sql_format!(
            "select * from {} where
            res_type ={} and res_data={} and user_id={} and status ={}",
            RbacResModel::table_name(),
            rkey.res_type,
            rkey.res_data,
            rkey.user_id,
            RbacResStatus::Enable as i8
        );
        Ok(sqlx::query_as::<_, RbacResModel>(sql.as_str())
            .fetch_one(&self.db)
            .await?)
    }
}

impl RbacResCache<'_> {
    pub async fn find_vec_by_info<'a>(
        &self,
        keys: &'a [ResInfo<'a>],
    ) -> RbacResult<Vec<(&'a ResInfo<'a>, Option<RbacResModel>)>> {
        let mut get = vec![];
        let mut out = vec![];
        for tmp in keys {
            match self
                .res
                .cache_res_data
                .get(&ResCacheKey {
                    res_type: tmp.res_type.to_owned(),
                    res_data: tmp.res_data.to_owned(),
                    user_id: tmp.user_id,
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
            match self.res.find_vec_by_info(&get).await {
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
