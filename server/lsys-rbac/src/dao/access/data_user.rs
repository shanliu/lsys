use super::RbacAccess;
use crate::model::RbacOpModel;
use crate::model::RbacOpStatus;
use crate::model::RbacPermModel;
use crate::model::RbacPermStatus;
use crate::model::RbacResModel;
use crate::model::RbacResStatus;
use crate::model::RbacRoleModel;
use crate::model::RbacRoleStatus;
use crate::model::RbacRoleUserModel;
use crate::model::RbacRoleUserRange;
use crate::model::RbacRoleUserStatus;
use crate::{
    dao::{
        op::OpInfo,
        res::ResInfo,
        result::{RbacError, RbacResult},
    },
    model::RbacRoleResRange,
};
use lsys_core::db::OffsetPageParam;
use lsys_core::db::QueryBuilderExt;
use lsys_core::db::TableMeta;
use lsys_core::utils::{STRING_CLEAR_FORMAT, StringClear, string_clear};
use serde::Serialize;
use sqlx::{MySql, QueryBuilder, Row};
use std::vec;

//资源查找由 系统授权 或 用户授权 的 可被访问角色 及 角色关联用户
//1. 资源授权非特定用户,只有角色数据
//2. 资源授权由角色管理的用户,可以获取到用户列表

#[derive(Serialize)]
pub struct AccessPublicResUserData {
    pub exist_system_session_all: bool, //role list 存在系统授权的会话角色可访问  -> 可通过 find_role_list_from_res 返回角色列表
    pub exist_system_user_all: bool, //user role list 存在系统授权的特定用户可访问 -> 可通过 find_user_list_from_res 返回用户列表
    pub exist_self_session_all: bool, //role list  -> 可通过 find_role_list_from_res 返回角色列表
    pub exist_self_user_all: bool,   //user role list -> 可通过 find_user_list_from_res 返回用户列表
}

impl RbacAccess {
    //获取配置为 可访问 任意资源 的 角色或用户信息
    //配置方: 系统或特定用户
    //资源范围: 任意资源
    //返回 角色详细
    pub async fn find_user_data_from_public(
        &self,
        user_id: u64, //资源用户ID
        app_id: u64,
    ) -> RbacResult<AccessPublicResUserData> {
        let mut qb = self.user_data_pub_sql(user_id, app_id).await;
        let data = qb
            .build_query_as::<(u64, i8, i8)>()
            .fetch_all(&self.db)
            .await?;
        let mut pub_access = AccessPublicResUserData {
            exist_system_session_all: false,
            exist_system_user_all: false,
            exist_self_session_all: false,
            exist_self_user_all: false,
        };
        for (db_user_id, db_user_range, db_res_range) in data {
            if !RbacRoleResRange::Any.eq(db_res_range) {
                continue;
            }
            if db_user_id == 0 {
                if RbacRoleUserRange::Session.eq(db_user_range) {
                    pub_access.exist_system_session_all = true;
                }
                if RbacRoleUserRange::Custom.eq(db_user_range) {
                    pub_access.exist_system_user_all = true;
                }
            } else {
                if RbacRoleUserRange::Session.eq(db_user_range) {
                    pub_access.exist_self_session_all = true;
                }
                if RbacRoleUserRange::Custom.eq(db_user_range) {
                    pub_access.exist_self_user_all = true;
                }
            }
        }
        Ok(pub_access)
    }
    async fn user_data_pub_sql(&self, user_id: u64, app_id: u64) -> QueryBuilder<'static, MySql> {
        let mut user_data = vec![0u64];
        if user_id > 0 {
            user_data.push(user_id);
        }
        let mut qb = QueryBuilder::<MySql>::new("select * from ((");
        qb.push(format!(
            "select role.user_id,role.user_range,role.res_range
            from {} as role",
            RbacRoleModel::table_name(),
        ));
        qb.push_where()
            .field_eq("role.status", RbacRoleStatus::Enable as i8);
        qb.push_and().field_in_copied("role.user_id", &user_data);
        qb.push_and().field_eq("role.app_id", app_id);
        qb.push_and()
            .field_eq("role.res_range", RbacRoleResRange::Any as i8);
        qb.push_and()
            .field_eq("role.user_range", RbacRoleUserRange::Session as i8);
        qb.push(" group by role.user_id,role.user_range,role.res_range");
        qb.push(" ) union all (");
        qb.push(format!(
            "select role.user_id,role.user_range,role.res_range
            from {} as role on perm.role_id=role.id
            join {} as role_user on role_user.role_id=role.id",
            RbacRoleModel::table_name(),
            RbacRoleUserModel::table_name(),
        ));
        qb.push_where();
        qb.field_eq("role.status", RbacRoleStatus::Enable as i8);
        qb.push_and().field_in_copied("role.user_id", &user_data);
        qb.push_and().field_eq("role.app_id", app_id);
        qb.push_and()
            .field_eq("role.res_range", RbacRoleResRange::Any as i8);
        qb.push_and()
            .field_eq("role.user_range", RbacRoleUserRange::Custom as i8);
        qb.push_and()
            .field_eq("role_user.status", RbacRoleUserStatus::Enable as i8);
        qb.push_and().push("(role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW())) group by role.user_id,role.res_range,role.user_range ");
        qb.push(")) as t");
        qb
    }
}

