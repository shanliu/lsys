use super::RbacRole;
use crate::{
    dao::result::RbacResult,
    model::{
        RbacOpModel, RbacPermModel, RbacPermStatus, RbacResModel, RbacRoleModel, RbacRoleResRange,
        RbacRoleStatus, RbacRoleUserModel, RbacRoleUserRange, RbacRoleUserStatus,
    },
};
use lsys_core::{
    db::{ModelTableName, SqlExpr, SqlQuote},
    string_clear, STRING_CLEAR_FORMAT,
};
use lsys_core::{impl_dao_fetch_map_by_vec, impl_dao_fetch_one_by_one, now_time, PageParam};
use lsys_core::{sql_format, StringClear};
use serde::Serialize;
use sqlx::Row;
use std::{collections::HashMap, vec};

//角色数据的获取

impl RbacRole {
    impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        RbacRoleModel,
        RbacResult<RbacRoleModel>,
        id,
        "id={id} and status = {status}",
        status = RbacRoleStatus::Enable
    );
    impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        RbacRoleModel,
        RbacResult<HashMap<u64, RbacRoleModel>>,
        id,
        id,
        "id in ({id}) and status = {status}",
        status = RbacRoleStatus::Enable
    );
}

pub struct RoleDataParam<'t> {
    pub user_id: u64,
    pub app_id: Option<u64>,
    pub user_range: Option<RbacRoleUserRange>,
    pub res_range: Option<RbacRoleResRange>,
    pub role_key: Option<&'t str>,
    pub role_name: Option<&'t str>,
    pub ids: Option<&'t [u64]>,
}

