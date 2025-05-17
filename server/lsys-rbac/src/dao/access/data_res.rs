use super::{AccessSessionRole, RbacAccess};
use crate::model::RbacRoleStatus;
use crate::{
    dao::result::RbacResult,
    model::{
        RbacOpModel, RbacOpStatus, RbacPermModel, RbacPermStatus, RbacResModel, RbacResStatus,
        RbacRoleModel, RbacRoleResRange, RbacRoleUserModel, RbacRoleUserRange, RbacRoleUserStatus,
    },
};
use lsys_core::db::SqlQuote;
use lsys_core::db::{ModelTableName, SqlExpr};
use lsys_core::{sql_format, string_clear};
use lsys_core::{PageParam, StringClear, STRING_CLEAR_FORMAT};
use serde::Serialize;
use sqlx::Row;

//查询指定用户可访问资源的调用流程:

//1. 查询系统或哪些用户给指定用户授权访问的资源
//find_res_user_list_from_user 得到存在授权的用户,0表示系统

//2. 根据查询到用户资源查询具体该用户授权情况

//用户 => 查询系统或某用户的被授权信息(find_res_data_from_user->AccessUserFromRes)=>配置了被查询用户(AccessUserFromRes:user_range_custom)=>禁止访问指定资源(AccessUserFromRes:exist_exclude_res_list)
//      => find_res_list_from_user(user_range=custom,role_user_id=0,res_range=exclude)
//用户 => 查询系统或某用户的被授权信息(find_res_data_from_user->AccessUserFromRes)=>配置了被查询用户(AccessUserFromRes:user_range_custom)=>可以访问指定资源(AccessUserFromRes:exist_include_res_list)
//      => find_res_list_from_user(user_range=custom,role_user_id=0,res_range=include)
//用户 => 查询系统或某用户的被授权信息(find_res_data_from_user->AccessUserFromRes)=>配置了被查询用户(AccessUserFromRes:user_range_custom)=>可以访问任何资源(AccessUserFromRes:exist_any_res)
//      => 除了`禁止访问指定资源`外的被查询用户可以访问任意资源

//3. 查询会话角色
//用户+会话橘色 =>find_res_range_from_session_role 查询出授权访问
//当为 授权类型为include或exclude,通过 find_res_list_from_session_role 查询出详细

impl RbacAccess {
    fn find_res_user_custom_sql_from_user(
        &self,
        user_id: u64,                //访问用户ID,必须>0
        res_range: RbacRoleResRange, //RbacRoleResRange::Include RbacRoleResRange::Exclude
    ) -> String {
        match res_range {
            RbacRoleResRange::Any => {
                sql_format!(
                    "select  role.user_id
                    from {} as role 
                    join {} as role_user on role_user.role_id=role.id
                    where  role.status ={} 
                    and role.user_id>0 and role.user_range={} and role.res_range={}
                    and role_user.user_id={} and (role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW()))
                    ",
                    RbacRoleModel::table_name(),
                    RbacRoleUserModel::table_name(),
                    RbacRoleStatus::Enable as i8,
                    RbacRoleUserRange::Custom as i8,
                    res_range as i8,
                    user_id
                )
            }
            RbacRoleResRange::Exclude | RbacRoleResRange::Include => {
                sql_format!(
                    "select  role.user_id
                    from {} as role 
                    join {} as perm on role.id=perm.role_id
                    join {} as res on perm.res_id=res.id
                    join {} as op on perm.op_id=op.id
                    join {} as role_user on role_user.role_id=role.id
                    where  role.status ={} and perm.status ={} and res.status ={} and op.status ={}
                    and role.user_id>0 and role.user_range={} and role.res_range={}
                    and role_user.user_id={} and (role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW()))
                    ",
                    RbacRoleModel::table_name(),
                    RbacPermModel::table_name(),
                    RbacResModel::table_name(),
                    RbacOpModel::table_name(),
                    RbacRoleUserModel::table_name(),
                    RbacRoleStatus::Enable as i8,
                    RbacPermStatus::Enable as i8,
                    RbacResStatus::Enable as i8,
                    RbacOpStatus::Enable as i8,
                    RbacRoleUserRange::Custom as i8,
                    res_range as i8,
                    user_id
                )
            }
        }
    }
    //被指定用户授权的用户列表
    //返回中,0为系统
    pub async fn find_res_user_list_from_user(
        &self,
        user_id: u64, //访问用户ID,0 为游客
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<u64>> {
        if user_id == 0 {
            return Ok(vec![]);
        }
        let sql_arr = [
            self.find_res_user_custom_sql_from_user(user_id, RbacRoleResRange::Exclude),
            self.find_res_user_custom_sql_from_user(user_id, RbacRoleResRange::Include),
            self.find_res_user_custom_sql_from_user(user_id, RbacRoleResRange::Any),
        ];
        let mut sql = format!(
            "select DISTINCT user_id from (({})) as tmp order by user_id asc ",
            sql_arr.join(") union all (")
        );
        if let Some(pdat) = page {
            sql = format!(" {} limit {} offset {} ", sql, pdat.limit, pdat.offset)
        };
        Ok(sqlx::query_scalar::<_, u64>(&sql)
            .fetch_all(&self.db)
            .await?)
    }
    //被指定用户授权的用户数量
    pub async fn find_res_user_count_from_user(
        &self,
        user_id: u64, //访问用户ID,0 为游客
    ) -> RbacResult<i64> {
        let sql_arr = [
            self.find_res_user_custom_sql_from_user(user_id, RbacRoleResRange::Exclude),
            self.find_res_user_custom_sql_from_user(user_id, RbacRoleResRange::Include),
            self.find_res_user_custom_sql_from_user(user_id, RbacRoleResRange::Any),
        ];
        if user_id == 0 {
            return Ok(0);
        }
        let sql = format!(
            "select COUNT(DISTINCT user_id) AS total from (({})) as tmp",
            sql_arr.join(") union all (")
        );
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.db)
            .await?)
    }
}

