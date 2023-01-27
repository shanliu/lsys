use std::{collections::BTreeMap, sync::Arc};

use crate::model::{
    RbacResModel, RbacResOpModel, RbacResStatus, RbacRoleModel, RbacRoleOpModel, RbacRoleStatus,
    RbacRoleUserModel, RbacTagsModel, RbacTagsSource,
};
use lsys_core::PageParam;
use sqlx_model::SqlQuote;
use sqlx_model::{sql_format, ModelTableName, SqlExpr};

use super::{res::RbacRes, role::RbacRole, RbacTags, UserRbacResult};

pub struct RbacData {
    res: Arc<RbacRes>,
    role: Arc<RbacRole>,
    tags: Arc<RbacTags>,
}

pub struct ResParam<'t> {
    pub user_id: u64,
    pub res_id: &'t Option<Vec<u64>>,
    pub res_name: &'t Option<String>,
    pub filter_tags: &'t Option<Vec<String>>,
    pub out_ops: bool,
    pub out_tags: bool,
    pub page: &'t Option<PageParam>,
}

pub enum RoleUserGroupParam {
    All,
    Ok,
    None,
}
pub struct RoleParam<'t> {
    pub user_id: u64,
    pub user_range: &'t Option<Vec<i8>>,
    pub res_range: &'t Option<Vec<i8>>,
    pub role_name: &'t Option<String>,
    pub role_id: &'t Option<Vec<u64>>,
    pub filter_tags: &'t Option<Vec<String>>,
    pub out_ops: bool,
    pub out_tags: bool,
    pub out_user_data: bool,
    pub out_user_group: RoleUserGroupParam,
    pub page: &'t Option<PageParam>,
    pub user_data_page: &'t Option<PageParam>,
}

