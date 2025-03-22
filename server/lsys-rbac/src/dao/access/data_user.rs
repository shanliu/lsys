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
use lsys_core::db::ModelTableName;
use lsys_core::db::SqlQuote;
use lsys_core::sql_format;
use lsys_core::PageParam;
use serde::Serialize;
use sqlx::Row;
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
        let sql = self.user_data_pub_sql(user_id, app_id).await;
        let data = sqlx::query_as::<_, (u64, i8, i8)>(&format!("({})", sql.join(" ) union all (")))
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
    async fn user_data_pub_sql(&self, user_id: u64, app_id: u64) -> [String; 2] {
        let mut user_data = vec![0];
        if user_id > 0 {
            user_data.push(user_id);
        }
        [sql_format!(
            "select role.user_id,role.user_range,role.res_range
            from {} as role 
            where role.status ={} and role.user_id in ({}) and role.app_id={}
            and role.res_range = {} and role.user_range = {}
            group by role.user_id,role.user_range,role.res_range",
            RbacRoleModel::table_name(),
            RbacRoleStatus::Enable as i8,
            &user_data,
            app_id,
            RbacRoleResRange::Any as i8,
            RbacRoleUserRange::Session as i8,
        ),sql_format!(
            "select role.user_id,role.user_range,role.res_range
            from {} as role on perm.role_id=role.id
            join {} as role_user on role_user.role_id=role.id
            where  role.status ={} and role.user_id in ({}) and role.app_id={}
            and role.res_range = {} and role.user_range = {} 
            and role_user.status={} and (role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW()))
            group by role.user_id,role.res_range,role.user_range
            ",
            RbacRoleModel::table_name(),
            RbacRoleUserModel::table_name(),
            RbacRoleStatus::Enable as i8,
            &user_data,
            app_id,
            RbacRoleResRange::Any as i8,
            RbacRoleUserRange::Custom as i8,
            RbacRoleUserStatus::Enable as i8,
        )]
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
                    let sql = [sql_format!(
                            "select role.user_id,role.user_range,role.res_range
                            from {} as perm  
                            join {} as role on perm.role_id=role.id
                            where perm.res_id ={} and perm.op_id={} 
                            and perm.status={} and and role.status ={} and role.user_id in ({})
                            and role.res_range in ({}) and role.user_range = {}
                            group by role.user_id,role.user_range,role.res_range",
                            RbacPermModel::table_name(),
                            RbacRoleModel::table_name(),
                            res_row.id,
                            op_row.id,
                            RbacPermStatus::Enable as i8,
                            RbacRoleStatus::Enable as i8,
                            &[user_id,0],
                            &[RbacRoleResRange::Exclude as i8,RbacRoleResRange::Include as i8],
                            RbacRoleUserRange::Session as i8,
                        ),sql_format!(
                            "select role.user_id,role.user_range,role.res_range
                            from {} as perm 
                            join {} as role on perm.role_id=role.id
                            join {} as role_user on role_user.role_id=role.id
                            where perm.res_id ={} and perm.op_id={} 
                            and perm.status={} and role.status ={} and role.user_id in ({})
                            and role.res_range in ({}) and role.user_range = {} 
                            and role_user.status={} and (role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW()))
                            group by role.user_id,role.res_range,role.user_range
                            ",
                            RbacPermModel::table_name(),
                            RbacRoleModel::table_name(),
                            RbacRoleUserModel::table_name(),
                            res_row.id,
                            op_row.id,
                            RbacPermStatus::Enable as i8,
                            RbacRoleStatus::Enable as i8,
                            &[user_id,0],
                            &[RbacRoleResRange::Exclude as i8,RbacRoleResRange::Include as i8],
                            RbacRoleUserRange::Custom as i8,
                            RbacRoleUserStatus::Enable as i8,
                        )];
                    let data = sqlx::query_as::<_, (u64, i8, i8)>(&format!(
                        "({})",
                        sql.join(" ) union all (")
                    ))
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
        field: &str,
    ) -> RbacResult<Vec<String>> {
        let mut uid = vec![];
        if param.is_self {
            uid.push(param.user_id);
        }
        if param.is_system {
            uid.push(0);
        }
        if uid.is_empty() {
            return Ok(vec![]);
        }
        let mut sql = vec![];
        if param.res_range_any {
            sql.push(sql_format!(
                "select 
                {}
                from {} as role  
                join {} as role_user on role_user.role_id=role.id
                where  role.status ={} and role_user.status={} and role.user_id in ({}) and role.app_id={}
                and role.res_range = {} and role.user_range = {} ",
                field,
                RbacRoleModel::table_name(),
                RbacRoleUserModel::table_name(),
                RbacRoleStatus::Enable as i8,
                RbacRoleUserStatus::Enable as i8,
                &uid,
                param.app_id,
                RbacRoleResRange::Any as i8,
                RbacRoleUserRange::Custom as i8,
            ));
        }
        if param.res_range_exclude {
            sql.push(sql_format!(
                "select 
                {}
                from {} as role  
                join {} as role_user on role_user.role_id=role.id
                join {} as perm on perm.role_id=role.id
                join {} as op on op.id=perm.op_id
                join {} as res on res.id=perm.res_id
                where  role.status ={} and role_user.status={} and perm.status={} and op.status={} and res.status={} 
                and role.user_id in ({}) and role.app_id={} and role.res_range = {} and role.user_range = {} 
                and op.op_key={} and res.res_type={} and res.res_data={}",
                field,
                RbacRoleModel::table_name(),
                RbacRoleUserModel::table_name(),
                RbacPermModel::table_name(),
                RbacOpModel::table_name(),
                RbacResModel::table_name(),
                RbacRoleStatus::Enable as i8,
                RbacRoleUserStatus::Enable as i8,
                RbacPermStatus::Enable as i8,
                RbacOpStatus::Enable as i8,
                RbacResStatus::Enable as i8,
                &uid,
                param.app_id,
                RbacRoleResRange::Exclude as i8,
                RbacRoleUserRange::Custom as i8,
                param.op_key,
                param.res_type,
                param.res_data
            ));
        }
        if param.res_range_include {
            sql.push(sql_format!(
                "select 
                {}
                from {} as role  
                join {} as role_user on role_user.role_id=role.id
                join {} as perm on perm.role_id=role.id
                join {} as op on op.id=perm.op_id
                join {} as res on res.id=perm.res_id
                where  role.status ={} and role_user.status={} and perm.status={} and op.status={} and res.status={} 
                and role.user_id in ({})  and role.app_id={} and role.res_range = {} and role.user_range = {} 
                and op.op_key={} and res.res_type={} and res.res_data={}",
                field,
                RbacRoleModel::table_name(),
                RbacRoleUserModel::table_name(),
                RbacPermModel::table_name(),
                RbacOpModel::table_name(),
                RbacResModel::table_name(),
                RbacRoleStatus::Enable as i8,
                RbacRoleUserStatus::Enable as i8,
                RbacPermStatus::Enable as i8,
                RbacOpStatus::Enable as i8,
                RbacResStatus::Enable as i8,
                &uid,
                param.app_id,
                RbacRoleResRange::Include as i8,
                RbacRoleUserRange::Custom as i8,
                param.op_key,
                param.res_type,
                param.res_data
            ));
        }
        Ok(sql)
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
        page: Option<&PageParam>,
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
        let sql = self.find_custom_user_list_sql_from_res(
            param, //资源用户ID
            // app_id,
            // res_type, //资源类型
            // res_data, //资源数据
            // op_key,   //授权操作结构列表,
            // res_range_exclude,
            // res_range_any,
            // res_range_include,
            // is_system,
            // is_self,
            field,
        )?;
        if sql.is_empty() {
            return Ok(vec![]);
        }
        let mut sql = format!("select * (({})) as tmp", sql.join(") union all ("));
        if let Some(pdat) = page {
            sql = format!(
                "select ({}) order by res_range asc limit {} offset {} ",
                sql, pdat.limit, pdat.offset
            )
        };
        Ok(sqlx::query(&sql)
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
        let sql = self.find_custom_user_list_sql_from_res(param, field)?;
        if sql.is_empty() {
            return Ok(0);
        }
        let sql = format!(
            "select sum(total) from (({})) as tmp",
            sql.join(") union all (")
        );
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.db)
            .await?)
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
    ) -> RbacResult<Vec<String>> {
        let mut uid = vec![];
        if param.is_self {
            uid.push(param.user_id);
        }
        if param.is_system {
            uid.push(0);
        }
        if uid.is_empty() {
            return Ok(vec![]);
        }

        let mut sql = vec![];
        if param.res_range_any {
            sql.push(sql_format!(
                "select 
               {}
               from {} as role  
               where  role.status ={} and role.user_id in ({}) and role.app_id={}
               and role.res_range = {} and role.user_range = {} ",
                field,
                RbacRoleModel::table_name(),
                RbacRoleStatus::Enable as i8,
                &uid,
                param.app_id,
                RbacRoleResRange::Any as i8,
                RbacRoleUserRange::Session as i8,
            ));
        }
        if param.res_range_exclude {
            sql.push(sql_format!(
                "select 
               {}
               from {} as role  
               join {} as perm on perm.role_id=role.id
               join {} as op on op.id=perm.op_id
               join {} as res on res.id=perm.res_id
               where  role.status ={}  and perm.status={} and op.status={} and res.status={} 
               and role.user_id in ({}) and role.app_id={} and role.res_range = {} and role.user_range = {} 
               and op.op_key={} and res.res_type={} and res.res_data={}",
                field,
                RbacRoleModel::table_name(),
                RbacPermModel::table_name(),
                RbacOpModel::table_name(),
                RbacResModel::table_name(),
                RbacRoleStatus::Enable as i8,
                RbacPermStatus::Enable as i8,
                RbacOpStatus::Enable as i8,
                RbacResStatus::Enable as i8,
                &uid,
                param.app_id,
                RbacRoleResRange::Exclude as i8,
                RbacRoleUserRange::Session as i8,
                param.op_key,
                param.res_type,
                param.res_data
            ));
        }
        if param.res_range_include {
            sql.push(sql_format!(
                "select 
               {}
               from {} as role  
               join {} as perm on perm.role_id=role.id
               join {} as op on op.id=perm.op_id
               join {} as res on res.id=perm.res_id
               where  role.status ={}  and perm.status={} and op.status={} and res.status={} 
               and role.user_id in ({}) and role.app_id={}  and role.res_range = {} and role.user_range = {}
               and op.op_key={} and res.res_type={} and res.res_data={}",
                field,
                RbacRoleModel::table_name(),
                RbacPermModel::table_name(),
                RbacOpModel::table_name(),
                RbacResModel::table_name(),
                RbacRoleStatus::Enable as i8,
                RbacPermStatus::Enable as i8,
                RbacOpStatus::Enable as i8,
                RbacResStatus::Enable as i8,
                &uid,
                param.app_id,
                RbacRoleResRange::Include as i8,
                RbacRoleUserRange::Session as i8,
                param.op_key,
                param.res_type,
                param.res_data
            ));
        }
        Ok(sql)
    }
    //获取 系统或特定用户 指定资源 的 被授权可访问角色列表
    //会话角色的角色列表
    pub async fn find_session_role_list_from_res(
        &self,
        param: &SessionUserListResData<'_>,
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<AccessResRoleRow>> {
        let field = r#"
            role.id as role_id,
            role.user_id as role_user_id,
            role.role_key as role_key,
            role.role_name as role_name,
            role.res_range as res_range,
            role.user_range as user_range
       "#;
        let sql = self.find_session_role_list_sql_from_res(param, field)?;
        if sql.is_empty() {
            return Ok(vec![]);
        }
        let mut sql = format!("select * (({})) as tmp", sql.join(") union all ("));
        if let Some(pdat) = page {
            sql = format!(
                "select ({}) order by res_range asc limit {} offset {} ",
                sql, pdat.limit, pdat.offset
            )
        };
        Ok(sqlx::query(&sql)
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
        let sql = self.find_session_role_list_sql_from_res(param, field)?;
        if sql.is_empty() {
            return Ok(0);
        }
        let sql = format!(
            "select sum(total) from  (({})) as tmp",
            sql.join(") union all (")
        );
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.db)
            .await?)
    }
}