//system
#[derive(Serialize)]
pub struct AccessUserFromRes {
    //存在指定资源列表被禁止访问[优先] find_res_list_from_user(res_range=exclude)
    pub exist_exclude_res_list: bool,
    //任何资源被访问(被禁止访问除外)
    pub exist_any_res: bool,
    //存在指定资源列表被访问 find_res_list_from_user(res_range=include)
    pub exist_include_res_list: bool,
}

impl RbacAccess {
    fn find_res_data_from_custom_user_sql(&self, user_id: u64, role_user_id: u64) -> Vec<String> {
        let sql = vec![
              // 针对特定用户配置权限
              sql_format!(
                "select  role.res_range
                from {} as role 
                join {} as role_user on role.id=role_user.role_id
                where role.status ={} and role.user_id ={}
                and role.res_range = {} and role.user_range = {}
                and role_user.status={} and role_user.user_id={} and (role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW())) limit 1",
                RbacRoleModel::table_name(),
                RbacRoleUserModel::table_name(),
                RbacRoleStatus::Enable as i8,
                role_user_id,
                RbacRoleResRange::Any as i8,
                RbacRoleUserRange::Custom as i8,
                RbacRoleUserStatus::Enable as i8,
                user_id
            ),
            sql_format!(
                "select role.res_range
                from {} as role 
                join {} as perm on role.id=perm.role_id
                join {} as role_user on role.id=role_user.role_id
                where role.status ={} and role.user_id ={}
                and role.res_range = {} and role.user_range = {}
                and role_user.status={} and role_user.user_id={}  and (role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW()))
                limit 1",
                RbacRoleModel::table_name(),
                RbacPermModel::table_name(),
                RbacRoleUserModel::table_name(),
                RbacRoleStatus::Enable as i8,
                role_user_id,
                RbacRoleResRange::Exclude as i8,
                RbacRoleUserRange::Custom as i8,
                RbacRoleUserStatus::Enable as i8,
                user_id
            ),
            sql_format!(
                "select role.res_range
                from {} as role 
                join {} as perm on role.id=perm.role_id
                join {} as role_user on role.id=role_user.role_id
                where role.status ={} and role.user_id = {}
                and role.res_range ={} and role.user_range = {}
                and role_user.status={} and role_user.user_id={}  and (role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW()))
                limit 1",
                RbacRoleModel::table_name(),
                RbacPermModel::table_name(),
                RbacRoleUserModel::table_name(),
                RbacRoleStatus::Enable as i8,
                role_user_id,
                RbacRoleResRange::Include as i8,
                RbacRoleUserRange::Custom as i8,
                RbacRoleUserStatus::Enable as i8,
                user_id
            ),
        ];
        sql
    }
    //列出所有可以访问的资源,包含系统资源跟用户资源
    //不包含会话角色 RbacRoleUserRange::Session,会话角色获取对应被授权资源参见 find_res_range_from_session_role
    pub async fn find_res_data_from_custom_user(
        &self,
        role_user_id: u64,   //0为查询系统资源,>0为某用户资源
        access_user_id: u64, //访问用户ID,0 为游客
    ) -> RbacResult<AccessUserFromRes> {
        let mut user_range_custom = AccessUserFromRes {
            exist_any_res: false,
            exist_exclude_res_list: false,
            exist_include_res_list: false,
        };
        if access_user_id == 0 {
            return Ok(user_range_custom);
        }
        let sql = self.find_res_data_from_custom_user_sql(access_user_id, role_user_id);
        let data = sqlx::query_scalar::<_, i8>(&format!("({})", sql.join(" ) union all (")))
            .fetch_all(&self.db)
            .await?;
        for db_res_range in data {
            if RbacRoleResRange::Any.eq(db_res_range) {
                user_range_custom.exist_any_res = true;
            } else if RbacRoleResRange::Exclude.eq(db_res_range) {
                user_range_custom.exist_exclude_res_list = true;
            } else if RbacRoleResRange::Include.eq(db_res_range) {
                user_range_custom.exist_include_res_list = true;
            }
        }
        Ok(user_range_custom)
    }
}