impl RbacData {
    pub fn new(res: Arc<RbacRes>, role: Arc<RbacRole>, tags: Arc<RbacTags>) -> Self {
        RbacData { res, role, tags }
    }
    /// 获取资源数量
    pub async fn res_count(
        &self,
        user_id: u64,
        res_name: &Option<String>,
        res_ids: &Option<Vec<u64>>,
        filter_tags: &Option<Vec<String>>,
    ) -> UserRbacResult<i64> {
        let res = if let Some(ftag) = filter_tags {
            let where_sql = if let Some(ref name) = res_name {
                let id_where = if let Some(dat) = res_ids {
                    if dat.is_empty() {
                        return Ok(0);
                    } else {
                        SqlExpr(sql_format!("id in {} and", dat))
                    }
                } else {
                    SqlExpr("".to_string())
                };
                let sql = sql_format!(
                    " and from_id in (
                            select id from {table} 
                            where 
                            {ids}
                            user_id = {user_id} 
                            and status={status}
                            and name like {name}
                    )",
                    ids = id_where,
                    table = RbacResModel::table_name(),
                    user_id = user_id,
                    status = RbacResStatus::Enable as i8,
                    name = format!("%{}%", name)
                );
                Some(sql)
            } else {
                None
            };
            self.tags
                .count_by_name(
                    user_id,
                    ftag,
                    res_ids,
                    RbacTagsSource::Res,
                    where_sql.as_ref(),
                )
                .await?
        } else {
            self.res.get_count(user_id, res_name, res_ids).await?
        };
        Ok(res)
    }
    /// 获取资源数据
    pub async fn res_data<'t>(
        &self,
        res_param: &'t ResParam<'t>,
    ) -> UserRbacResult<Vec<(RbacResModel, Vec<RbacResOpModel>, Vec<RbacTagsModel>)>> {
        let res = if let Some(ftag) = res_param.filter_tags {
            let tags = self
                .tags
                .find_by_name(
                    res_param.user_id,
                    ftag,
                    res_param.res_id,
                    RbacTagsSource::Res,
                    res_param.page,
                )
                .await?;
            self.res
                .get_res(
                    res_param.user_id,
                    res_param.res_name,
                    &Some(tags.iter().map(|e| e.from_id).collect()),
                    &None,
                )
                .await?
        } else {
            self.res
                .get_res(
                    res_param.user_id,
                    res_param.res_name,
                    res_param.res_id,
                    res_param.page,
                )
                .await?
        };
        let ops = if res_param.out_ops {
            self.res
                .res_get_ops(&res.iter().map(|e| e.id).collect::<Vec<_>>())
                .await?
        } else {
            BTreeMap::new()
        };
        let tags = if res_param.out_tags {
            self.res
                .res_get_tags(&res.iter().map(|e| e.id).collect::<Vec<_>>())
                .await?
        } else {
            BTreeMap::new()
        };
        let out = res
            .into_iter()
            .map(|e| {
                let t = tags
                    .get(&e.id)
                    .map(|e| e.to_owned())
                    .unwrap_or_else(Vec::new);
                let o = ops
                    .get(&e.id)
                    .map(|e| e.to_owned())
                    .unwrap_or_else(Vec::new);
                (e, o, t)
            })
            .collect::<Vec<(RbacResModel, Vec<RbacResOpModel>, Vec<RbacTagsModel>)>>();
        Ok(out)
    }
    /// 获取角色数量
    pub async fn role_count(
        &self,
        user_id: u64,
        user_range: &Option<Vec<i8>>,
        res_range: &Option<Vec<i8>>,
        role_name: &Option<String>,
        role_ids: &Option<Vec<u64>>,
        filter_tags: &Option<Vec<String>>,
    ) -> UserRbacResult<i64> {
        let res = if let Some(ftag) = filter_tags {
            let where_sql = if role_name.is_some() || user_range.is_some() || res_range.is_some() {
                let id_where = if let Some(dat) = role_ids {
                    if dat.is_empty() {
                        return Ok(0);
                    } else {
                        SqlExpr(sql_format!("id in {} and", dat))
                    }
                } else {
                    SqlExpr("".to_string())
                };
                let mut sql = sql_format!(
                    "
                            select id from {table} 
                            where 
                            {ids}
                            user_id = {user_id} 
                            and status={status}
                    ",
                    ids = id_where,
                    table = RbacRoleModel::table_name(),
                    user_id = user_id,
                    status = RbacRoleStatus::Enable as i8
                );
                if let Some(ref name) = role_name {
                    sql += &sql_format!(" and name like {name}", name = format!("%{}%", name));
                }
                if let Some(ref ur) = user_range {
                    sql += &sql_format!(" and user_range in  ({})", ur);
                }
                if let Some(ref rr) = res_range {
                    sql += &sql_format!(" and res_op_range in ({})", rr);
                }
                Some(format!(" and from_id in ({})", sql))
            } else {
                None
            };

            self.tags
                .count_by_name(
                    user_id,
                    ftag,
                    role_ids,
                    RbacTagsSource::Role,
                    where_sql.as_ref(),
                )
                .await?
        } else {
            self.role
                .get_count(user_id, user_range, res_range, role_name, role_ids)
                .await?
        };
        Ok(res)
    }
    /// 获取角色数据
    pub async fn role_data<'t>(
        &self,
        role_param: &'t RoleParam<'t>,
    ) -> UserRbacResult<
        Vec<(
            RbacRoleModel,
            Vec<RbacRoleUserModel>,
            Vec<RbacRoleOpModel>,
            Vec<RbacTagsModel>,
            Option<i64>,
        )>,
    > {
        let role = if let Some(ftag) = role_param.filter_tags {
            let tags = self
                .tags
                .find_by_name(
                    role_param.user_id,
                    ftag,
                    role_param.role_id,
                    RbacTagsSource::Role,
                    role_param.page,
                )
                .await?;
            self.role
                .get_role(
                    role_param.user_id,
                    role_param.user_range,
                    role_param.res_range,
                    role_param.role_name,
                    &Some(tags.iter().map(|e| e.from_id).collect()),
                    role_param.page,
                )
                .await?
        } else {
            self.role
                .get_role(
                    role_param.user_id,
                    role_param.user_range,
                    role_param.res_range,
                    role_param.role_name,
                    role_param.role_id,
                    role_param.page,
                )
                .await?
        };

        let ops = if role_param.out_ops {
            self.role
                .role_get_ops(&role.iter().map(|e| e.id).collect::<Vec<_>>())
                .await?
        } else {
            BTreeMap::new()
        };

        let tags = if role_param.out_tags {
            self.role
                .role_get_tags(&role.iter().map(|e| e.id).collect::<Vec<_>>())
                .await?
        } else {
            BTreeMap::new()
        };

        let gs = match role_param.out_user_group {
            RoleUserGroupParam::All => {
                self.role
                    .role_group_users(&role.iter().map(|e| e.id).collect::<Vec<_>>(), true)
                    .await?
            }
            RoleUserGroupParam::Ok => {
                self.role
                    .role_group_users(&role.iter().map(|e| e.id).collect::<Vec<_>>(), false)
                    .await?
            }
            RoleUserGroupParam::None => BTreeMap::new(),
        };

        let us = if role_param.out_user_data {
            self.role
                .role_get_users(
                    &role.iter().map(|e| e.id).collect::<Vec<_>>(),
                    &None,
                    role_param.user_data_page,
                )
                .await?
        } else {
            BTreeMap::new()
        };
        let out = role
            .into_iter()
            .map(|e| {
                let t = tags
                    .get(&e.id)
                    .map(|e| e.to_owned())
                    .unwrap_or_else(Vec::new);
                let o = ops
                    .get(&e.id)
                    .map(|e| e.to_owned())
                    .unwrap_or_else(Vec::new);
                let u = us.get(&e.id).map(|e| e.to_owned()).unwrap_or_else(Vec::new);
                let s = gs.get(&e.id).map(|e| e.to_owned());
                (e, u, o, t, s)
            })
            .collect::<Vec<(
                RbacRoleModel,
                Vec<RbacRoleUserModel>,
                Vec<RbacRoleOpModel>,
                Vec<RbacTagsModel>,
                Option<i64>,
            )>>();
        Ok(out)
    }
}
