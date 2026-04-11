//RBAC中资源相关实现

use crate::model::{RbacResModel, RbacResStatus};
use lsys_core::db::{QueryBuilderExt, TableMeta};
use lsys_core::utils::{STRING_CLEAR_FORMAT, StringClear, string_clear};
use sqlx::{MySql, QueryBuilder};
use std::vec;

use super::{RbacRes, ResCacheKey};
use crate::dao::res::RbacResCache;
use crate::dao::result::RbacResult;

//资源的授权检查的相关实现

pub struct ResInfo<'t> {
    pub res_type: &'t str, //资源类型
    pub res_data: &'t str, //资源数据
    pub user_id: u64,      //资源用户ID
    pub app_id: u64,       //用户ID下的APPid
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
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select * from {} where (",
            RbacResModel::table_name(),
        ));
        for (i, rkey) in keys.iter().enumerate() {
            if i > 0 {
                qb.push(" or ");
            }
            let res_type = string_clear(
                rkey.res_type,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            let res_data = string_clear(
                rkey.res_data,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            qb.push("(").field_eq("res_type", res_type);
            qb.push_and().field_eq("res_data", res_data);
            qb.push_and().field_eq("user_id", rkey.user_id);
            qb.push_and().field_eq("app_id", rkey.app_id);
            qb.push(")");
        }
        qb.push(")")
            .push_and()
            .field_eq("status", RbacResStatus::Enable as i8);
        let res = qb
            .build_query_as::<RbacResModel>()
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
                                && f.app_id == e.app_id
                        })
                        .map(|f| f.to_owned()),
                )
            })
            .collect();
        Ok(out)
    }
    /// 根据资源KEY获取资源
    pub async fn find_one_by_info<'a>(&self, rkey: &'a ResInfo<'a>) -> RbacResult<RbacResModel> {
        let res_type = string_clear(
            rkey.res_type,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(33),
        );
        let res_data = string_clear(
            rkey.res_data,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(33),
        );
        let sql = format!(
            "select * from {} where
            res_type =? and res_data=? and user_id=? and app_id=? and status =?",
            RbacResModel::table_name(),
        );
        Ok(sqlx::query_as::<_, RbacResModel>(sql.as_str())
            .bind(&res_type)
            .bind(&res_data)
            .bind(rkey.user_id)
            .bind(rkey.app_id)
            .bind(RbacResStatus::Enable as i8)
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