#[derive(Serialize)]
pub struct AccessResUserData {
    //user_id=0
    pub exist_system_session_exclude: bool, //role list -> 可通过 find_role_list_from_res 返回角色列表
    pub exist_system_session_include: bool, //role list -> 可通过 find_role_list_from_res 返回角色列表
    pub exist_system_user_exclude: bool, //user role list -> 可通过 find_user_list_from_res 返回用户列表
    pub exist_system_user_include: bool, //user role list -> 可通过 find_user_list_from_res 返回用户列表
    //user_id>0
    pub exist_self_session_exclude: bool, //role list -> 可通过 find_role_list_from_res 返回角色列表
    pub exist_self_session_include: bool, //role list -> 可通过 find_role_list_from_res 返回角色列表
    pub exist_self_user_exclude: bool, //user role list -> 可通过 find_user_list_from_res 返回用户列表
    pub exist_self_user_include: bool, //user role list -> 可通过 find_user_list_from_res 返回用户列表
}

impl RbacAccess {
    //获取配置为 禁止访问资源或授权访问 的 角色或用户信息
    //配置方: 系统或特定用户
    //资源范围: 指定资源
    //返回 可访问或禁止访问 这个资源的 角色详细
    pub async fn find_user_data_from_res(
        &self,
        user_id: u64,   //资源用户ID
        app_id: u64,    //用户下的APP,可为0
        res_type: &str, //资源类型
        res_data: &str, //资源数据
        op_key: &str,   //授权操作结构列表
    ) -> RbacResult<AccessResUserData> {
        let mut res_access = AccessResUserData {
            exist_system_session_include: false,
            exist_system_user_include: false,
            exist_system_user_exclude: false,
            exist_self_user_include: false,
            exist_system_session_exclude: false,
            exist_self_user_exclude: false,
            exist_self_session_exclude: false,
            exist_self_session_include: false,
        };
        match self
            .res
            .find_one_by_info(&ResInfo {
                res_type,
                res_data,
                user_id,
                app_id,
            })
            .await
        {
            Ok(res_row) => match self
                .op
                .find_one_by_info(&OpInfo {
                    op_key,
                    user_id,
                    app_id,
                })
                .await
            {
                Ok(op_row) => {
                    let mut qb = QueryBuilder::<MySql>::new("select * from ((");
                    qb.push(format!(
                        "select role.user_id,role.user_range,role.res_range
                            from {} as perm
                            join {} as role on perm.role_id=role.id",
                        RbacPermModel::table_name(),
                        RbacRoleModel::table_name(),
                    ));
                    qb.push_where().field_eq("perm.res_id", res_row.id);
                    qb.push_and().field_eq("perm.op_id", op_row.id);
                    qb.push_and()
                        .field_eq("perm.status", RbacPermStatus::Enable as i8);
                    qb.push_and()
                        .field_eq("role.status", RbacRoleStatus::Enable as i8);
                    qb.push_and()
                        .field_in_copied("role.user_id", &[user_id, 0u64]);
                    qb.push_and().field_in_copied(
                        "role.res_range",
                        &[
                            RbacRoleResRange::Exclude as i8,
                            RbacRoleResRange::Include as i8,
                        ],
                    );
                    qb.push_and()
                        .field_eq("role.user_range", RbacRoleUserRange::Session as i8);
                    qb.push(" group by role.user_id,role.user_range,role.res_range");
                    qb.push(" ) union all (");
                    qb.push(format!(
                        "select role.user_id,role.user_range,role.res_range
                            from {} as perm
                            join {} as role on perm.role_id=role.id
                            join {} as role_user on role_user.role_id=role.id",
                        RbacPermModel::table_name(),
                        RbacRoleModel::table_name(),
                        RbacRoleUserModel::table_name(),
                    ));
                    qb.push_where().field_eq("perm.res_id", res_row.id);
                    qb.push_and().field_eq("perm.op_id", op_row.id);
                    qb.push_and()
                        .field_eq("perm.status", RbacPermStatus::Enable as i8);
                    qb.push_and()
                        .field_eq("role.status", RbacRoleStatus::Enable as i8);
                    qb.push_and()
                        .field_in_copied("role.user_id", &[user_id, 0u64]);
                    qb.push_and().field_in_copied(
                        "role.res_range",
                        &[
                            RbacRoleResRange::Exclude as i8,
                            RbacRoleResRange::Include as i8,
                        ],
                    );
                    qb.push_and()
                        .field_eq("role.user_range", RbacRoleUserRange::Custom as i8);
                    qb.push_and()
                        .field_eq("role_user.status", RbacRoleUserStatus::Enable as i8);
                    qb.push_and().push("(role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW())) group by role.user_id,role.res_range,role.user_range ");
                    qb.push(")) as t");
                    let data = qb
                        .build_query_as::<(u64, i8, i8)>()
                        .fetch_all(&self.db)
                        .await?;
                    for (db_user_id, db_user_range, db_res_range) in data {
                        if RbacRoleResRange::Exclude.eq(db_res_range) {
                            if db_user_id == 0 {
                                if RbacRoleUserRange::Session.eq(db_user_range) {
                                    res_access.exist_system_session_exclude = true;
                                }
                                if RbacRoleUserRange::Custom.eq(db_user_range) {
                                    res_access.exist_system_user_exclude = true;
                                }
                            } else {
                                if RbacRoleUserRange::Session.eq(db_user_range) {
                                    res_access.exist_self_session_exclude = true;
                                }
                                if RbacRoleUserRange::Custom.eq(db_user_range) {
                                    res_access.exist_self_user_exclude = true;
                                }
                            }
                        } else if RbacRoleResRange::Include.eq(db_res_range) {
                            if db_user_id == 0 {
                                if RbacRoleUserRange::Session.eq(db_user_range) {
                                    res_access.exist_system_session_include = true;
                                }
                                if RbacRoleUserRange::Custom.eq(db_user_range) {
                                    res_access.exist_system_user_include = true;
                                }
                            } else {
                                if RbacRoleUserRange::Session.eq(db_user_range) {
                                    res_access.exist_self_session_include = true;
                                }
                                if RbacRoleUserRange::Custom.eq(db_user_range) {
                                    res_access.exist_self_user_include = true;
                                }
                            }
                        }
                    }
                    Ok(res_access)
                }
                Err(RbacError::Sqlx(sqlx::Error::RowNotFound)) => Ok(res_access),
                Err(err) => Err(err),
            },
            Err(RbacError::Sqlx(sqlx::Error::RowNotFound)) => Ok(res_access),
            Err(err) => Err(err),
        }
    }
}