#[derive(Serialize)]
pub struct AccessPermRow {
    pub res_user_id: u64,
    pub res_type: String,
    pub res_data: String,
    pub res_name: String,
    pub op_key: String,
    pub op_name: String,
    pub perm_time: u64,
    pub perm_user_id: u64,
}

impl RbacAccess {
    fn res_list_sql_field(&self) -> &str {
        r#"
            res.user_id as res_user_id,
            res.res_type as res_type,
            res.res_data as res_data,
            res.res_name as res_name,
            op.op_key as op_key,
            op.op_name as op_name,
            op.change_time as perm_time,
            op.change_user_id as perm_user_id,
        "#
    }
    fn res_list_from_mysql_row(&self, row: sqlx::mysql::MySqlRow) -> AccessPermRow {
        AccessPermRow {
            res_user_id: row.try_get::<u64, &str>("res_user_id").unwrap_or_default(),
            res_type: row.try_get::<String, &str>("res_type").unwrap_or_default(),
            res_data: row.try_get::<String, &str>("res_data").unwrap_or_default(),
            res_name: row.try_get::<String, &str>("res_name").unwrap_or_default(),
            op_key: row.try_get::<String, &str>("op_key").unwrap_or_default(),
            op_name: row.try_get::<String, &str>("op_name").unwrap_or_default(),
            perm_time: row.try_get::<u64, &str>("perm_time").unwrap_or_default(),
            perm_user_id: row.try_get::<u64, &str>("perm_user_id").unwrap_or_default(),
        }
    }
}

impl RbacAccess {
    fn find_res_custom_sql_from_user(
        &self,
        user_id: u64,                //访问用户ID,0 为游客
        role_user_id: u64,           //指定角色用户,0为系统
        role_app_id: Option<u64>,    //应用ID
        res_range: RbacRoleResRange, //RbacRoleResRange::Include RbacRoleResRange::Exclude
        field: &str,
    ) -> String {
        sql_format!(
            "select {}
            from {} as role 
            join {} as perm on role.id=perm.role_id
            join {} as res on perm.res_id=res.id
            join {} as op on perm.op_id=op.id
            join {} as role_user on role_user.role_id=role.id
            where  role.status ={} and perm.status ={} and res.status ={} and op.status ={}
            and role.user_id={} {} and role.user_range={} and role.res_range={}
            and role_user.user_id={} and (role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW()))
            ",
            field,
            RbacRoleModel::table_name(),
            RbacPermModel::table_name(),
            RbacResModel::table_name(),
            RbacOpModel::table_name(),
            RbacRoleUserModel::table_name(),
            RbacRoleStatus::Enable as i8,
            RbacPermStatus::Enable as i8,
            RbacResStatus::Enable as i8,
            RbacOpStatus::Enable as i8,
            SqlExpr(match role_app_id{
                Some(app_id)=>sql_format!(" and role.app_id={}",app_id),
                None=>"".to_string()
            }),
            role_user_id,
            RbacRoleUserRange::Custom as i8,
            res_range as i8,
            user_id
        )
    }
    //被用户或系统授权的授权数量
    pub async fn find_res_count_from_custom_user(
        &self,
        user_id: u64,                //访问用户ID,0 为游客
        role_user_id: u64,           //指定角色用户,0为系统
        role_app_id: Option<u64>,    //应用ID
        res_range: RbacRoleResRange, //RbacRoleResRange::Exclude | RbacRoleResRange::Include
    ) -> RbacResult<i64> {
        match res_range {
            RbacRoleResRange::Exclude | RbacRoleResRange::Include => {
                let sql = if user_id == 0 {
                    return Ok(0);
                } else {
                    self.find_res_custom_sql_from_user(
                        user_id,
                        role_user_id,
                        role_app_id,
                        res_range,
                        "count(*)",
                    )
                };
                Ok(sqlx::query_scalar::<_, i64>(&sql)
                    .fetch_one(&self.db)
                    .await?)
            }
            RbacRoleResRange::Any => Ok(0),
        }
    }
    //被用户或系统授权的授权列表
    pub async fn find_res_list_from_custom_user(
        &self,
        user_id: u64,                //访问用户ID,0 为游客
        role_user_id: u64,           //指定角色用户,0为系统
        role_app_id: Option<u64>,    //应用ID
        res_range: RbacRoleResRange, //RbacRoleResRange::Include RbacRoleResRange::Exclude
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<AccessPermRow>> {
        match res_range {
            RbacRoleResRange::Exclude | RbacRoleResRange::Include => {
                let mut sql = if user_id == 0 {
                    return Ok(vec![]);
                } else {
                    self.find_res_custom_sql_from_user(
                        user_id,
                        role_user_id,
                        role_app_id,
                        res_range,
                        self.res_list_sql_field(),
                    )
                };
                if let Some(pdat) = page {
                    sql = format!(
                        "order by perm.id desc {} limit {} offset {} ",
                        sql, pdat.limit, pdat.offset
                    )
                };
                Ok(sqlx::query(&sql)
                    .try_map(|row| Ok(self.res_list_from_mysql_row(row)))
                    .fetch_all(&self.db)
                    .await?)
            }
            RbacRoleResRange::Any => Ok(vec![]),
        }
    }
}