//资源管理
impl RbacRole {
    fn role_sql(&self, filed: &str, role_param: &RoleDataParam<'_>) -> Option<String> {
        let mut sql = sql_format!(
            "select {} from {} where user_id = {} and status={}",
            SqlExpr(filed),
            RbacRoleModel::table_name(),
            role_param.user_id,
            RbacRoleStatus::Enable,
        );
        if let Some(val) = role_param.app_id {
            sql += sql_format!(" and app_id = {}", val).as_str();
        }
        if let Some(val) = role_param.role_key {
            let val = string_clear(val, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            if val.is_empty() {
                return None;
            }
            sql += sql_format!(" and role_key = {}", val).as_str();
        }
        if let Some(val) = role_param.user_range {
            sql += sql_format!(" and user_range = {}", val as i8).as_str();
        }
        if let Some(val) = role_param.res_range {
            sql += sql_format!(" and res_range = {}", val as i8).as_str();
        }
        if let Some(val) = role_param.role_name {
            let val = string_clear(val, StringClear::LikeKeyWord, None);
            sql += sql_format!(" and role_name like {}", format!("%{}%", val)).as_str();
        }
        if let Some(rid) = role_param.ids {
            if rid.is_empty() {
                return None;
            } else {
                sql += &sql_format!(" and id in ({})", rid);
            }
        }
        Some(sql)
    }
    /// 获取指定条件的角色数量
    pub async fn role_count(&self, role_param: &RoleDataParam<'_>) -> RbacResult<i64> {
        match self.role_sql("count(*) as total", role_param) {
            Some(sql) => {
                let query = sqlx::query_scalar::<_, i64>(&sql);
                let res = query.fetch_one(&self.db).await?;
                Ok(res)
            }
            None => Ok(0),
        }
    }
    /// 获取指定用户和ID的列表
    pub async fn role_data(
        &self,
        role_param: &RoleDataParam<'_>,
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<RbacRoleModel>> {
        match self.role_sql("*", role_param) {
            Some(mut sql) => {
                if let Some(pdat) = page {
                    sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
                }
                Ok(sqlx::query_as::<_, RbacRoleModel>(&sql)
                    .fetch_all(&self.db)
                    .await?)
            }
            None => Ok(vec![]),
        }
    }
}

#[derive(Default)]
pub struct RoleDataAttrParam {
    pub user_count: Option<bool>,
    pub user_data: Option<u64>,
    pub res_count: Option<bool>,
    pub res_op_count: Option<bool>,
}
#[derive(Default)]
pub struct RbacRoleInfoData {
    pub user_count: Option<i64>,
    pub user_data: Option<Vec<RbacRoleUserModel>>,
    pub res_count: Option<i64>,
    pub res_op_count: Option<i64>,
}
impl RbacRole {
    /// 获取角色对应的资源授权数量
    pub async fn role_res_op_count(&self, role_id: &[u64]) -> RbacResult<Vec<(u64, i64)>> {
        if role_id.is_empty() {
            return Ok(vec![]);
        }
        let sql = sql_format!(
                "select role_id,count(*) as total from {} where role_id in ({}) and status={} group by role_id",
                RbacPermModel::table_name(),
                role_id,
                RbacPermStatus::Enable
            );
        let perm_counts = sqlx::query(&sql)
            .try_map(|row: sqlx::mysql::MySqlRow| {
                let role_id = row.try_get::<u64, &str>("role_id").unwrap_or_default();
                let total = row.try_get::<i64, &str>("total").unwrap_or_default();
                Ok((role_id, total))
            })
            .fetch_all(&self.db)
            .await?;
        Ok(perm_counts)
    }
    /// 获取角色对应的资源数量
    pub async fn role_res_count(&self, role_id: &[u64]) -> RbacResult<Vec<(u64, i64)>> {
        if role_id.is_empty() {
            return Ok(vec![]);
        }
        let sql = sql_format!(
                "select role_id,COUNT(DISTINCT res_id) as total from {} where role_id in ({}) and status={} group by role_id",
                RbacPermModel::table_name(),
                role_id,
                RbacPermStatus::Enable
            );
        let perm_counts = sqlx::query(&sql)
            .try_map(|row: sqlx::mysql::MySqlRow| {
                let role_id = row.try_get::<u64, &str>("role_id").unwrap_or_default();
                let total = row.try_get::<i64, &str>("total").unwrap_or_default();
                Ok((role_id, total))
            })
            .fetch_all(&self.db)
            .await?;
        Ok(perm_counts)
    }
    /// 获取角色详细信息包括扩展数据
    pub async fn role_info(
        &self,
        role_param: &RoleDataParam<'_>,
        role_attr: &RoleDataAttrParam,
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<(RbacRoleModel, RbacRoleInfoData)>> {
        let role_data = self.role_data(role_param, page).await?;
        let user_count_map = if role_attr.user_count.unwrap_or(false) {
            let role_ids = role_data.iter().map(|e| e.id).collect::<Vec<_>>();
            self.role_user_group_count(&role_ids, false)
                .await?
                .into_iter()
                .collect::<HashMap<u64, i64>>()
        } else {
            HashMap::new()
        };
        let user_data_limit = role_attr.user_data.unwrap_or(0);
        let user_data_map = if user_data_limit > 0 {
            let role_ids = role_data.iter().map(|e| e.id).collect::<Vec<_>>();
            self.role_user_group_data(
                &role_ids,
                None,
                false,
                Some(&lsys_core::PageParam::new(0, user_data_limit)),
            )
            .await?
        } else {
            HashMap::new()
        };
        let res_count_map = if role_attr.res_count.unwrap_or(false) {
            let role_ids = role_data.iter().map(|e| e.id).collect::<Vec<_>>();
            let res_counts = self.role_res_count(&role_ids).await?;
            res_counts.into_iter().collect::<HashMap<u64, i64>>()
        } else {
            HashMap::new()
        };
        let res_op_count_map = if role_attr.res_op_count.unwrap_or(false) {
            let role_ids = role_data.iter().map(|e| e.id).collect::<Vec<_>>();
            let res_op_counts = self.role_res_op_count(&role_ids).await?;
            res_op_counts.into_iter().collect::<HashMap<u64, i64>>()
        } else {
            HashMap::new()
        };
        Ok(role_data
            .into_iter()
            .map(|e| {
                let user_count = user_count_map.get(&e.id).copied();
                let user_data = user_data_map.get(&e.id).map(|v| v.to_owned());
                let res_count = res_count_map.get(&e.id).copied();
                let res_op_count = res_op_count_map.get(&e.id).copied();
                (
                    e,
                    RbacRoleInfoData {
                        user_count,
                        user_data,
                        res_count,
                        res_op_count,
                    },
                )
            })
            .collect::<Vec<_>>())
    }
}

#[derive(Serialize)]
pub struct RolePermData {
    pub user_id: u64,
    pub op_id: u64,
    pub op_key: String,
    pub op_name: String,
    pub op_status: i8,
    pub res_id: u64,
    pub res_type: String,
    pub res_data: String,
    pub res_name: String,
    pub res_status: i8,
    pub change_user_id: u64,
    pub change_time: u64,
}
impl RbacRole {
    pub async fn role_perm_data(
        &self,
        role: &RbacRoleModel,
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<RolePermData>> {
        if !RbacRoleResRange::Exclude.eq(role.res_range)
            && !RbacRoleResRange::Include.eq(role.res_range)
        {
            return Ok(vec![]);
        }
        let mut sql = sql_format!(
            "select 
                res.user_id,perm.change_user_id,perm.change_time,
                res.id as res_id,res.res_type,res.res_data,res.status as res_status,
                op.id as op_id,op.op_key,op.op_name,op.status as op_status 
            from 
            {} as perm
            join {} as res on perm.res_id=res.id
            join {} as op on perm.op_id=op.id
            where perm.status={} 
                and perm.role_id={}
            order by perm.id desc",
            RbacPermModel::table_name(),
            RbacResModel::table_name(),
            RbacOpModel::table_name(),
            RbacPermStatus::Enable,
            role.id,
        );
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }
        Ok(sqlx::query(&sql)
            .try_map(|row: sqlx::mysql::MySqlRow| {
                let user_id = row.try_get::<u64, &str>("user_id").unwrap_or_default();
                let op_id = row.try_get::<u64, &str>("op_id").unwrap_or_default();
                let op_key = row.try_get::<String, &str>("op_key").unwrap_or_default();
                let op_name = row.try_get::<String, &str>("op_name").unwrap_or_default();
                let op_status = row.try_get::<i8, &str>("op_status").unwrap_or_default();
                let res_id = row.try_get::<u64, &str>("res_id").unwrap_or_default();
                let res_type = row.try_get::<String, &str>("res_type").unwrap_or_default();
                let res_data = row.try_get::<String, &str>("res_data").unwrap_or_default();
                let res_name = row.try_get::<String, &str>("res_name").unwrap_or_default();
                let res_status = row.try_get::<i8, &str>("res_status").unwrap_or_default();
                let change_user_id = row
                    .try_get::<u64, &str>("change_user_id")
                    .unwrap_or_default();
                let change_time = row.try_get::<u64, &str>("change_time").unwrap_or_default();
                Ok(RolePermData {
                    user_id,
                    op_id,
                    op_key,
                    op_name,
                    op_status,
                    res_id,
                    res_type,
                    res_data,
                    res_name,
                    res_status,
                    change_user_id,
                    change_time,
                })
            })
            .fetch_all(&self.db)
            .await?)
    }
    pub async fn role_perm_count(&self, role: &RbacRoleModel) -> RbacResult<i64> {
        if !RbacRoleResRange::Exclude.eq(role.res_range)
            && !RbacRoleResRange::Include.eq(role.res_range)
        {
            return Ok(0);
        }
        Ok(sqlx::query_scalar::<_, i64>(&sql_format!(
            "select count(*) as total from {} where status={} and role_id={}",
            RbacPermModel::table_name(),
            RbacPermStatus::Enable,
            role.id,
        ))
        .fetch_one(&self.db)
        .await?)
    }
}

impl RbacRole {
    pub async fn role_user_data(
        &self,
        role: &RbacRoleModel,
        all: bool,
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<RbacRoleUserModel>> {
        if !RbacRoleUserRange::Custom.eq(role.user_range) {
            return Ok(vec![]);
        }
        let mut sql = sql_format!(
            "select  * from  {}  where status={} and role_id={} {}
            order by id desc",
            RbacRoleUserModel::table_name(),
            RbacRoleUserStatus::Enable,
            role.id,
            if all {
                SqlExpr("".to_string())
            } else {
                SqlExpr(sql_format!(
                    " and (timeout=0 or timeout>{})",
                    now_time().unwrap_or(0)
                ))
            }
        );
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }
        Ok(sqlx::query_as::<_, RbacRoleUserModel>(&sql)
            .fetch_all(&self.db)
            .await?)
    }
    pub async fn role_user_count(&self, role: &RbacRoleModel, all: bool) -> RbacResult<i64> {
        if !RbacRoleUserRange::Custom.eq(role.user_range) {
            return Ok(0);
        }
        Ok(sqlx::query_scalar::<_, i64>(&sql_format!(
            "select count(*) as total from {} where status={} and role_id={} {}",
            RbacRoleUserModel::table_name(),
            RbacRoleUserStatus::Enable,
            role.id,
            if all {
                SqlExpr("".to_string())
            } else {
                SqlExpr(sql_format!(
                    " and (timeout=0 or timeout>{})",
                    now_time().unwrap_or(0)
                ))
            }
        ))
        .fetch_one(&self.db)
        .await?)
    }
    //一批角色的的角色包含用户数量
    pub async fn role_user_group_count(
        &self,
        role_ids: &[u64],
        all: bool,
    ) -> RbacResult<Vec<(u64, i64)>> {
        if role_ids.is_empty() {
            return Ok(vec![]);
        }
        let sql = sql_format!(
            "select role_id,
            count(*) as total
            from {} 
            where role_id in ({}) 
            and status={}
            {}
            group by role_id ",
            RbacRoleUserModel::table_name(),
            role_ids,
            RbacRoleUserStatus::Enable as i8,
            if all {
                SqlExpr("".to_string())
            } else {
                SqlExpr(sql_format!(
                    " and (timeout=0 or timeout>{})",
                    now_time().unwrap_or(0)
                ))
            }
        );
        Ok(sqlx::query_as::<_, (u64, i64)>(sql.as_str())
            .fetch_all(&self.db)
            .await?)
    }
    /// 一批角色获取角色对应用户
    pub async fn role_user_group_data(
        &self,
        role_ids: &[u64],
        user_ids: Option<&[u64]>, //用在检查指定用户id是否已经添加
        all: bool,
        page: Option<&PageParam>,
    ) -> RbacResult<HashMap<u64, Vec<RbacRoleUserModel>>> {
        if role_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let mut sql = sql_format!(
            "select * from {} where role_id in ({}) and status={} {}",
            RbacRoleUserModel::table_name(),
            role_ids,
            RbacRoleUserStatus::Enable,
            if all {
                SqlExpr("".to_string())
            } else {
                SqlExpr(sql_format!(
                    " and (timeout=0 or timeout>{})",
                    now_time().unwrap_or(0)
                ))
            }
        );

        if let Some(u_ids) = user_ids {
            sql += &sql_format!(" and user_id in ({})", u_ids);
        }
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }
        let data = sqlx::query_as::<_, RbacRoleUserModel>(&sql)
            .fetch_all(&self.db)
            .await?;
        let mut map: HashMap<u64, Vec<RbacRoleUserModel>> = HashMap::new();
        for tmp in data {
            map.entry(tmp.role_id).or_default().push(tmp);
        }
        Ok(map)
    }
}
