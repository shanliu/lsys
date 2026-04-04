use super::res_type::ResTypeParam;
use super::RbacRes;
use crate::dao::result::RbacResult;
use crate::model::{
    RbacOpModel, RbacOpResModel, RbacOpResStatus, RbacOpStatus, RbacPermModel, RbacPermStatus,
    RbacResModel, RbacResStatus,
};
use lsys_core::db::{OffsetPageParam, QueryBuilderExt, TableMeta, WhereClause};
use lsys_core::utils::{string_clear, StringClear, STRING_CLEAR_FORMAT};
use serde::Serialize;
use sqlx::Row;
use sqlx::{MySql, QueryBuilder};
use std::collections::{HashMap, HashSet};
use std::vec;
//RBAC中资源相关实现

//资源的数据获取

impl RbacRes {
    pub async fn find_by_id(&self, id: &u64) -> RbacResult<RbacResModel> {
        Ok(
            lsys_core::db::utils::Fetch::<MySql, RbacResModel>::one(&self.db, |qb| {
                qb.field_eq("id", *id)
                  .push_and()
                  .field_eq("status", RbacResStatus::Enable as i8);
            })
            .await?,
        )
    }
    pub async fn find_by_ids(&self, ids: &[u64]) -> RbacResult<HashMap<u64, RbacResModel>> {
        Ok(lsys_core::db::utils::Fetch::<MySql, RbacResModel>::map(
            &self.db,
            |qb| {
                qb.field_in_copied("id", ids)
                  .push_and()
                  .field_eq("status", RbacResStatus::Enable as i8);
            },
            |v| v.id,
        )
        .await?)
    }
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
    fn res_sql(
        &self,
        filed: &str,
        res_param: &ResDataParam<'_>,
    ) -> Option<QueryBuilder<'static, MySql>> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select {} from {}",
            filed,
            RbacResModel::table_name(),
        ));
        qb.push_where().field_eq("status", RbacResStatus::Enable as i8);
        if let Some(val) = res_param.app_id {
            qb.push_and().field_eq("app_id", val);
        }
        if let Some(val) = res_param.user_id {
            qb.push_and().field_eq("user_id", val);
        }
        if let Some(val) = res_param.res_type {
            let val = string_clear(val, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            if val.is_empty() {
                return None;
            }
            qb.push_and().field_eq("res_type", val);
        }
        if let Some(val) = res_param.res_data {
            let val = string_clear(val, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            qb.push_and().field_eq("res_data", val);
        }
        if let Some(val) = res_param.res_name {
            let val = string_clear(val, StringClear::LikeKeyWord, None);
            qb.push_and().field_like("res_name", format!("%{}%", val));
        }
        if let Some(rid) = res_param.ids {
            if rid.is_empty() {
                return None;
            } else {
                qb.push_and().field_in_copied("id", rid);
            }
        }
        Some(qb)
    }
    /// 获取指定条件的角色数量
    pub async fn res_count(&self, res_param: &ResDataParam<'_>) -> RbacResult<i64> {
        match self.res_sql("count(*) as total", res_param) {
            Some(mut qb) => {
                let res = qb.build_query_scalar::<i64>().fetch_one(&self.db).await?;
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
        let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
            "select res_id,count(*) as total from {}",
            RbacPermModel::table_name(),
        ));
        qb.push_where().field_in_copied("res_id", res_ids);
        qb.push_and().field_eq("status", RbacPermStatus::Enable as i8).push(" group by res_id");
        let perm_counts = qb
            .build()
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
        page: &OffsetPageParam,
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
                res_sql.push((e.res_type.clone(), e.user_id, e.app_id));
            }
            let mut qb = QueryBuilder::<MySql>::new("");
            for (i, (res_type, user_id, app_id)) in res_sql.iter().enumerate() {
                if i > 0 {
                    qb.push(" union all ");
                }
                qb.push(format!(
                    "select res_type,user_id,app_id,count(*) as total from {}",
                    RbacOpResModel::table_name(),
                ));
                qb.push_where().field_eq("status", RbacOpResStatus::Enable as i8);
                qb.push_and().field_eq("res_type", res_type.clone());
                qb.push_and().field_eq("user_id", *user_id);
                qb.push_and().field_eq("app_id", *app_id);
                qb.push(" group by res_type,user_id,app_id");
            }
            let op_counts = qb
                .build()
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
        page: &OffsetPageParam,
    ) -> RbacResult<Vec<RbacResModel>> {
        match self.res_sql("*", res_param) {
            Some(mut qb) => {
                qb.push(" order by id desc");
                page.push_limit(&mut qb);
                let res = qb
                    .build_query_as::<RbacResModel>()
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
    fn res_type_sql_push_where(
        &self,
        wc: &mut WhereClause<'_, '_, MySql>,
        res_param: &ResTypeListParam<'_>,
    ) -> Option<()> {
        wc.and().field_eq("status", RbacResStatus::Enable as i8);
        if let Some(val) = res_param.user_id {
            wc.and().field_eq("user_id", val);
        }
        if let Some(val) = res_param.app_id {
            wc.and().field_eq("app_id", val);
        }
        if let Some(val) = res_param.res_type {
            if val.is_empty() {
                return None;
            }
            wc.and().field_eq("res_type", val.to_string());
        }
        Some(())
    }
    //获取某资源类型数据
    pub async fn res_type_data(
        &self,
        res_param: &ResTypeListParam<'_>,
        page: &OffsetPageParam,
    ) -> RbacResult<Vec<ResTypeListRecord>> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select user_id,app_id,res_type,CONVERT(count(*),UNSIGNED) as total from {}",
            RbacResModel::table_name(),
        ));
        let mut wc = WhereClause::new(&mut qb);
        if self.res_type_sql_push_where(&mut wc, res_param).is_none() {
            return Ok(vec![]);
        }
        wc.builder().push(" group by user_id,app_id,res_type");
        page.push_limit(wc.builder());
        let res = wc.builder().build()
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
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select count(*) as total from (select user_id,app_id,res_type from {}",
            RbacResModel::table_name(),
        ));
        let mut wc = WhereClause::new(&mut qb);
        if self.res_type_sql_push_where(&mut wc, res_param).is_none() {
            return Ok(0);
        }
        wc.builder().push(" group by user_id,app_id,res_type) as t ");
        Ok(wc.builder().build_query_scalar::<i64>()
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
        page: &OffsetPageParam,
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
                Some(op_dat)
            }
            None => None,
        };
        let res_type = string_clear(
            res_type_data.res_type,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(33),
        );
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select op_res.* from {} as op
                join {} as op_res on op.id=op_res.op_id",
            RbacOpModel::table_name(),
            RbacOpResModel::table_name(),
        ));
        qb.push_where().field_eq("op.user_id", res_type_data.user_id);
        qb.push_and().field_eq("op.status", RbacOpStatus::Enable as i8);
        qb.push_and().field_eq("op_res.user_id", res_type_data.user_id);
        qb.push_and().field_eq("op_res.app_id", res_type_data.app_id);
        qb.push_and().field_eq("op_res.res_type", res_type);
        qb.push_and().field_eq("op_res.status", RbacOpResStatus::Enable as i8);
        if let Some(ref op_dat) = op_sql {
            qb.push_and().field_in_string("op.op_key", op_dat);
        }
        qb.push(" order by op_res.change_time desc");
        page.push_limit(&mut qb);
        let mut res = qb
            .build_query_as::<RbacOpResModel>()
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
            let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select * from {}",
                RbacOpModel::table_name(),
            ));
            qb.push_where().field_in_copied("id", &op_ids);
            let tmp_res = qb
                .build_query_as::<RbacOpModel>()
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
        let sql = format!(
            "select count(*) as total from {} as op
                join {} as op_res on op.id=op_res.op_id
                where op.user_id=? and op.status=? and op_res.user_id=? 
                and op_res.app_id=?  and op_res.res_type=? and op_res.status=?",
            RbacOpModel::table_name(),
            RbacOpResModel::table_name(),
        );
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .bind(res_type_data.user_id)
            .bind(RbacOpStatus::Enable as i8)
            .bind(res_type_data.user_id)
            .bind(res_type_data.app_id)
            .bind(&res_type)
            .bind(RbacOpResStatus::Enable as i8)
            .fetch_one(&self.db)
            .await?)
    }
}