impl RbacAccess {
    //列出会话角色可访问资源范围
    pub async fn find_res_range_from_session_role(
        &self,
        //该数据直接映射为对应角色
        role_data: &AccessSessionRole<'_>,
    ) -> RbacResult<RbacRoleResRange> {
        let role_key = string_clear(
            role_data.role_key,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(33),
        );
        if role_key.is_empty() {
            return Err(sqlx::Error::RowNotFound.into());
        }

        let sql = sql_format!(
            "select role.res_range
            from {} as role 
            where role.status ={} and role.role_key={} and role.user_id={} and role.user_range = {} limit 1",
            RbacRoleModel::table_name(),
            RbacRoleStatus::Enable as i8,
            role_key ,
            role_data.user_id,
            RbacRoleUserRange::Session as i8,
        );
        let res_range = sqlx::query_scalar::<_, i8>(&sql)
            .fetch_one(&self.db)
            .await?;
        Ok(RbacRoleResRange::try_from(res_range)?)
    }
    fn find_res_sql_from_session_role(
        &self,
        role_data: &AccessSessionRole,
        res_range: RbacRoleResRange,
        field: &str,
    ) -> String {
        let role_key = string_clear(
            role_data.role_key,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(33),
        );
        sql_format!(
            "select {}
            from {} as role 
            join {} as perm on role.id=perm.role_id
            join {} as res on perm.res_id=res.id
            join {} as op on perm.op_id=op.id
            where 
                role.status ={} and role.role_key={} and role.res_range={} and role.user_id={} and role.user_range = {}
                and perm.status ={} and res.status ={} and op.status ={}
            ",
            field,
            RbacRoleModel::table_name(),
            RbacPermModel::table_name(),
            RbacResModel::table_name(),
            RbacOpModel::table_name(),
            RbacRoleStatus::Enable as i8,
            role_key,
            res_range as i8,
            role_data.user_id,
            RbacRoleUserRange::Session as i8,
            RbacPermStatus::Enable as i8,
            RbacResStatus::Enable as i8,
            RbacOpStatus::Enable as i8,
        )
    }
    //列出会话角色可访问授权数量
    pub async fn find_res_count_from_session_role(
        &self,
        //该数据直接映射为对应角色
        role_data: &AccessSessionRole<'_>,
        res_range: RbacRoleResRange,
    ) -> RbacResult<i64> {
        let sql = self.find_res_sql_from_session_role(role_data, res_range, "count(*)");
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.db)
            .await?)
    }
    //列出会话角色可访问授权数据
    pub async fn find_res_list_from_session_role(
        &self,
        //该数据直接映射为对应角色
        role_data: &AccessSessionRole<'_>,
        res_range: RbacRoleResRange,
        page: Option<&PageParam>,
    ) -> RbacResult<Vec<AccessPermRow>> {
        let mut sql =
            self.find_res_sql_from_session_role(role_data, res_range, self.res_list_sql_field());
        if let Some(pdat) = page {
            sql = format!(
                "order by perm.id desc {} limit {} offset {} ",
                sql, pdat.limit, pdat.offset
            )
        };
        Ok(sqlx::query(&sql)
            .try_map(|row| Ok(self.res_list_from_mysql_row(row)))
            .fetch_all(&self.db)
            .await?)
    }
}