#[derive(Serialize)]
pub struct AccessResUserRow {
    pub role_id: u64,
    pub role_user_id: u64, //0 为系统
    pub role_key: String,
    pub role_name: String,
    pub res_range: i8, //include or exclude
    pub user_id: u64,
    pub timeout: u64,
}

pub struct CustomUserListResData<'t> {
    pub user_id: u64, //资源用户ID
    pub app_id: u64,
    pub res_type: &'t str, //资源类型
    pub res_data: &'t str, //资源数据
    pub op_key: &'t str,   //授权操作结构列表,
    pub res_range_exclude: bool,
    pub res_range_any: bool,
    pub res_range_include: bool,
    pub is_system: bool,
    pub is_self: bool,
}

impl RbacAccess {
    fn find_custom_user_list_sql_from_res(
        &self,
        param: &CustomUserListResData<'_>,
        field: &str,
        qb: &mut QueryBuilder<'_, MySql>,
    ) -> RbacResult<bool> {
        let mut uid = vec![];
        if param.is_self {
            uid.push(param.user_id);
        }
        if param.is_system {
            uid.push(0u64);
        }
        if uid.is_empty() {
            return Ok(false);
        }
        let mut first = true;
        if param.res_range_any {
            if !first {
                qb.push(" union all ");
            }
            first = false;
            qb.push(format!(
                "(select
                {}
                from {} as role
                join {} as role_user on role_user.role_id=role.id",
                field,
                RbacRoleModel::table_name(),
                RbacRoleUserModel::table_name(),
            ));
            qb.push_where()
                .field_eq("role.status", RbacRoleStatus::Enable as i8);
            qb.push_and()
                .field_eq("role_user.status", RbacRoleUserStatus::Enable as i8);
            qb.push_and().field_in_copied("role.user_id", &uid);
            qb.push_and().field_eq("role.app_id", param.app_id);
            qb.push_and()
                .field_eq("role.res_range", RbacRoleResRange::Any as i8);
            qb.push_and()
                .field_eq("role.user_range", RbacRoleUserRange::Custom as i8);
            qb.push(" )");
        }
        if param.res_range_exclude {
            let op_key = string_clear(
                param.op_key,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            let res_type = string_clear(
                param.res_type,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            let res_data = string_clear(
                param.res_data,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            if !first {
                qb.push(" union all ");
            }
            first = false;
            qb.push(format!(
                "(select
                {}
                from {} as role
                join {} as role_user on role_user.role_id=role.id
                join {} as perm on perm.role_id=role.id
                join {} as op on op.id=perm.op_id
                join {} as res on res.id=perm.res_id",
                field,
                RbacRoleModel::table_name(),
                RbacRoleUserModel::table_name(),
                RbacPermModel::table_name(),
                RbacOpModel::table_name(),
                RbacResModel::table_name(),
            ));
            qb.push_where()
                .field_eq("role.status", RbacRoleStatus::Enable as i8);
            qb.push_and()
                .field_eq("role_user.status", RbacRoleUserStatus::Enable as i8);
            qb.push_and()
                .field_eq("perm.status", RbacPermStatus::Enable as i8);
            qb.push_and()
                .field_eq("op.status", RbacOpStatus::Enable as i8);
            qb.push_and()
                .field_eq("res.status", RbacResStatus::Enable as i8);
            qb.push_and().field_in_copied("role.user_id", &uid);
            qb.push_and().field_eq("role.app_id", param.app_id);
            qb.push_and()
                .field_eq("role.res_range", RbacRoleResRange::Exclude as i8);
            qb.push_and()
                .field_eq("role.user_range", RbacRoleUserRange::Custom as i8);
            qb.push_and().field_eq("op.op_key", op_key);
            qb.push_and().field_eq("res.res_type", res_type);
            qb.push_and().field_eq("res.res_data", res_data);
            qb.push(")");
        }
        if param.res_range_include {
            let op_key = string_clear(
                param.op_key,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            let res_type = string_clear(
                param.res_type,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            let res_data = string_clear(
                param.res_data,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            if !first {
                qb.push(" union all ");
            }
            first = false;
            qb.push(format!(
                "(select
                {}
                from {} as role
                join {} as role_user on role_user.role_id=role.id
                join {} as perm on perm.role_id=role.id
                join {} as op on op.id=perm.op_id
                join {} as res on res.id=perm.res_id",
                field,
                RbacRoleModel::table_name(),
                RbacRoleUserModel::table_name(),
                RbacPermModel::table_name(),
                RbacOpModel::table_name(),
                RbacResModel::table_name(),
            ));
            qb.push_where()
                .field_eq("role.status", RbacRoleStatus::Enable as i8);
            qb.push_and()
                .field_eq("role_user.status", RbacRoleUserStatus::Enable as i8);
            qb.push_and()
                .field_eq("perm.status", RbacPermStatus::Enable as i8);
            qb.push_and()
                .field_eq("op.status", RbacOpStatus::Enable as i8);
            qb.push_and()
                .field_eq("res.status", RbacResStatus::Enable as i8);
            qb.push_and().field_in_copied("role.user_id", &uid);
            qb.push_and().field_eq("role.app_id", param.app_id);
            qb.push_and()
                .field_eq("role.res_range", RbacRoleResRange::Include as i8);
            qb.push_and()
                .field_eq("role.user_range", RbacRoleUserRange::Custom as i8);
            qb.push_and().field_eq("op.op_key", op_key);
            qb.push_and().field_eq("res.res_type", res_type);
            qb.push_and().field_eq("res.res_data", res_data);
            qb.push(")");
        }
        Ok(!first)
    }

    //获取 系统或特定用户 指定资源 的 被授权可访问用户列表
    //已配置的特定用户
    pub async fn find_custom_user_list_from_res(
        &self,
        param: &CustomUserListResData<'_>,
        // user_id: u64, //资源用户ID
        // app_id: u64,
        // res_type: &str, //资源类型
        // res_data: &str, //资源数据
        // op_key: &str,   //授权操作结构列表,
        // res_range_exclude: bool,
        // res_range_any: bool,
        // res_range_include: bool,
        // is_system: bool,
        // is_self: bool,
        page: &OffsetPageParam,
    ) -> RbacResult<Vec<AccessResUserRow>> {
        let field = r#"
        role.id as role_id,
               role.user_id as role_user_id,
               role.role_key as role_key,
               role.role_name as role_name,
               role.res_range as res_range,
               role_user.user_id as user_id,
               role_user.timeout as timeout
       "#;
        let mut qb = QueryBuilder::<MySql>::new("select (select * (");
        let has_data = self.find_custom_user_list_sql_from_res(param, field, &mut qb)?;
        if !has_data {
            return Ok(vec![]);
        }
        qb.push(") as tmp) order by res_range asc");
        page.push_limit(&mut qb);
        Ok(qb
            .build()
            .try_map(|row: sqlx::mysql::MySqlRow| {
                Ok(AccessResUserRow {
                    role_id: row.try_get::<u64, &str>("role_id").unwrap_or_default(),
                    role_user_id: row.try_get::<u64, &str>("role_user_id").unwrap_or_default(),
                    role_key: row.try_get::<String, &str>("role_key").unwrap_or_default(),
                    role_name: row.try_get::<String, &str>("role_name").unwrap_or_default(),
                    res_range: row.try_get::<i8, &str>("res_range").unwrap_or_default(),
                    user_id: row.try_get::<u64, &str>("user_id").unwrap_or_default(),
                    timeout: row.try_get::<u64, &str>("timeout").unwrap_or_default(),
                })
            })
            .fetch_all(&self.db)
            .await?)
    }
    //获取 系统或特定用户 指定资源 的 被授权可访问用户列表
    //已配置的特定用户
    pub async fn find_custom_user_count_from_res(
        &self,
        param: &CustomUserListResData<'_>,
        // user_id: u64, //资源用户ID
        // app_id: u64,
        // res_type: &str, //资源类型
        // res_data: &str, //资源数据
        // op_key: &str,   //授权操作结构列表,
        // res_range_exclude: bool,
        // res_range_any: bool,
        // res_range_include: bool,
        // is_system: bool,
        // is_self: bool,
    ) -> RbacResult<i64> {
        let field = r#" count(*) as total "#;
        let mut qb = QueryBuilder::<MySql>::new("select sum(total) from (");
        let has_data = self.find_custom_user_list_sql_from_res(param, field, &mut qb)?;
        if !has_data {
            return Ok(0);
        }
        qb.push(") as tmp");
        Ok(qb.build_query_scalar::<i64>().fetch_one(&self.db).await?)
    }
}

#[derive(Serialize)]
pub struct AccessResRoleRow {
    pub role_id: u64,
    pub role_user_id: u64, //0 为系统
    pub role_key: String,
    pub role_name: String,
    pub res_range: i8,  //exclude include any
    pub user_range: i8, //session any logged
}

pub struct SessionUserListResData<'t> {
    pub user_id: u64, //资源用户ID
    pub app_id: u64,
    pub res_type: &'t str, //资源类型
    pub res_data: &'t str, //资源数据
    pub op_key: &'t str,   //授权操作结构列表
    pub res_range_exclude: bool,
    pub res_range_any: bool,
    pub res_range_include: bool,
    pub is_system: bool,
    pub is_self: bool,
}

impl RbacAccess {
    //获取 系统或特定用户 指定资源 的 被授权可访问角色列表SQL
    //会话角色的角色列表SQL
    fn find_session_role_list_sql_from_res(
        &self,
        param: &SessionUserListResData<'_>,
        field: &str,
        qb: &mut QueryBuilder<'_, MySql>,
    ) -> RbacResult<bool> {
        let mut uid = vec![];
        if param.is_self {
            uid.push(param.user_id);
        }
        if param.is_system {
            uid.push(0u64);
        }
        if uid.is_empty() {
            return Ok(false);
        }

        let mut first = true;
        if param.res_range_any {
            if !first {
                qb.push(" union all ");
            }
            first = false;
            qb.push(format!(
                "(select
               {}
               from {} as role",
                field,
                RbacRoleModel::table_name(),
            ));
            qb.push_where()
                .field_eq("role.status", RbacRoleStatus::Enable as i8);
            qb.push_and().field_in_copied("role.user_id", &uid);
            qb.push_and().field_eq("role.app_id", param.app_id);
            qb.push_and()
                .field_eq("role.res_range", RbacRoleResRange::Any as i8);
            qb.push_and()
                .field_eq("role.user_range", RbacRoleUserRange::Session as i8);
            qb.push(" )");
        }
        if param.res_range_exclude {
            let op_key = string_clear(
                param.op_key,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            let res_type = string_clear(
                param.res_type,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            let res_data = string_clear(
                param.res_data,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            if !first {
                qb.push(" union all ");
            }
            first = false;
            qb.push(format!(
                "(select
               {}
               from {} as role
               join {} as perm on perm.role_id=role.id
               join {} as op on op.id=perm.op_id
               join {} as res on res.id=perm.res_id",
                field,
                RbacRoleModel::table_name(),
                RbacPermModel::table_name(),
                RbacOpModel::table_name(),
                RbacResModel::table_name(),
            ));
            qb.push_where()
                .field_eq("role.status", RbacRoleStatus::Enable as i8);
            qb.push_and()
                .field_eq("perm.status", RbacPermStatus::Enable as i8);
            qb.push_and()
                .field_eq("op.status", RbacOpStatus::Enable as i8);
            qb.push_and()
                .field_eq("res.status", RbacResStatus::Enable as i8);
            qb.push_and().field_in_copied("role.user_id", &uid);
            qb.push_and().field_eq("role.app_id", param.app_id);
            qb.push_and()
                .field_eq("role.res_range", RbacRoleResRange::Exclude as i8);
            qb.push_and()
                .field_eq("role.user_range", RbacRoleUserRange::Session as i8);
            qb.push_and().field_eq("op.op_key", op_key);
            qb.push_and().field_eq("res.res_type", res_type);
            qb.push_and().field_eq("res.res_data", res_data);
            qb.push(")");
        }
        if param.res_range_include {
            let op_key = string_clear(
                param.op_key,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            let res_type = string_clear(
                param.res_type,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            let res_data = string_clear(
                param.res_data,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            if !first {
                qb.push(" union all ");
            }
            first = false;
            qb.push(format!(
                "(select
               {}
               from {} as role
               join {} as perm on perm.role_id=role.id
               join {} as op on op.id=perm.op_id
               join {} as res on res.id=perm.res_id",
                field,
                RbacRoleModel::table_name(),
                RbacPermModel::table_name(),
                RbacOpModel::table_name(),
                RbacResModel::table_name(),
            ));
            qb.push_where()
                .field_eq("role.status", RbacRoleStatus::Enable as i8);
            qb.push_and()
                .field_eq("perm.status", RbacPermStatus::Enable as i8);
            qb.push_and()
                .field_eq("op.status", RbacOpStatus::Enable as i8);
            qb.push_and()
                .field_eq("res.status", RbacResStatus::Enable as i8);
            qb.push_and().field_in_copied("role.user_id", &uid);
            qb.push_and().field_eq("role.app_id", param.app_id);
            qb.push_and()
                .field_eq("role.res_range", RbacRoleResRange::Include as i8);
            qb.push_and()
                .field_eq("role.user_range", RbacRoleUserRange::Session as i8);
            qb.push_and().field_eq("op.op_key", op_key);
            qb.push_and().field_eq("res.res_type", res_type);
            qb.push_and().field_eq("res.res_data", res_data);
            qb.push(")");
        }
        Ok(!first)
    }
    //获取 系统或特定用户 指定资源 的 被授权可访问角色列表
    //会话角色的角色列表
    pub async fn find_session_role_list_from_res(
        &self,
        param: &SessionUserListResData<'_>,
        page: &OffsetPageParam,
    ) -> RbacResult<Vec<AccessResRoleRow>> {
        let field = r#"
            role.id as role_id,
            role.user_id as role_user_id,
            role.role_key as role_key,
            role.role_name as role_name,
            role.res_range as res_range,
            role.user_range as user_range
       "#;
        let mut qb = QueryBuilder::<MySql>::new("select (select * (");
        let has_data = self.find_session_role_list_sql_from_res(param, field, &mut qb)?;
        if !has_data {
            return Ok(vec![]);
        }
        qb.push(") as tmp) order by res_range asc");
        page.push_limit(&mut qb);
        Ok(qb
            .build()
            .try_map(|row: sqlx::mysql::MySqlRow| {
                Ok(AccessResRoleRow {
                    role_id: row.try_get::<u64, &str>("role_id").unwrap_or_default(),
                    role_user_id: row.try_get::<u64, &str>("role_user_id").unwrap_or_default(),
                    role_key: row.try_get::<String, &str>("role_key").unwrap_or_default(),
                    role_name: row.try_get::<String, &str>("role_name").unwrap_or_default(),
                    res_range: row.try_get::<i8, &str>("res_range").unwrap_or_default(),
                    user_range: row.try_get::<i8, &str>("user_range").unwrap_or_default(),
                })
            })
            .fetch_all(&self.db)
            .await?)
    }
    //会话角色的角色数量
    pub async fn find_session_role_count_from_res(
        &self,
        param: &SessionUserListResData<'_>,
    ) -> RbacResult<i64> {
        let field = r#"
            count(*) as total
       "#;
        let mut qb = QueryBuilder::<MySql>::new("select sum(total) from (");
        let has_data = self.find_session_role_list_sql_from_res(param, field, &mut qb)?;
        if !has_data {
            return Ok(0);
        }
        qb.push(") as tmp");
        Ok(qb.build_query_scalar::<i64>().fetch_one(&self.db).await?)
    }
}
