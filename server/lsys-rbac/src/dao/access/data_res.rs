use super::{AccessSessionRole, RbacAccess};
use crate::model::RbacRoleStatus;
use crate::{
    dao::result::RbacResult,
    model::{
        RbacOpModel, RbacOpStatus, RbacPermModel, RbacPermStatus, RbacResModel, RbacResStatus,
        RbacRoleModel, RbacRoleResRange, RbacRoleUserModel, RbacRoleUserRange, RbacRoleUserStatus,
    },
};
use lsys_core::db::TableMeta;
use lsys_core::db::{OffsetPageParam, QueryBuilderExt};
use lsys_core::utils::{STRING_CLEAR_FORMAT, StringClear, string_clear};
use serde::Serialize;
use sqlx::{MySql, QueryBuilder, Row};

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
    fn push_res_user_custom_sql_from_user(
        qb: &mut QueryBuilder<'_, MySql>,
        user_id: u64,
        res_range: RbacRoleResRange,
    ) {
        match res_range {
            RbacRoleResRange::Any => {
                qb.push(format!(
                    "select  role.user_id
                    from {} as role
                    join {} as role_user on role_user.role_id=role.id",
                    RbacRoleModel::table_name(),
                    RbacRoleUserModel::table_name(),
                ));
                qb.push_where()
                    .field_eq("role.status", RbacRoleStatus::Enable as i8);
                qb.push_and().field_gt("role.user_id", 0);
                qb.push_and()
                    .field_eq("role.user_range", RbacRoleUserRange::Custom as i8);
                qb.push_and().field_eq("role.res_range", res_range as i8);
                qb.push_and().field_eq("role_user.user_id", user_id);
                qb.push_and().push("(");
                qb.field_eq("role_user.timeout", 0);
                qb.push_or()
                    .push("role_user.timeout >= UNIX_TIMESTAMP(NOW())");
                qb.push(")");
            }
            RbacRoleResRange::Exclude | RbacRoleResRange::Include => {
                qb.push(format!(
                    "select  role.user_id
                    from {} as role
                    join {} as perm on role.id=perm.role_id
                    join {} as res on perm.res_id=res.id
                    join {} as op on perm.op_id=op.id
                    join {} as role_user on role_user.role_id=role.id",
                    RbacRoleModel::table_name(),
                    RbacPermModel::table_name(),
                    RbacResModel::table_name(),
                    RbacOpModel::table_name(),
                    RbacRoleUserModel::table_name(),
                ));
                qb.push_where()
                    .field_eq("role.status", RbacRoleStatus::Enable as i8);
                qb.push_and()
                    .field_eq("perm.status", RbacPermStatus::Enable as i8);
                qb.push_and()
                    .field_eq("res.status", RbacResStatus::Enable as i8);
                qb.push_and()
                    .field_eq("op.status", RbacOpStatus::Enable as i8);
                qb.push_and().field_gt("role.user_id", 0);
                qb.push_and()
                    .field_eq("role.user_range", RbacRoleUserRange::Custom as i8);
                qb.push_and().field_eq("role.res_range", res_range as i8);
                qb.push_and().field_eq("role_user.user_id", user_id);
                qb.push_and().push("(");
                qb.field_eq("role_user.timeout", 0);
                qb.push_or()
                    .push("role_user.timeout >= UNIX_TIMESTAMP(NOW())");
                qb.push(")");
            }
        }
    }
    //被指定用户授权的用户列表
    //返回中,0为系统
    pub async fn find_res_user_list_from_user(
        &self,
        user_id: u64, //访问用户ID,0 为游客
        page: &OffsetPageParam,
    ) -> RbacResult<Vec<u64>> {
        if user_id == 0 {
            return Ok(vec![]);
        }
        let mut qb = QueryBuilder::<MySql>::new("select DISTINCT user_id from ((");
        Self::push_res_user_custom_sql_from_user(&mut qb, user_id, RbacRoleResRange::Exclude);
        qb.push(") union all (");
        Self::push_res_user_custom_sql_from_user(&mut qb, user_id, RbacRoleResRange::Include);
        qb.push(") union all (");
        Self::push_res_user_custom_sql_from_user(&mut qb, user_id, RbacRoleResRange::Any);
        qb.push(")) as tmp order by user_id asc");
        page.push_limit(&mut qb);
        Ok(qb.build_query_scalar::<u64>().fetch_all(&self.db).await?)
    }
    //被指定用户授权的用户数量
    pub async fn find_res_user_count_from_user(
        &self,
        user_id: u64, //访问用户ID,0 为游客
    ) -> RbacResult<i64> {
        if user_id == 0 {
            return Ok(0);
        }
        let mut qb = QueryBuilder::<MySql>::new("select COUNT(DISTINCT user_id) AS total from ((");
        Self::push_res_user_custom_sql_from_user(&mut qb, user_id, RbacRoleResRange::Exclude);
        qb.push(") union all (");
        Self::push_res_user_custom_sql_from_user(&mut qb, user_id, RbacRoleResRange::Include);
        qb.push(") union all (");
        Self::push_res_user_custom_sql_from_user(&mut qb, user_id, RbacRoleResRange::Any);
        qb.push(")) as tmp");
        Ok(qb.build_query_scalar::<i64>().fetch_one(&self.db).await?)
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
    fn push_res_data_from_custom_user_sql(
        qb: &mut QueryBuilder<'_, MySql>,
        user_id: u64,
        role_user_id: u64,
    ) {
        // Any range
        qb.push(format!(
            "select  role.res_range
            from {} as role
            join {} as role_user on role.id=role_user.role_id",
            RbacRoleModel::table_name(),
            RbacRoleUserModel::table_name(),
        ));
        qb.push_where()
            .field_eq("role.status", RbacRoleStatus::Enable as i8);
        qb.push_and().field_eq("role.user_id", role_user_id);
        qb.push_and()
            .field_eq("role.res_range", RbacRoleResRange::Any as i8);
        qb.push_and()
            .field_eq("role.user_range", RbacRoleUserRange::Custom as i8);
        qb.push_and()
            .field_eq("role_user.status", RbacRoleUserStatus::Enable as i8);
        qb.push_and().field_eq("role_user.user_id", user_id);
        qb.push_and()
            .push("(role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW())) limit 1");

        qb.push(" ) union all (");

        // Exclude range
        qb.push(format!(
            "select role.res_range
            from {} as role
            join {} as perm on role.id=perm.role_id
            join {} as role_user on role.id=role_user.role_id",
            RbacRoleModel::table_name(),
            RbacPermModel::table_name(),
            RbacRoleUserModel::table_name(),
        ));
        qb.push_where()
            .field_eq("role.status", RbacRoleStatus::Enable as i8);
        qb.push_and().field_eq("role.user_id", role_user_id);
        qb.push_and()
            .field_eq("role.res_range", RbacRoleResRange::Exclude as i8);
        qb.push_and()
            .field_eq("role.user_range", RbacRoleUserRange::Custom as i8);
        qb.push_and()
            .field_eq("role_user.status", RbacRoleUserStatus::Enable as i8);
        qb.push_and().field_eq("role_user.user_id", user_id);
        qb.push_and()
            .push("(role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW())) limit 1");

        qb.push(" ) union all (");

        // Include range
        qb.push(format!(
            "select role.res_range
            from {} as role
            join {} as perm on role.id=perm.role_id
            join {} as role_user on role.id=role_user.role_id",
            RbacRoleModel::table_name(),
            RbacPermModel::table_name(),
            RbacRoleUserModel::table_name(),
        ));
        qb.push_where()
            .field_eq("role.status", RbacRoleStatus::Enable as i8);
        qb.push_and().field_eq("role.user_id", role_user_id);
        qb.push_and()
            .field_eq("role.res_range", RbacRoleResRange::Include as i8);
        qb.push_and()
            .field_eq("role.user_range", RbacRoleUserRange::Custom as i8);
        qb.push_and()
            .field_eq("role_user.status", RbacRoleUserStatus::Enable as i8);
        qb.push_and().field_eq("role_user.user_id", user_id);
        qb.push_and()
            .push("(role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW())) limit 1");
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
        let mut qb = QueryBuilder::<MySql>::new("select * from ((");
        Self::push_res_data_from_custom_user_sql(&mut qb, access_user_id, role_user_id);
        qb.push(")) as t");
        let data = qb.build_query_scalar::<i8>().fetch_all(&self.db).await?;
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
    fn push_res_custom_sql_from_user(
        qb: &mut QueryBuilder<'_, MySql>,
        user_id: u64,
        role_user_id: u64,
        role_app_id: Option<u64>,
        res_range: RbacRoleResRange,
        field: &str,
    ) {
        qb.push(format!(
            "select {}
            from {} as role
            join {} as perm on role.id=perm.role_id
            join {} as res on perm.res_id=res.id
            join {} as op on perm.op_id=op.id
            join {} as role_user on role_user.role_id=role.id",
            field,
            RbacRoleModel::table_name(),
            RbacPermModel::table_name(),
            RbacResModel::table_name(),
            RbacOpModel::table_name(),
            RbacRoleUserModel::table_name(),
        ));
        qb.push_where()
            .field_eq("role.status", RbacRoleStatus::Enable as i8);
        qb.push_and()
            .field_eq("perm.status", RbacPermStatus::Enable as i8);
        qb.push_and()
            .field_eq("res.status", RbacResStatus::Enable as i8);
        qb.push_and()
            .field_eq("op.status", RbacOpStatus::Enable as i8);
        qb.push_and().field_eq("role.user_id", role_user_id);
        if let Some(app_id) = role_app_id {
            qb.push_and().field_eq("role.app_id", app_id);
        }
        qb.push_and()
            .field_eq("role.user_range", RbacRoleUserRange::Custom as i8);
        qb.push_and().field_eq("role.res_range", res_range as i8);
        qb.push_and().field_eq("role_user.user_id", user_id);
        qb.push_and()
            .push("(role_user.timeout=0 or role_user.timeout >= UNIX_TIMESTAMP(NOW()))");
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
                if user_id == 0 {
                    return Ok(0);
                }
                let mut qb = QueryBuilder::<MySql>::new("");
                Self::push_res_custom_sql_from_user(
                    &mut qb,
                    user_id,
                    role_user_id,
                    role_app_id,
                    res_range,
                    "count(*)",
                );
                Ok(qb.build_query_scalar::<i64>().fetch_one(&self.db).await?)
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
        page: &OffsetPageParam,
    ) -> RbacResult<Vec<AccessPermRow>> {
        match res_range {
            RbacRoleResRange::Exclude | RbacRoleResRange::Include => {
                if user_id == 0 {
                    return Ok(vec![]);
                }
                let mut qb = QueryBuilder::<MySql>::new("");
                Self::push_res_custom_sql_from_user(
                    &mut qb,
                    user_id,
                    role_user_id,
                    role_app_id,
                    res_range,
                    self.res_list_sql_field(),
                );
                qb.push(" order by perm.id desc");
                page.push_limit(&mut qb);
                Ok(qb
                    .build()
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

        let sql = format!(
            "select role.res_range
            from {} as role
            where role.status =? and role.role_key=? and role.user_id=? and role.user_range = ? limit 1",
            RbacRoleModel::table_name(),
        );
        let res_range = sqlx::query_scalar::<_, i8>(&sql)
            .bind(RbacRoleStatus::Enable as i8)
            .bind(&role_key)
            .bind(role_data.user_id)
            .bind(RbacRoleUserRange::Session as i8)
            .fetch_one(&self.db)
            .await?;
        Ok(RbacRoleResRange::try_from(res_range)?)
    }
    fn push_res_sql_from_session_role(
        qb: &mut QueryBuilder<'_, MySql>,
        role_data: &AccessSessionRole,
        res_range: RbacRoleResRange,
        field: &str,
    ) {
        let role_key = string_clear(
            role_data.role_key,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(33),
        );
        qb.push(format!(
            "select {}
            from {} as role
            join {} as perm on role.id=perm.role_id
            join {} as res on perm.res_id=res.id
            join {} as op on perm.op_id=op.id",
            field,
            RbacRoleModel::table_name(),
            RbacPermModel::table_name(),
            RbacResModel::table_name(),
            RbacOpModel::table_name(),
        ));
        qb.push_where()
            .field_eq("role.status", RbacRoleStatus::Enable as i8);
        qb.push_and().field_eq("role.role_key", role_key);
        qb.push_and().field_eq("role.res_range", res_range as i8);
        qb.push_and().field_eq("role.user_id", role_data.user_id);
        qb.push_and()
            .field_eq("role.user_range", RbacRoleUserRange::Session as i8);
        qb.push_and()
            .field_eq("perm.status", RbacPermStatus::Enable as i8);
        qb.push_and()
            .field_eq("res.status", RbacResStatus::Enable as i8);
        qb.push_and()
            .field_eq("op.status", RbacOpStatus::Enable as i8);
    }
    //列出会话角色可访问授权数量
    pub async fn find_res_count_from_session_role(
        &self,
        //该数据直接映射为对应角色
        role_data: &AccessSessionRole<'_>,
        res_range: RbacRoleResRange,
    ) -> RbacResult<i64> {
        let mut qb = QueryBuilder::<MySql>::new("");
        Self::push_res_sql_from_session_role(&mut qb, role_data, res_range, "count(*)");
        Ok(qb.build_query_scalar::<i64>().fetch_one(&self.db).await?)
    }
    //列出会话角色可访问授权数据
    pub async fn find_res_list_from_session_role(
        &self,
        //该数据直接映射为对应角色
        role_data: &AccessSessionRole<'_>,
        res_range: RbacRoleResRange,
        page: &OffsetPageParam,
    ) -> RbacResult<Vec<AccessPermRow>> {
        let mut qb = QueryBuilder::<MySql>::new("");
        Self::push_res_sql_from_session_role(
            &mut qb,
            role_data,
            res_range,
            self.res_list_sql_field(),
        );
        qb.push(" order by perm.id desc");
        page.push_limit(&mut qb);
        Ok(qb
            .build()
            .try_map(|row| Ok(self.res_list_from_mysql_row(row)))
            .fetch_all(&self.db)
            .await?)
    }
}
