use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    vec,
};

use lsys_core::{
    cache::LocalCache, fluent_message, impl_dao_fetch_map_by_vec, impl_dao_fetch_one_by_one,
    now_time, PageParam, RequestEnv,
};

use lsys_logger::dao::ChangeLogger;
use serde::Serialize;
use sqlx::{Acquire, FromRow, MySql, Pool, Row, Transaction};
use sqlx_model::{
    executor_option, model_option_set, sql_format, Insert, ModelTableName, Select, SqlExpr,
    SqlQuote, Update, WhereOption,
};
use tracing::{debug, error};

use crate::model::{
    RbacResModel, RbacResOpModel, RbacResStatus, RbacRoleModel, RbacRoleModelRef, RbacRoleOpModel,
    RbacRoleOpModelRef, RbacRoleOpPositivity, RbacRoleOpStatus, RbacRoleResOpRange, RbacRoleStatus,
    RbacRoleUserModel, RbacRoleUserModelRef, RbacRoleUserRange, RbacRoleUserStatus, RbacTagsModel,
    RbacTagsSource,
};

use super::{
    logger::{LogRole, LogRoleOp, LogRoleUser, LogRoleUserAction},
    RbacResData, RbacTags, RoleCheckData, RoleCheckRow, UserRbacError, UserRbacResult,
};

pub const ROLE_PRIORITY_NONE: i8 = -1;
pub const ROLE_PRIORITY_MIN: i8 = 0;
pub const ROLE_PRIORITY_MAX: i8 = 100;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RoleRelationKey {
    pub relation_key: String,
    pub user_id: u64,
}
impl RoleRelationKey {
    pub fn system(key: impl ToString) -> Self {
        Self {
            relation_key: key.to_string(),
            user_id: 0,
        }
    }
    pub fn user(key: impl ToString, user_id: u64) -> Self {
        Self {
            relation_key: key.to_string(),
            user_id,
        }
    }
}
impl ToString for RoleRelationKey {
    fn to_string(&self) -> String {
        format!("access-relation-{}-{}", self.user_id, self.relation_key)
    }
}

#[derive(Clone, Debug)]
pub struct RoleDetailRow {
    pub role: RbacRoleModel,
    pub role_ops: Vec<RbacRoleOpModel>,
}

#[derive(Clone, Debug)]
pub struct RoleAccessRow {
    pub role: RbacRoleModel,
    pub res_op_id: u64,
    pub op_positivity: RbacRoleOpPositivity,
    pub timeout: u64,
}

//角色管理
pub struct RbacRole {
    db: Pool<MySql>,
    tags: Arc<RbacTags>,
    cache_relation: Arc<LocalCache<String, Option<RoleDetailRow>>>,
    cache_access: Arc<LocalCache<String, Option<RoleAccessRow>>>,
    logger: Arc<ChangeLogger>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RoleAddUser {
    pub user_id: u64,
    pub timeout: u64, //换成时间不超过此值，查询时要有此值
}

#[derive(Clone, Debug, Serialize)]
pub struct RoleSetOp {
    pub res: RbacResModel,
    pub res_op: Vec<(RbacResOpModel, RbacRoleOpPositivity)>,
}

impl RbacRole {
    pub fn new(
        db: Pool<MySql>,

        tags: Arc<RbacTags>,
        cache_relation: Arc<LocalCache<String, Option<RoleDetailRow>>>,
        cache_access: Arc<LocalCache<String, Option<RoleAccessRow>>>,
        logger: Arc<ChangeLogger>,
    ) -> Self {
        Self {
            cache_relation,
            cache_access,
            db,
            // fluent,
            tags,
            logger,
        }
    }
    impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        RbacRoleModel,
        UserRbacResult<RbacRoleModel>,
        id,
        "id={id} and status = {status}",
        status = RbacRoleStatus::Enable
    );
    impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        RbacRoleModel,
        UserRbacResult<HashMap<u64, RbacRoleModel>>,
        id,
        id,
        "id in ({id}) and status = {status}",
        status = RbacRoleStatus::Enable
    );
    /// 获取指定条件的角色数量
    pub async fn get_count(
        &self,
        user_id: u64,
        user_range: &Option<Vec<i8>>,
        res_range: &Option<Vec<i8>>,
        role_name: &Option<String>,
        role_ids: &Option<Vec<u64>>,
    ) -> UserRbacResult<i64> {
        let mut sql = sql_format!(
            "select count(*) as total from {} where user_id = ? and status=?",
            RbacRoleModel::table_name()
        );
        if let Some(ref ur) = user_range {
            sql += &sql_format!(" and user_range in  ({})", ur);
        }
        if let Some(ref rr) = res_range {
            sql += &sql_format!(" and res_op_range in ({})", rr);
        }
        if let Some(ref name) = role_name {
            sql += sql_format!(" and name like {}", format!("%{}%", name)).as_str();
        }
        if let Some(ref rid) = role_ids {
            if rid.is_empty() {
                return Ok(0);
            } else {
                sql += &sql_format!(" and id in ({})", rid);
            }
        }
        let mut query = sqlx::query_scalar::<_, i64>(&sql);
        query = query.bind(user_id).bind(RbacRoleStatus::Enable as i8);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    /// 指定用户的所有TAG
    pub async fn user_role_tags(&self, user_id: u64) -> UserRbacResult<Vec<(String, i64)>> {
        self.tags
            .group_by_user_id(user_id, RbacTagsSource::Role)
            .await
    }
    /// 获取指定用户和ID的列表
    #[allow(clippy::too_many_arguments)]
    pub async fn get_role(
        &self,
        user_id: u64,
        user_range: &Option<Vec<i8>>,
        res_range: &Option<Vec<i8>>,
        role_name: &Option<String>,
        relation_prefix: &Option<String>,
        role_ids: &Option<Vec<u64>>,
        page: &Option<PageParam>,
    ) -> UserRbacResult<Vec<RbacRoleModel>> {
        let mut sql = sql_format!(
            "user_id = {} and status={}",
            user_id,
            RbacRoleStatus::Enable
        )
        .to_string();
        if let Some(ref rid) = role_ids {
            if rid.is_empty() {
                return Ok(vec![]);
            } else {
                sql += &sql_format!(" and id in ({})", rid);
            }
        }
        if let Some(ref ur) = user_range {
            sql += &sql_format!(" and user_range in  ({})", ur);
        }
        if let Some(ref rr) = res_range {
            sql += &sql_format!(" and res_op_range in ({})", rr);
        }
        if let Some(ref rr) = relation_prefix {
            sql += sql_format!(" and relation_key like {}", format!("{}%", rr)).as_str();
        }
        if let Some(name) = role_name {
            sql += sql_format!(" and name like {}", format!("%{}%", name)).as_str();
        }
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }

        let data = Select::type_new::<RbacRoleModel>()
            .fetch_all_by_where::<RbacRoleModel, _>(&WhereOption::Where(sql), &self.db)
            .await?;
        Ok(data)
    }
    /// 获取角色TAG
    pub async fn role_get_tags(
        &self,
        res_ids: &[u64],
    ) -> UserRbacResult<BTreeMap<u64, Vec<RbacTagsModel>>> {
        let data = self.tags.find_by_ids(res_ids, RbacTagsSource::Role).await?;
        let mut result = BTreeMap::<u64, Vec<RbacTagsModel>>::new();
        for res_op in data {
            result.entry(res_op.from_id).or_default().push(res_op);
        }
        Ok(result)
    }
    /// 设置角色TAG
    pub async fn role_set_tags<'t>(
        &self,
        role: &RbacRoleModel,
        tags: &[String],
        user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserRbacResult<()> {
        let tags = {
            let mut tout = Vec::with_capacity(tags.len());
            for tmp in tags.iter() {
                tout.push(check_length!(tmp, "name", 32));
            }
            tout
        };
        self.tags
            .set_tags(
                role.id,
                role.user_id,
                RbacTagsSource::Role,
                &tags,
                user_id,
                transaction,
                env_data,
            )
            .await
    }
    /// 检查优先级是否正确
    fn priority_check(&self, priority: i8) -> UserRbacResult<()> {
        if !(ROLE_PRIORITY_MIN..=ROLE_PRIORITY_MAX).contains(&priority) {
            return Err(UserRbacError::System(
                fluent_message!("rbac-priority-range",
                    {
                        "max":ROLE_PRIORITY_MAX,
                        "min":ROLE_PRIORITY_MIN
                    }
                ),
            )); // "priority Out of range: {$max}-{$min}",
        }
        Ok(())
    }
    /// 非 指定关系角色 检查
    fn not_relation_check(&self, user_range: RbacRoleUserRange) -> UserRbacResult<()> {
        if user_range == RbacRoleUserRange::Relation {
            return Err(UserRbacError::System(fluent_message!(
                "rbac-user-range-bad"
            ))); //"relation role plase use relation method"
        }
        Ok(())
    }
    //根据角色名获取启用的角色
    pub async fn find_enable_role_by_name(
        &self,
        user_id: u64,
        name: String,
    ) -> Result<RbacRoleModel, sqlx::Error> {
        Select::type_new::<RbacRoleModel>()
            .fetch_one_by_where::<RbacRoleModel, _>(
                &WhereOption::Where(sql_format!(
                    "user_id={} and name={} and status={}",
                    user_id,
                    name,
                    RbacRoleStatus::Enable
                )),
                &self.db,
            )
            .await
    }
    //根据关系名获取角色列表
    pub async fn find_enable_role_by_relation_keys(
        &self,
        user_id: u64,
        relation_keys: &Vec<String>,
    ) -> Result<Vec<RbacRoleModel>, sqlx::Error> {
        if relation_keys.is_empty() {
            return Ok(vec![]);
        }
        let sql = sql_format!(
            "user_id={} and relation_key in ({}) and status={}",
            user_id,
            relation_keys,
            RbacRoleStatus::Enable
        );
        Select::type_new::<RbacRoleModel>()
            .fetch_all_by_where::<RbacRoleModel, _>(&WhereOption::Where(sql), &self.db)
            .await
    }
    //根据关系名获取角色
    pub async fn find_enable_role_by_relation_key(
        &self,
        user_id: u64,
        relation_key: String,
    ) -> Result<RbacRoleModel, sqlx::Error> {
        Select::type_new::<RbacRoleModel>()
            .fetch_one_by_where::<RbacRoleModel, _>(
                &WhereOption::Where(sql_format!(
                    "user_id={} and relation_key={} and status={}",
                    user_id,
                    relation_key,
                    RbacRoleStatus::Enable
                )),
                &self.db,
            )
            .await
    }
    /// 添加角色
    #[allow(clippy::too_many_arguments)]
    async fn inner_add_role<'t>(
        &self,
        user_id: u64,
        relation_key: String,
        name: String,
        user_range: RbacRoleUserRange,
        res_op_range: RbacRoleResOpRange,
        priority: i8,
        add_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserRbacResult<u64> {
        self.priority_check(priority)?;
        let name = check_length!(name, "name", 32);
        let relation_key = if user_range == RbacRoleUserRange::Relation {
            check_length!(relation_key, "relation_key", 32)
        } else {
            "".to_string()
        };

        let res = self.find_enable_role_by_name(user_id, name.clone()).await;

        match res {
            Ok(row) => Err(UserRbacError::System(
                fluent_message!("rbac-role-exist",{
                    "name":row.name,
                    "id":row.id
                }), //"role {$name} already exists [{$id}]"
            )),
            Err(sqlx::Error::RowNotFound) => {
                let time = now_time().unwrap_or_default();
                let ur = user_range as i8;
                let orr = res_op_range as i8;
                let piy = priority;
                let relation_key_tmp = relation_key.clone();
                let idata = model_option_set!(RbacRoleModelRef,{
                    name:name,
                    user_range:ur,
                    res_op_range:orr,
                    priority: piy,
                    user_id:user_id,
                    change_time:time,
                    relation_key:relation_key_tmp,
                    change_user_id:add_user_id,
                    status:(RbacRoleStatus::Enable as i8),
                });

                let mut db = match transaction {
                    Some(pb) => pb.begin().await?,
                    None => self.db.begin().await?,
                };
                let tmp = Insert::<sqlx::MySql, RbacRoleModel, _>::new(idata)
                    .execute(&mut db)
                    .await;
                let id = match tmp {
                    Ok(res) => res.last_insert_id(),
                    Err(e) => {
                        db.rollback().await?;
                        return Err(e)?;
                    }
                };
                db.commit().await?;

                //cache clean----------------------------
                // public-global-{RbacRoleUserRange}
                if (user_range == RbacRoleUserRange::AllUser
                    || user_range == RbacRoleUserRange::Login)
                    && (res_op_range == RbacRoleResOpRange::AllowAll
                        || res_op_range == RbacRoleResOpRange::DenyAll)
                {
                    self.cache_access
                        .clear(
                            &self.find_role_cache_key_by_public_global(user_range as i8, user_id),
                        )
                        .await;
                }

                //access-relation-{role.user_id}-{relation_key}
                if user_range == RbacRoleUserRange::Relation {
                    self.cache_relation
                        .clear(
                            &RoleRelationKey {
                                relation_key: relation_key.clone(),
                                user_id,
                            }
                            .to_string(),
                        )
                        .await;
                }
                //cache clean----------------------------

                self.logger
                    .add(
                        &LogRole {
                            name,
                            relation_key,
                            priority,
                            user_range,
                            res_op_range,
                            action: "add",
                        },
                        &Some(id),
                        &Some(user_id),
                        &Some(add_user_id),
                        None,
                        env_data,
                    )
                    .await;

                Ok(id)
            }
            Err(err) => Err(err.into()),
        }
    }
    //添加角色
    #[allow(clippy::too_many_arguments)]
    pub async fn add_role<'t>(
        &self,
        user_id: u64,
        name: String,
        user_range: RbacRoleUserRange,
        res_op_range: RbacRoleResOpRange,
        priority: i8,
        add_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserRbacResult<u64> {
        self.not_relation_check(user_range)?;
        self.inner_add_role(
            user_id,
            "".to_string(),
            name,
            user_range,
            res_op_range,
            priority,
            add_user_id,
            transaction,
            env_data,
        )
        .await
    }
    //添加用户 指定关系角色
    #[allow(clippy::too_many_arguments)]
    pub async fn add_relation_role<'t>(
        &self,
        user_id: u64,
        relation_key: String,
        name: String,
        res_op_range: RbacRoleResOpRange,
        priority: i8,
        add_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserRbacResult<u64> {
        if relation_key.is_empty() {
            return Err(UserRbacError::System(fluent_message!(
                "rbac-miss-relation-key"
            ))); //"set role relation key can't be empty"
        }
        let user_range = RbacRoleUserRange::Relation;
        let res = self
            .find_enable_role_by_relation_key(user_id, relation_key.clone())
            .await;
        match res {
            Ok(row) => Err(UserRbacError::System(
                fluent_message!("rbac-relation-key-exist",{
                    "name":row.name,
                    "id":row.id,
                    "relation_key":relation_key
                }), //"role relation key {$relation_key}  already exists on {$name}  [{$id}]",
            )),
            Err(sqlx::Error::RowNotFound) => {
                self.inner_add_role(
                    user_id,
                    relation_key,
                    name,
                    user_range,
                    res_op_range,
                    priority,
                    add_user_id,
                    transaction,
                    env_data,
                )
                .await
            }
            Err(err) => Err(err.into()),
        }
    }

    /// 编辑角色
    #[allow(clippy::too_many_arguments)]
    async fn inner_edit_role<'t>(
        &self,
        role: &RbacRoleModel,
        name: Option<String>,
        relation_key: Option<String>,
        priority: Option<i8>,
        user_range: Option<RbacRoleUserRange>,
        res_op_range: Option<RbacRoleResOpRange>,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserRbacResult<()> {
        let name = if let Some(name) = name {
            Some(check_length!(name, "name", 32))
        } else {
            None
        };
        let relation_key = if user_range
            .map(|ref user_range| *user_range == RbacRoleUserRange::Relation)
            .unwrap_or(false)
            || (RbacRoleUserRange::Relation.eq(role.res_op_range) && user_range.is_none())
        {
            if let Some(relation_key_inner) = relation_key {
                Some(check_length!(relation_key_inner, "relation_key", 32))
            } else {
                None
            }
        } else {
            None
        };

        if let Some(p) = priority {
            self.priority_check(p)?;
        }
        if let Some(n) = &name {
            let tname = n.to_owned();
            if tname != role.name {
                match self.find_enable_role_by_name(role.user_id, tname).await {
                    Ok(row) => {
                        return Err(UserRbacError::System(
                            fluent_message!("rbac-role-exist",{
                                "name":row.name,
                                "id":row.id
                            }),
                            //"role name {$name} already exists in [{$id}]",
                        ));
                    }
                    Err(sqlx::Error::RowNotFound) => {}
                    Err(err) => return Err(err.into()),
                }
            }
        }
        let ur = user_range.map(|e| e as i8);
        let ro = res_op_range.map(|e| e as i8);
        let time = now_time().unwrap_or_default();
        let mut change = sqlx_model::model_option_set!(RbacRoleModelRef,{
            change_user_id:change_user_id,
            change_time:time,
        });
        change.name = name.as_ref();
        change.relation_key = relation_key.as_ref();
        change.user_range = ur.as_ref();
        change.res_op_range = ro.as_ref();
        change.priority = priority.as_ref();

        let change_user = if RbacRoleUserRange::User.eq(role.user_range)
            && user_range
                .map(|e| e != RbacRoleUserRange::User)
                .unwrap_or(false)
        {
            Select::type_new::<RbacRoleUserModel>()
                .fetch_all_by_where::<RbacRoleUserModel, _>(
                    &WhereOption::Where(sql_format!(
                        "role_id={} and status={}",
                        role.id,
                        RbacRoleUserStatus::Enable
                    )),
                    &self.db,
                )
                .await?
        } else {
            vec![]
        };

        let change_op = if RbacRoleResOpRange::AllowCustom.eq(role.res_op_range)
            && res_op_range
                .map(|e| e != RbacRoleResOpRange::AllowCustom)
                .unwrap_or(false)
        {
            Select::type_new::<RbacRoleOpModel>()
                .fetch_all_by_where::<RbacRoleOpModel, _>(
                    &WhereOption::Where(sql_format!(
                        "role_id={} and status={}",
                        role.id,
                        RbacRoleOpStatus::Enable
                    )),
                    &self.db,
                )
                .await?
        } else {
            vec![]
        };

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        if RbacRoleUserRange::User.eq(role.user_range)
            && user_range
                .map(|e| e != RbacRoleUserRange::User)
                .unwrap_or(false)
        {
            let change_role_user = sqlx_model::model_option_set!(RbacRoleUserModelRef,{
                change_user_id:change_user_id,
                change_time:time,
                status:(RbacRoleUserStatus::Delete as i8)
            });
            let tmp = Update::<sqlx::MySql, RbacRoleUserModel, _>::new(change_role_user)
                .execute_by_where(
                    &WhereOption::Where(sql_format!("role_id={}", role.id)),
                    &mut db,
                )
                .await;
            if let Err(e) = tmp {
                db.rollback().await?;
                return Err(e)?;
            }
        }
        if RbacRoleResOpRange::AllowCustom.eq(role.res_op_range)
            && res_op_range
                .map(|e| e != RbacRoleResOpRange::AllowCustom)
                .unwrap_or(false)
        {
            let change_role_op = sqlx_model::model_option_set!(RbacRoleOpModelRef,{
                change_user_id:change_user_id,
                change_time:time,
                status:(RbacRoleOpStatus::Delete as i8)
            });
            let tmp = Update::<sqlx::MySql, RbacRoleOpModel, _>::new(change_role_op)
                .execute_by_where(
                    &WhereOption::Where(sql_format!("role_id={}", role.id)),
                    &mut db,
                )
                .await;
            if let Err(e) = tmp {
                db.rollback().await?;
                return Err(e)?;
            }
        }
        let update = Update::<sqlx::MySql, RbacRoleModel, _>::new(change);
        let tmp = update.execute_by_pk(role, &mut db).await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }
        db.commit().await?;

        //cache clean----------------------------
        //public-global-{RbacRoleUserRange}
        if res_op_range
            .map(|e| e == RbacRoleResOpRange::AllowAll || e == RbacRoleResOpRange::DenyAll)
            .unwrap_or(false)
        {
            if let Some(ur) = user_range {
                if ur == RbacRoleUserRange::AllUser || ur == RbacRoleUserRange::Login {
                    self.cache_access
                        .clear(&self.find_role_cache_key_by_public_global(ur as i8, role.user_id))
                        .await;
                }
            }
        }
        if ((RbacRoleUserRange::AllUser.eq(role.user_range)
            || RbacRoleUserRange::Login.eq(role.user_range))
            && RbacRoleResOpRange::AllowAll.eq(role.res_op_range)
            || RbacRoleResOpRange::DenyAll.eq(role.res_op_range))
            && (user_range.is_some() || res_op_range.is_some() || priority.is_some())
        {
            self.cache_access
                .clear(&self.find_role_cache_key_by_public_global(role.user_range, role.user_id))
                .await;
        }
        //public-res-user-{RbacRoleUserRange}-{role.user_id}

        if (RbacRoleUserRange::AllUser.eq(role.user_range)
            || RbacRoleUserRange::Login.eq(role.user_range))
            && (user_range.is_some() || res_op_range.is_some() || priority.is_some())
        {
            self.cache_access
                .clear(&self.find_role_cache_key_by_public_global(role.user_range, role.user_id))
                .await;
        }
        // public-res-{RbacRoleUserRange}-{yaf_rbac_role_op.op_id}
        if RbacRoleResOpRange::AllowCustom.eq(role.res_op_range) {
            if let Some(ur) = user_range {
                if ur == RbacRoleUserRange::AllUser || ur == RbacRoleUserRange::Login {
                    for tmp in change_op.iter() {
                        self.cache_access
                            .clear(&self.find_role_cache_key_by_public_res(
                                ur as i8,
                                tmp.res_op_id,
                                role.user_id,
                            ))
                            .await;
                    }
                }
            }
        }
        if ((RbacRoleUserRange::AllUser.eq(role.user_range)
            || RbacRoleUserRange::Login.eq(role.user_range))
            && RbacRoleResOpRange::AllowCustom.eq(role.res_op_range))
            && (user_range.is_some() || res_op_range.is_some() || priority.is_some())
        {
            for tmp in change_op.iter() {
                self.cache_access
                    .clear(&self.find_role_cache_key_by_public_res(
                        role.user_range,
                        tmp.res_op_id,
                        role.user_id,
                    ))
                    .await;
            }
        }
        //access-relation-{role.user_id}-{relation_key}
        if RbacRoleUserRange::Relation.eq(role.user_range) {
            if let Some(ref key) = relation_key {
                self.cache_relation
                    .clear(
                        &RoleRelationKey {
                            relation_key: key.to_string(),
                            user_id: role.user_id,
                        }
                        .to_string(),
                    )
                    .await;
            }
            if relation_key.is_some() || res_op_range.is_some() || priority.is_some() {
                self.cache_relation
                    .clear(
                        &RoleRelationKey {
                            relation_key: role.relation_key.clone(),
                            user_id: role.user_id,
                        }
                        .to_string(),
                    )
                    .await;
            }
        }

        if RbacRoleUserRange::User.eq(role.user_range) {
            if let Some(rt) = res_op_range {
                // user-global-{view.user_id}
                if RbacRoleResOpRange::AllowAll == rt || RbacRoleResOpRange::DenyAll == rt {
                    for tmp in change_user.iter() {
                        self.cache_access
                            .clear(
                                &self.find_role_cache_key_by_user_global(tmp.user_id, role.user_id),
                            )
                            .await;
                    }
                }
            }
            // user-global-{view.user_id}
            if (RbacRoleResOpRange::AllowAll.eq(role.res_op_range)
                || RbacRoleResOpRange::DenyAll.eq(role.res_op_range))
                && (user_range.is_some() || res_op_range.is_some() || priority.is_some())
            {
                for tmp in change_user.iter() {
                    self.cache_access
                        .clear(&self.find_role_cache_key_by_user_global(tmp.user_id, tmp.user_id))
                        .await;
                }
            }

            //user-res-{view.user_id}-{role_op.id}
            if RbacRoleResOpRange::AllowCustom.eq(role.res_op_range)
                && (user_range.is_some() || res_op_range.is_some() || priority.is_some())
            {
                for tmp_user in change_user.iter() {
                    for tmp_op in change_op.iter() {
                        self.cache_access
                            .clear(&self.find_role_cache_key_by_user_res(
                                tmp_user.user_id,
                                tmp_op.res_op_id,
                                0,
                            ))
                            .await;
                    }
                }
            }
        }
        //cache clean----------------------------

        self.logger
            .add(
                &LogRole {
                    priority: priority.unwrap_or(role.priority),
                    user_range: user_range.unwrap_or(
                        RbacRoleUserRange::try_from(role.user_range)
                            .unwrap_or(RbacRoleUserRange::AllUser),
                    ),
                    res_op_range: res_op_range.unwrap_or(
                        RbacRoleResOpRange::try_from(role.res_op_range)
                            .unwrap_or(RbacRoleResOpRange::AllowAll),
                    ),
                    relation_key: relation_key.unwrap_or(role.relation_key.to_owned()),
                    name: name.unwrap_or(role.name.clone().to_owned()),
                    action: "edit",
                },
                &Some(role.id),
                &Some(role.user_id),
                &Some(change_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
    //编辑角色
    #[allow(clippy::too_many_arguments)]
    pub async fn edit_role<'t>(
        &self,
        role: &RbacRoleModel,
        name: Option<String>,
        priority: Option<i8>,
        user_range: Option<RbacRoleUserRange>,
        res_op_range: Option<RbacRoleResOpRange>,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserRbacResult<()> {
        self.not_relation_check(RbacRoleUserRange::try_from(role.user_range)?)?;
        if let Some(ur) = user_range {
            self.not_relation_check(ur)?;
        }
        self.inner_edit_role(
            role,
            name,
            None,
            priority,
            user_range,
            res_op_range,
            change_user_id,
            transaction,
            env_data,
        )
        .await
    }
    //根据关系key编辑指定关系角色
    #[allow(clippy::too_many_arguments)]
    pub async fn edit_relation_role<'t>(
        &self,
        role: &RbacRoleModel,
        relation_key: Option<String>,
        name: Option<String>,
        priority: Option<i8>,
        res_op_range: Option<RbacRoleResOpRange>,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserRbacResult<()> {
        if let Some(n) = &relation_key {
            let tname = n.to_owned();
            if tname.is_empty() {
                return Err(UserRbacError::System(fluent_message!(
                    "rbac-miss-relation-key" //,
                                             //  "set role relation key can't be empty"
                )));
            }
            if tname != role.relation_key {
                match self
                    .find_enable_role_by_relation_key(role.user_id, tname)
                    .await
                {
                    Ok(row) => {
                        return Err(UserRbacError::System(
                            fluent_message!("rbac-role-exist",{
                                "name":row.name,
                                "id":row.id
                            }), //"role relation_key {$name} already exists in [{$id}]",
                        ));
                    }
                    Err(sqlx::Error::RowNotFound) => {}
                    Err(err) => return Err(err.into()),
                }
            }
        }
        self.inner_edit_role(
            role,
            name,
            relation_key,
            priority,
            None,
            res_op_range,
            change_user_id,
            transaction,
            env_data,
        )
        .await
    }

    /// 删除角色
    pub async fn del_role<'t>(
        &self,
        role: &RbacRoleModel,
        delete_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserRbacResult<()> {
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(RbacRoleModelRef,{
            change_user_id:delete_user_id,
            change_time:time,
            status:(RbacRoleStatus::Delete as i8)
        });

        let change_user = if RbacRoleUserRange::User.eq(role.user_range) {
            Select::type_new::<RbacRoleUserModel>()
                .fetch_all_by_where::<RbacRoleUserModel, _>(
                    &WhereOption::Where(sql_format!(
                        "role_id={} and status={}",
                        role.id,
                        RbacRoleUserStatus::Enable,
                    )),
                    &self.db,
                )
                .await?
        } else {
            vec![]
        };

        let change_op = if RbacRoleResOpRange::AllowCustom.eq(role.res_op_range) {
            Select::type_new::<RbacRoleOpModel>()
                .fetch_all_by_where::<RbacRoleOpModel, _>(
                    &WhereOption::Where(sql_format!(
                        "role_id={} and status={}",
                        role.id,
                        RbacRoleOpStatus::Enable
                    )),
                    &self.db,
                )
                .await?
        } else {
            vec![]
        };

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::<sqlx::MySql, RbacRoleModel, _>::new(change)
            .execute_by_pk(role, &mut db)
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }
        let tmp = self
            .tags
            .del_tags(
                role.id,
                RbacTagsSource::Role,
                delete_user_id,
                Some(&mut db),
                env_data,
            )
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }

        if RbacRoleUserRange::User.eq(role.user_range) {
            let change = sqlx_model::model_option_set!(RbacRoleUserModelRef,{
                change_user_id:delete_user_id,
                change_time:time,
                status:(RbacRoleUserStatus::Delete as i8)
            });
            let tmp = Update::<sqlx::MySql, RbacRoleUserModel, _>::new(change)
                .execute_by_where(
                    &WhereOption::Where(sql_format!("role_id={}", role.id)),
                    &mut db,
                )
                .await;
            if let Err(e) = tmp {
                db.rollback().await?;
                return Err(e)?;
            }
        }

        if RbacRoleResOpRange::AllowCustom.eq(role.res_op_range) {
            let change = sqlx_model::model_option_set!(RbacRoleOpModelRef,{
                change_user_id:delete_user_id,
                change_time:time,
                status:(RbacRoleOpStatus::Delete as i8)
            });
            let tmp = Update::<sqlx::MySql, RbacRoleOpModel, _>::new(change)
                .execute_by_where(
                    &WhereOption::Where(sql_format!("role_id={}", role.id)),
                    &mut db,
                )
                .await;
            if let Err(e) = tmp {
                db.rollback().await?;
                return Err(e)?;
            }
        }
        db.commit().await?;
        //cache clean----------------------------
        //public-global-{RbacRoleUserRange}
        if (RbacRoleUserRange::AllUser.eq(role.user_range)
            || RbacRoleUserRange::Login.eq(role.user_range))
            && RbacRoleResOpRange::AllowAll.eq(role.res_op_range)
            || RbacRoleResOpRange::DenyAll.eq(role.res_op_range)
        {
            self.cache_access
                .clear(&self.find_role_cache_key_by_public_global(role.user_range, role.user_id))
                .await;
        }

        //public-res-{RbacRoleUserRange}-{yaf_rbac_role_op.op_id}
        if (RbacRoleUserRange::AllUser.eq(role.user_range)
            || RbacRoleUserRange::Login.eq(role.user_range))
            && RbacRoleResOpRange::AllowCustom.eq(role.res_op_range)
        {
            for tmp in change_op.iter() {
                self.cache_access
                    .clear(&self.find_role_cache_key_by_public_res(
                        role.user_range,
                        tmp.res_op_id,
                        role.user_id,
                    ))
                    .await;
            }
        }

        //access-relation-{role.user_id}-{relation_key}
        if RbacRoleUserRange::Relation.eq(role.user_range) {
            self.cache_relation
                .clear(
                    &RoleRelationKey {
                        relation_key: role.relation_key.clone(),
                        user_id: role.user_id,
                    }
                    .to_string(),
                )
                .await;
        }
        // user-global-{view.user_id}
        if RbacRoleUserRange::User.eq(role.user_range)
            && (RbacRoleResOpRange::AllowAll.eq(role.res_op_range)
                || RbacRoleResOpRange::DenyAll.eq(role.res_op_range))
        {
            for tmp in change_user.iter() {
                self.cache_access
                    .clear(&self.find_role_cache_key_by_user_global(tmp.user_id, role.user_id))
                    .await;
            }
        }

        //user-res-{view.user_id}-{role_op.id}
        if RbacRoleUserRange::User.eq(role.user_range)
            && RbacRoleResOpRange::AllowCustom.eq(role.res_op_range)
        {
            for tmp_user in change_user.iter() {
                for tmp_op in change_op.iter() {
                    self.cache_access
                        .clear(&self.find_role_cache_key_by_user_res(
                            tmp_user.user_id,
                            tmp_op.res_op_id,
                            role.user_id,
                        ))
                        .await;
                }
            }
        }
        //cache clean----------------------------

        self.logger
            .add(
                &LogRole {
                    name: role.name.clone(),
                    relation_key: role.relation_key.clone(),
                    priority: role.priority,
                    user_range: RbacRoleUserRange::try_from(role.user_range)
                        .unwrap_or(RbacRoleUserRange::AllUser),
                    res_op_range: RbacRoleResOpRange::try_from(role.res_op_range)
                        .unwrap_or(RbacRoleResOpRange::AllowAll),
                    action: "del",
                },
                &Some(role.id),
                &Some(role.user_id),
                &Some(delete_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
    /// 角色添加用户
    pub async fn role_add_user<'t>(
        &self,
        role: &RbacRoleModel,
        user_vec: &[RoleAddUser],
        add_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserRbacResult<()> {
        if user_vec.is_empty() {
            return Ok(());
        }
        if !RbacRoleUserRange::User.eq(role.user_range) {
            return Err(UserRbacError::System(
                fluent_message!("rbac-res-op-user-wrong",{
                    "name":role.name,
                    "role_id":role.id,
                    "range":role.user_range
                }), //"role({$name})[range:{$range}] can't set user [{$role_id}]",
            ));
        }
        let db = &self.db;

        let user_id_vec = user_vec.iter().map(|e| e.user_id).collect::<Vec<_>>();

        let res = Select::type_new::<RbacRoleUserModel>()
            .fetch_one_by_where::<RbacRoleUserModel, _>(
                &WhereOption::Where(sql_format!(
                    "user_id in ({}) and role_id={} and status={}",
                    user_id_vec,
                    role.id,
                    RbacRoleUserStatus::Enable
                )),
                db,
            )
            .await;

        let time = now_time().unwrap_or_default();

        let mut add_item = vec![];
        let mut add_uids = vec![];
        for RoleAddUser { user_id, timeout } in user_vec.iter() {
            if res
                .iter()
                .any(|x| x.user_id == *user_id && x.timeout == *timeout)
            {
                continue;
            }
            let mut item = model_option_set!(RbacRoleUserModelRef,{
                role_id:role.id,
                change_time:time,
                change_user_id:add_user_id,
                status:(RbacRoleUserStatus::Enable as i8),
            });
            item.user_id = Some(user_id);
            item.timeout = Some(timeout);
            add_item.push(item);
            add_uids.push(*user_id);
        }
        if !add_item.is_empty() {
            executor_option!(
                {
                    Insert::<sqlx::MySql, RbacRoleUserModel, _>::new_vec(add_item)
                        .execute(db)
                        .await?;
                },
                transaction,
                db,
                db
            );
        }
        //cache clean----------------------------
        // user-global-{view.user_id}
        if RbacRoleUserRange::User.eq(role.user_range)
            && (RbacRoleResOpRange::AllowAll.eq(role.res_op_range)
                || RbacRoleResOpRange::DenyAll.eq(role.res_op_range))
        {
            for uid in add_uids.iter() {
                self.cache_access
                    .clear(&self.find_role_cache_key_by_user_global(*uid, role.user_id))
                    .await;
            }
        }

        //user-res-{view.user_id}-{role_op.id}
        if RbacRoleUserRange::User.eq(role.user_range)
            && RbacRoleResOpRange::AllowCustom.eq(role.res_op_range)
            && !add_uids.is_empty()
        {
            let change_op = Select::type_new::<RbacRoleOpModel>()
                .fetch_all_by_where::<RbacRoleOpModel, _>(
                    &WhereOption::Where(sql_format!(
                        "role_id={} and status={}",
                        role.id,
                        RbacRoleOpStatus::Enable
                    )),
                    &self.db,
                )
                .await?;
            for uid in add_uids.iter() {
                for tmp_op in change_op.iter() {
                    self.cache_access
                        .clear(&self.find_role_cache_key_by_user_res(
                            *uid,
                            tmp_op.res_op_id,
                            role.user_id,
                        ))
                        .await;
                }
            }
        }
        //cache clean----------------------------

        self.logger
            .add(
                &LogRoleUser {
                    action: LogRoleUserAction::Add,
                    name: role.name.clone(),
                    add_user: Some(user_vec.to_owned()),
                    del_user: None,
                },
                &Some(role.id),
                &Some(role.user_id),
                &Some(add_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
    /// 角色删除用户
    pub async fn role_del_user<'t>(
        &self,
        role: &RbacRoleModel,
        user_id_vec: &[u64],
        del_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserRbacResult<u64> {
        if user_id_vec.is_empty() {
            return Ok(0);
        }
        let time = now_time().unwrap_or_default();
        let ddata = model_option_set!(RbacRoleUserModelRef,{
            change_user_id:del_user_id,
            change_time:time,
            status:(RbacRoleUserStatus::Delete as i8),
        });

        let db = &self.db;
        let res = executor_option!(
            {
                Update::<sqlx::MySql, RbacRoleUserModel, _>::new(ddata)
                    .execute_by_where(
                        &WhereOption::Where(sql_format!(
                            "role_id ={} and user_id  in ({})",
                            role.id,
                            user_id_vec
                        )),
                        db,
                    )
                    .await?
            },
            transaction,
            db,
            db
        );
        //cache clean----------------------------
        // user-global-{view.user_id}
        if RbacRoleUserRange::User.eq(role.user_range)
            && (RbacRoleResOpRange::AllowAll.eq(role.res_op_range)
                || RbacRoleResOpRange::DenyAll.eq(role.res_op_range))
        {
            for uid in user_id_vec.iter() {
                self.cache_access
                    .clear(&self.find_role_cache_key_by_user_global(*uid, role.user_id))
                    .await;
            }
        }

        //user-res-{view.user_id}-{role_op.id}
        if RbacRoleUserRange::User.eq(role.user_range)
            && RbacRoleResOpRange::AllowCustom.eq(role.res_op_range)
            && !user_id_vec.is_empty()
        {
            let change_op = Select::type_new::<RbacRoleOpModel>()
                .fetch_all_by_where::<RbacRoleOpModel, _>(
                    &WhereOption::Where(sql_format!(
                        "role_id={} and status={}",
                        role.id,
                        RbacRoleOpStatus::Enable
                    )),
                    &self.db,
                )
                .await?;
            for uid in user_id_vec.iter() {
                for tmp_op in change_op.iter() {
                    self.cache_access
                        .clear(&self.find_role_cache_key_by_user_res(
                            *uid,
                            tmp_op.res_op_id,
                            role.user_id,
                        ))
                        .await;
                }
            }
        }
        //cache clean----------------------------

        self.logger
            .add(
                &LogRoleUser {
                    action: LogRoleUserAction::Del,
                    name: role.name.clone(),
                    add_user: None,
                    del_user: Some(user_id_vec.to_owned()),
                },
                &Some(role.id),
                &Some(role.user_id),
                &Some(del_user_id),
                None,
                env_data,
            )
            .await;
        Ok(res.rows_affected())
    }
    //汇总指定关系角色的用户数量
    pub async fn role_group_users(
        &self,
        role_ids: &[u64],
        all: bool,
    ) -> UserRbacResult<BTreeMap<u64, i64>> {
        if role_ids.is_empty() {
            return Ok(BTreeMap::new());
        }
        let ok_where = if all {
            SqlExpr("".to_string())
        } else {
            SqlExpr(sql_format!(
                " and (timeout=0 or timeout>{})",
                now_time().unwrap_or(0)
            ))
        };
        let sql = sql_format!(
            "select role_id,
            count(*) as total 
            from {table} 
            where role_id in ({role_ids}) 
            and status={status}
            {ok_where}
            group by role_id ",
            table = RbacRoleUserModel::table_name(),
            role_ids = role_ids,
            status = RbacRoleUserStatus::Enable as i8,
            ok_where = ok_where
        );
        let data = sqlx::query_as::<_, (u64, i64)>(sql.as_str())
            .fetch_all(&self.db)
            .await?;
        let mut result = BTreeMap::<u64, i64>::new();
        for op in data {
            result.entry(op.0).or_insert(op.1);
        }
        Ok(result)
    }
    /// 角色获取用户
    pub async fn role_get_users(
        &self,
        role_ids: &[u64],
        user_ids: &Option<Vec<u64>>, //用在检查指定用户id是否已经添加
        page: &Option<PageParam>,
    ) -> UserRbacResult<BTreeMap<u64, Vec<RbacRoleUserModel>>> {
        if role_ids.is_empty() {
            return Ok(BTreeMap::new());
        }
        let db = &self.db;
        let mut sql = sql_format!(
            "role_id in ({}) and status={}",
            role_ids,
            RbacRoleUserStatus::Enable
        );

        if let Some(u_ids) = user_ids {
            sql += &sql_format!(" and user_id in ({})", u_ids);
        }
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }
        let data = Select::type_new::<RbacRoleUserModel>()
            .fetch_all_by_where::<RbacRoleUserModel, _>(&WhereOption::Where(sql), db)
            .await?;
        let mut result = BTreeMap::<u64, Vec<RbacRoleUserModel>>::new();
        for op in data {
            result.entry(op.role_id).or_default().push(op);
        }
        Ok(result)
    }
    /// 获取指定用户的指定关系角色key
    pub async fn get_role_relation_data(
        &self,
        user_id: &u64,
        prefix: &Option<String>,
        page: &Option<PageParam>,
    ) -> UserRbacResult<Vec<String>> {
        let mut sql = sql_format!(
            "select relation_key from {} where user_id = {} and status={} and user_range={}",
            RbacRoleModel::table_name(),
            user_id,
            RbacRoleUserStatus::Enable,
            RbacRoleUserRange::Relation
        );
        if let Some(prefix) = prefix {
            sql += &sql_format!(" and relation_key like {}", format!("{}%", prefix));
        }
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }
        let query = sqlx::query_scalar::<_, String>(&sql);
        let res = query.fetch_all(&self.db).await?;
        Ok(res)
    }
    /// 获取指定用户的指定关系角色数量
    pub async fn get_role_relation_count(
        &self,
        user_id: &u64,
        prefix: &Option<String>,
    ) -> UserRbacResult<i64> {
        let mut sql = sql_format!(
            "select count(*) as total from {} where user_id = {} and status={} and user_range={}",
            RbacRoleModel::table_name(),
            user_id,
            RbacRoleUserStatus::Enable,
            RbacRoleUserRange::Relation
        );
        if let Some(prefix) = prefix {
            sql += &sql_format!(" and relation_key like {}", format!("{}%", prefix));
        }
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    /// 角色获取用户数量
    pub async fn role_get_user_count(
        &self,
        role_ids: &[u64],
        user_ids: &Option<Vec<u64>>,
    ) -> UserRbacResult<i64> {
        if role_ids.is_empty() {
            return Ok(0);
        }
        let mut sql = sql_format!(
            "select count(*) as total from {} where role_id in ({}) and status=?",
            RbacRoleUserModel::table_name(),
            role_ids
        );
        if let Some(u_ids) = user_ids {
            sql += &sql_format!(" and user_id in ({})", u_ids);
        }
        let mut query = sqlx::query_scalar::<_, i64>(&sql);
        query = query.bind(RbacRoleUserStatus::Enable as i8);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    /// 角色获取操作
    pub async fn role_get_ops<'t>(
        &self,
        role_ids: &[u64],
    ) -> UserRbacResult<BTreeMap<u64, Vec<RbacRoleOpModel>>> {
        if role_ids.is_empty() {
            return Ok(BTreeMap::new());
        }
        let db = &self.db;
        let data = Select::type_new::<RbacRoleOpModel>()
            .fetch_all_by_where::<RbacRoleOpModel, _>(
                &WhereOption::Where(sql_format!(
                    "role_id IN ({}) and status={}",
                    role_ids,
                    RbacRoleStatus::Enable
                )),
                db,
            )
            .await?;
        let mut result = BTreeMap::<u64, Vec<RbacRoleOpModel>>::new();
        for op in data {
            result.entry(op.role_id).or_default().push(op);
        }
        Ok(result)
    }
    //从角色的资源关系中删除指定资源操作id的关系
    pub(crate) async fn all_role_del_ops<'t>(
        &self,
        role_op_id_vec: &[u64],
        del_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserRbacResult<()> {
        let rops = if role_op_id_vec.is_empty() {
            vec![]
        } else {
            Select::type_new::<RbacRoleOpModel>()
                .fetch_all_by_where::<RbacRoleOpModel, _>(
                    &WhereOption::Where(sql_format!(
                        "res_op_id in ({}) and status={}",
                        role_op_id_vec,
                        RbacRoleOpStatus::Enable
                    )),
                    &self.db,
                )
                .await?
        };
        let roles = if rops.is_empty() {
            vec![]
        } else {
            Select::type_new::<RbacRoleModel>()
                .fetch_all_by_where::<RbacRoleModel, _>(
                    &WhereOption::Where(sql_format!(
                        "id in ({}) ",
                        rops.iter().map(|e| e.role_id).collect::<Vec<u64>>()
                    )),
                    &self.db,
                )
                .await?
        };
        let del_op = rops.iter().map(|e| e.id).collect::<Vec<u64>>();
        let time = now_time().unwrap_or_default();
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let ddata = model_option_set!(RbacRoleOpModelRef,{
            change_user_id:del_user_id,
            change_time:time,
            status:(RbacRoleOpStatus::Delete as i8),
        });
        if !del_op.is_empty() {
            let tmp = Update::<sqlx::MySql, RbacRoleOpModel, _>::new(ddata)
                .execute_by_where(
                    &WhereOption::Where(sql_format!("id in ({})", del_op)),
                    &mut db,
                )
                .await;
            if let Err(e) = tmp {
                db.rollback().await?;
                return Err(e)?;
            }
        }
        db.commit().await?;
        //cache clean----------------------------
        //public-res-{RbacRoleUserRange}-{yaf_rbac_role_op.op_id}
        for r in roles.iter() {
            for tmp in rops.iter() {
                if tmp.role_id != r.id {
                    continue;
                }
                // public-res-{RbacRoleUserRange}-{yaf_rbac_role_op.op_id}
                if RbacRoleUserRange::AllUser.eq(r.user_range)
                    || RbacRoleUserRange::Login.eq(r.user_range)
                {
                    self.cache_access
                        .clear(&self.find_role_cache_key_by_public_res(
                            r.user_range,
                            tmp.res_op_id,
                            0,
                        ))
                        .await;
                }
            }
            //access-relation-{role.user_id}-{relation_key}
            if RbacRoleUserRange::Relation.eq(r.user_range) {
                self.cache_relation
                    .clear(
                        &RoleRelationKey {
                            relation_key: r.relation_key.clone(),
                            user_id: r.user_id,
                        }
                        .to_string(),
                    )
                    .await;
            }

            //user-res-{view.user_id}-{role_op.id}
            if RbacRoleUserRange::User.eq(r.user_range)
                && RbacRoleResOpRange::AllowCustom.eq(r.res_op_range)
            {
                let user_ops = Select::type_new::<RbacRoleUserModel>()
                    .fetch_all_by_where::<RbacRoleUserModel, _>(
                        &WhereOption::Where(sql_format!(
                            "role_id={} and status={}",
                            r.id,
                            RbacRoleUserStatus::Enable
                        )),
                        &self.db,
                    )
                    .await?;
                for ru in user_ops.iter() {
                    for tmp in rops.iter() {
                        if tmp.role_id != r.id {
                            continue;
                        }
                        self.cache_access
                            .clear(&self.find_role_cache_key_by_user_res(
                                ru.user_id,
                                tmp.res_op_id,
                                0,
                            ))
                            .await;
                    }
                }
            }
        }

        //cache clean----------------------------

        Ok(())
    }
    /// 角色设置资源的操作
    pub async fn role_set_ops<'t>(
        &self,
        role: &RbacRoleModel,
        role_op_vec: &[RoleSetOp],
        set_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserRbacResult<()> {
        if !RbacRoleResOpRange::AllowCustom.eq(role.res_op_range) {
            return Err(UserRbacError::System(
                fluent_message!("rbac-res-op-range-wrong",{
                    "name":role.name,
                    "role_id":role.id,
                    "range":role.res_op_range
                }), //"role({$name})[range:{$range}] can't set ops [{$role_id}]",[
            ));
        }

        let time = now_time().unwrap_or_default();
        let db = &self.db;

        //确认资源都存在且属于当前角色用户
        let res_id = role_op_vec.iter().map(|e| e.res.id).collect::<Vec<_>>();
        let fres = if res_id.is_empty() {
            vec![]
        } else {
            Select::type_new::<RbacResModel>()
                .fetch_all_by_where::<RbacResModel, _>(
                    &WhereOption::Where(sql_format!(
                        "id in ({}) and status={}",
                        res_id,
                        RbacResStatus::Enable
                    )),
                    db,
                )
                .await?
        };
        //传入的res是否存都存着
        for tmp in role_op_vec.iter() {
            if !fres.iter().any(|e| e.id == tmp.res.id) {
                return Err(UserRbacError::System(
                    fluent_message!("rbac-role-miss-res",{
                        "id":tmp.res.id,
                        "name":tmp.res.name
                    }), //"res {$id} does not exist, may be delete",[
                ));
            }
        }
        for tmp in fres.iter() {
            //限制非系统角色不能加非本角色用户资源
            if role.user_id > 0 && tmp.user_id != role.user_id {
                return Err(UserRbacError::System(
                    fluent_message!("rbac-role-bad-res-user",{
                        "res":tmp.name,
                        "user_id":tmp.user_id
                    }), //,"res[{$res}:{$user_id}] do not belong to you"
                ));
            }
        }

        //确认权限操作ID是否正确
        let res_op_id = role_op_vec
            .iter()
            .flat_map(|e| e.res_op.iter().map(|t| t.0.id).collect::<Vec<u64>>())
            .collect::<Vec<_>>();
        let fops = if res_op_id.is_empty() {
            vec![]
        } else {
            Select::type_new::<RbacResOpModel>()
                .fetch_all_by_where::<RbacResOpModel, _>(
                    &WhereOption::Where(sql_format!(
                        "id in ({}) and status={}",
                        res_op_id,
                        RbacRoleOpStatus::Enable
                    )),
                    db,
                )
                .await?
        };
        for tmp in role_op_vec.iter() {
            for optmp in tmp.res_op.iter() {
                if let Some(res_op) = fops.iter().find(|e| e.id == optmp.0.id) {
                    if res_op.res_id != tmp.res.id {
                        //发现数据库中的res op 的res id 跟传入的res id 不一致
                        return Err(UserRbacError::System(
                            fluent_message!("rbac-role-wrong-res-op",{
                                "id":optmp.0.id,
                                "name":optmp.0.name,
                                "res_id":optmp.0.res_id,
                                "p_res_id":tmp.res.id
                            }), //"res op [{$id}:] res id not match [{$res_id}!={$p_res_id}] ",
                        ));
                    }
                } else {
                    //未发现数据库中的res op
                    return Err(UserRbacError::System(
                        fluent_message!("rbac-role-miss-res-op",{
                            "id":optmp.0.id,
                            "name":optmp.0.name
                        }), //"res op {$id} does not exist, may be delete",[
                    ));
                }
            }
        }

        //确认需要移除的权限
        let rops = Select::type_new::<RbacRoleOpModel>()
            .fetch_all_by_where::<RbacRoleOpModel, _>(
                &WhereOption::Where(sql_format!(
                    "role_id={} and status={}",
                    role.id,
                    RbacRoleOpStatus::Enable,
                )),
                db,
            )
            .await?;

        let mut del_op = vec![];
        for iop in rops.iter() {
            let mut find = false;
            for res_opi in role_op_vec.iter() {
                for (res_opt, res_op_positivity) in res_opi.res_op.iter() {
                    if res_opt.id == iop.res_op_id && (*res_op_positivity as i8) == iop.positivity {
                        find = true;
                        break;
                    }
                }
            }
            if !find {
                del_op.push((iop.id, iop.res_op_id));
            }
        }

        let tmp_id = rops.iter().map(|e| e.res_op_id).collect::<Vec<_>>();
        let mut add_item = vec![];
        for res_opi in role_op_vec.iter() {
            for (res_opt, res_op_positivity) in res_opi.res_op.iter() {
                if tmp_id.contains(&res_opt.id) {
                    continue;
                }
                add_item.push((res_opt.id, res_op_positivity.to_owned() as i8));
            }
        }
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        if !add_item.is_empty() {
            let mut add_op = Vec::with_capacity(add_item.len());
            for (opid, oppo) in add_item.iter() {
                let mut item = model_option_set!(RbacRoleOpModelRef,{
                    role_id:role.id,
                    change_time:time,
                    change_user_id:set_user_id,
                    status:(RbacRoleOpStatus::Enable as i8),
                });
                item.res_op_id = Some(opid);
                item.positivity = Some(oppo);
                add_op.push(item);
            }

            let tmp = Insert::<sqlx::MySql, RbacRoleOpModel, _>::new_vec(add_op)
                .execute(&mut db)
                .await;
            if let Err(e) = tmp {
                db.rollback().await?;
                return Err(e)?;
            };
        }

        if !del_op.is_empty() {
            let ddata = model_option_set!(RbacRoleOpModelRef,{
                change_user_id:set_user_id,
                change_time:time,
                status:(RbacRoleOpStatus::Delete as i8),
            });
            let tmp = Update::<sqlx::MySql, RbacRoleOpModel, _>::new(ddata)
                .execute_by_where(
                    &WhereOption::Where(sql_format!(
                        "id in ({})",
                        del_op.iter().map(|e| e.0).collect::<Vec<u64>>()
                    )),
                    &mut db,
                )
                .await;
            if let Err(e) = tmp {
                db.rollback().await?;
                return Err(e)?;
            }
        }

        db.commit().await?;

        //cache clean----------------------------
        //public-res-{RbacRoleUserRange}-{yaf_rbac_role_op.op_id}
        for (_, op_id) in del_op.iter() {
            if RbacRoleUserRange::AllUser.eq(role.user_range)
                || RbacRoleUserRange::Login.eq(role.user_range)
            {
                self.cache_access
                    .clear(&self.find_role_cache_key_by_public_res(role.user_range, *op_id, 0))
                    .await;
            }
        }
        for (op_id, _) in add_item.iter() {
            if RbacRoleUserRange::AllUser.eq(role.user_range)
                || RbacRoleUserRange::Login.eq(role.user_range)
            {
                self.cache_access
                    .clear(&self.find_role_cache_key_by_public_res(role.user_range, *op_id, 0))
                    .await;
            }
        }
        //access-relation-{role.user_id}-{relation_key}
        if RbacRoleUserRange::Relation.eq(role.user_range) {
            self.cache_relation
                .clear(
                    &RoleRelationKey {
                        relation_key: role.relation_key.clone(),
                        user_id: role.user_id,
                    }
                    .to_string(),
                )
                .await;
        }
        //user-res-{view.user_id}-{role_op.id}
        if RbacRoleUserRange::User.eq(role.user_range)
            && RbacRoleResOpRange::AllowCustom.eq(role.res_op_range)
        {
            let user_ops = Select::type_new::<RbacRoleUserModel>()
                .fetch_all_by_where::<RbacRoleUserModel, _>(
                    &WhereOption::Where(sql_format!(
                        "role_id={} and status={}",
                        role.id,
                        RbacRoleUserStatus::Enable
                    )),
                    &self.db,
                )
                .await?;
            for ru in user_ops.iter() {
                for (op_id, _) in add_item.iter() {
                    self.cache_access
                        .clear(&self.find_role_cache_key_by_user_res(ru.user_id, *op_id, 0))
                        .await;
                }
            }
            for ru in user_ops.iter() {
                for (_, op_id) in del_op.iter() {
                    self.cache_access
                        .clear(&self.find_role_cache_key_by_user_res(ru.user_id, *op_id, 0))
                        .await;
                }
            }
        }
        //cache clean----------------------------

        self.logger
            .add(
                &LogRoleOp {
                    name: role.name.to_owned(),
                    role_op_vec: role_op_vec.to_owned(),
                },
                &Some(role.id),
                &Some(role.user_id),
                &Some(set_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
    /// 获取指定关系KEY的角色详细数据
    pub async fn find_role_detail_by_relation_key(
        &self,
        relation_role: &[RoleRelationKey],
    ) -> UserRbacResult<Vec<RoleDetailRow>> {
        if relation_role.is_empty() {
            return Ok(vec![]);
        }
        let mut where_sql = Vec::with_capacity(relation_role.len());
        for rkey in relation_role {
            where_sql.push(sql_format!(
                "(relation_key ={} and user_id={})",
                rkey.user_id,
                rkey.relation_key
            ));
        }
        let sql = sql_format!(
            "select role.* from {} as role
            where role.status ={} and role.user_range={} and ({})",
            RbacRoleModel::table_name(),
            RbacRoleStatus::Enable,
            RbacRoleUserRange::Relation,
            SqlExpr(where_sql.join(" or ")),
        );
        //查询出指定关系角色的列表
        let roles = sqlx::query_as::<_, RbacRoleModel>(sql.as_str())
            .fetch_all(&self.db)
            .await?;
        //当指定关系角色的类型为`自定义`,根据关系表查询出对应配置数据
        let role_op = if !roles.is_empty() {
            let res_id = roles
                .iter()
                .filter(|e| RbacRoleResOpRange::AllowCustom.eq(e.res_op_range))
                .map(|res| res.id)
                .collect::<Vec<_>>();
            if res_id.is_empty() {
                vec![]
            } else {
                Select::type_new::<RbacRoleOpModel>()
                    .fetch_all_by_where::<RbacRoleOpModel, _>(
                        &sqlx_model::WhereOption::Where(sql_format!(
                            "role_id in ({}) and status ={} order by id desc",
                            res_id,
                            RbacRoleOpStatus::Enable
                        )),
                        &self.db,
                    )
                    .await?
            }
        } else {
            vec![]
        };
        let mut out = Vec::with_capacity(roles.len());
        for role_ in roles.into_iter() {
            let mut role_ops = vec![];
            for orole_ in role_op.iter() {
                if orole_.role_id != role_.id {
                    continue;
                }
                role_ops.push(orole_.to_owned());
            }
            out.push(RoleDetailRow {
                role: role_,
                role_ops,
            });
        }
        Ok(out)
    }
    async fn filter_relation_role(
        &self,
        role_data: Vec<RoleDetailRow>,
        check_vec: &[RbacResData],
    ) -> RoleCheckData {
        let mut relation_data =
            Vec::with_capacity(check_vec.iter().fold(0, |acc, res| acc + res.ops.len()));
        for check_item in check_vec.iter() {
            for res_op in check_item.ops.iter() {
                let mut tmp = vec![];
                for RoleDetailRow { role, role_ops } in role_data.iter() {
                    for role_res_op in role_ops.iter() {
                        //为每个资源操作分配角色
                        if (role.user_id == 0//系统角色的全局授权或禁止
                        && (RbacRoleResOpRange::AllowAll.eq(role.res_op_range)
                        || RbacRoleResOpRange::DenyAll.eq(role.res_op_range)))
                            || (role.user_id == check_item.res.user_id //用户角色 授权或禁止
                           && RbacRoleResOpRange::AllowAll.eq(role.res_op_range)
                                || RbacRoleResOpRange::DenyAll.eq(role.res_op_range))
                            || (RbacRoleResOpRange::AllowCustom.eq(role.res_op_range)  //系统或用户指定操作授权
                           && res_op.id == role_res_op.res_op_id)
                        {
                            let positivity = RbacRoleOpPositivity::try_from(role_res_op.positivity)
                                .unwrap_or(RbacRoleOpPositivity::Allow);
                            tmp.push((role.clone(), positivity))
                        }
                    }
                }
                tmp.sort_by(|a, b| a.0.priority.cmp(&b.0.priority));
                relation_data.push(RoleCheckRow::ModelRole {
                    role: tmp.first().map(|e| e.to_owned()),
                    res_op_id: res_op.id,
                });
            }
        }
        RoleCheckData::new(relation_data)
    }
    //根据关系key获取待检测角色数据
    pub(crate) async fn find_role_by_relation(
        &self,
        relation_role: &[RoleRelationKey],
        check_vec: &[RbacResData],
    ) -> UserRbacResult<RoleCheckData> {
        let relaction_res = self.find_role_detail_by_relation_key(relation_role).await?;
        Ok(self.filter_relation_role(relaction_res, check_vec).await)
    }
    fn find_role_cache_key_by_public_res(
        &self,
        user_range: i8,
        op_id: u64,
        res_user_id: u64,
    ) -> String {
        format!("public-res-{}-{}-{}", user_range, op_id, res_user_id)
    }
    //指定用户类型[游客或登录用户]的系统层面 的 指定授权
    fn find_role_sql_by_public_res(&self, user_range: i8, op_id: u64, role_user_id: u64) -> String {
        sql_format!(
            r#"SELECT CONVERT(rop.res_op_id,UNSIGNED) as res_op_id,rop.positivity as positivity,ro.*
                FROM {rbac_role} as ro 
                join {rbac_role_op} as rop 
                on  ro.user_range={role_user_range} and ro.status ={role_status} and  ro.res_op_range={role_res_op_range} 
                    and rop.status ={role_op_status} and rop.res_op_id = {role_op_id} and ro.user_id={user_id} and ro.id =rop.role_id
                order by ro.priority desc,ro.id desc  limit 1"#,
            rbac_role = RbacRoleModel::table_name(),
            rbac_role_op = RbacRoleOpModel::table_name(),
            role_user_range = user_range,
            role_status = RbacRoleStatus::Enable,
            role_res_op_range = RbacRoleResOpRange::AllowCustom,
            role_op_status = RbacRoleOpStatus::Enable,
            role_op_id = op_id,
            user_id = role_user_id
        )
    }
    fn find_role_cache_key_by_public_global(&self, user_range: i8, res_user_id: u64) -> String {
        format!("public-global-{}-{}", user_range, res_user_id)
    }
    //res_user_id=0 指定用户类型[游客或登录用户]的系统层面 的 全局授权或全局禁止
    //res_user_id>0 指定用户类型[游客或登录用户]在对该指定用户的资源,进行全部授权或全部禁止
    fn find_role_sql_by_public_global(&self, user_range: i8, role_user_id: u64) -> String {
        //非AllowCustom时 positivity 仅填充,无实际作用
        sql_format!(
            r#"SELECT CONVERT(0,UNSIGNED) as res_op_id,0 as positivity,ro.*
            FROM {rbac_role} as ro WHERE ro.user_range={role_user_range} and ro.status ={role_status} and  ro.res_op_range IN ({role_res_op_range}) and ro.user_id={user_id}
            order by ro.priority desc,ro.id desc  limit 1  "#,
            rbac_role = RbacRoleModel::table_name(),
            role_user_range = user_range,
            role_status = RbacRoleStatus::Enable,
            role_res_op_range = [
                RbacRoleResOpRange::AllowAll.sql_quote(),
                RbacRoleResOpRange::DenyAll.sql_quote()
            ],
            user_id = role_user_id
        )
    }

    async fn find_role_by_public(
        &self,
        user_range: RbacRoleUserRange,
        check_vec: &[RbacResData],
    ) -> UserRbacResult<RoleCheckData> {
        if check_vec.is_empty() {
            return Ok(RoleCheckData::new(vec![]));
        }
        let mut sqls = Vec::with_capacity(
            check_vec.iter().fold(0, |acc, res| acc + res.ops.len()) + check_vec.len() + 1,
        );
        sqls.push(self.find_role_sql_by_public_global(user_range as i8, 0));
        for res in check_vec {
            sqls.push(self.find_role_sql_by_public_global(user_range as i8, res.res.user_id));
            for role_op in &res.ops {
                sqls.push(self.find_role_sql_by_public_res(user_range as i8, role_op.id, 0));
                sqls.push(self.find_role_sql_by_public_res(
                    user_range as i8,
                    role_op.id,
                    res.res.user_id,
                ));
            }
        }
        self.filter_find_role(self.find_role_by_sqls(sqls, false).await?, check_vec)
    }
    //获取游客的待检测角色数据
    pub(crate) async fn find_role_by_all_user(
        &self,
        check_vec: &[RbacResData],
    ) -> UserRbacResult<RoleCheckData> {
        self.find_role_by_public(RbacRoleUserRange::AllUser, check_vec)
            .await
    }
    //获取登陆用户的待检测角色数据
    pub(crate) async fn find_role_by_login_user(
        &self,
        check_vec: &[RbacResData],
    ) -> UserRbacResult<RoleCheckData> {
        self.find_role_by_public(RbacRoleUserRange::Login, check_vec)
            .await
    }
    async fn find_role_by_sqls(
        &self,
        sqls: Vec<String>,
        exist_timeout: bool,
    ) -> UserRbacResult<Vec<RoleAccessRow>> {
        Ok(sqlx::query(&format!(
            "select * from (({})) as t",
            sqls.join(") union all (")
        ))
        .try_map(
            |row: sqlx::mysql::MySqlRow| match RbacRoleModel::from_row(&row) {
                Ok(role) => {
                    let range = RbacRoleResOpRange::try_from(role.res_op_range)
                        .unwrap_or(RbacRoleResOpRange::AllowCustom);
                    let op_positivity = match range {
                        RbacRoleResOpRange::DenyAll => RbacRoleOpPositivity::Deny,
                        RbacRoleResOpRange::AllowAll => RbacRoleOpPositivity::Allow,
                        RbacRoleResOpRange::AllowCustom => RbacRoleOpPositivity::try_from(
                            row.try_get::<i8, &str>("positivity")
                                .unwrap_or(RbacRoleOpPositivity::Allow as i8),
                        )
                        .unwrap_or(RbacRoleOpPositivity::Allow),
                    };
                    let timeout = if exist_timeout {
                        row.try_get::<u64, &str>("timeout").unwrap_or(0)
                    } else {
                        0
                    };
                    let res_op_id = match row.try_get::<u64, &str>("res_op_id") {
                        Ok(id) => id,
                        Err(err) => {
                            // dbg!("{:?}", err);
                            error!(
                                "find_role_by_sqls get res_op_id fail:{:?} on id :{}",
                                err, role.id
                            );
                            0
                        }
                    };
                    Ok(RoleAccessRow {
                        role,
                        res_op_id,
                        op_positivity,
                        timeout,
                    })
                }
                Err(err) => Err(err),
            },
        )
        .fetch_all(&self.db)
        .await?)
    }
    fn filter_find_role(
        &self,
        find_role_data: Vec<RoleAccessRow>,
        check_vec: &[RbacResData],
    ) -> UserRbacResult<RoleCheckData> {
        debug!("filter role :{:?} on check {:?}", find_role_data, check_vec);
        let res_op_len = check_vec.iter().fold(0, |acc, res| acc + res.ops.len());
        let mut out = Vec::with_capacity(res_op_len);

        for check_item in check_vec {
            for res_op in &check_item.ops {
                let mut tmp = vec![];
                for RoleAccessRow {
                    role,
                    res_op_id,
                    op_positivity,
                    timeout: _,
                } in find_role_data.iter()
                {
                    //为每个资源操作分配角色
                    if (role.user_id == 0//系统角色的全局授权或禁止
                         && (RbacRoleResOpRange::AllowAll.eq(role.res_op_range)
                         || RbacRoleResOpRange::DenyAll.eq(role.res_op_range)))
                        || (role.user_id == check_item.res.user_id //用户角色 授权或禁止
                            && RbacRoleResOpRange::AllowAll.eq(role.res_op_range)
                            || RbacRoleResOpRange::DenyAll.eq(role.res_op_range))
                        || (RbacRoleResOpRange::AllowCustom.eq(role.res_op_range)  //系统或用户指定操作授权
                            && res_op.id == *res_op_id)
                    {
                        tmp.push((role.to_owned(), op_positivity.to_owned()))
                    }
                }
                tmp.sort_by(|a, b| a.0.priority.cmp(&b.0.priority));
                out.push(RoleCheckRow::ModelRole {
                    role: tmp.first().map(|e| e.to_owned()),
                    res_op_id: res_op.id,
                });
            }
        }
        Ok(RoleCheckData::new(out))
    }

    fn find_role_cache_key_by_user_res(
        &self,
        user_id: u64,
        op_id: u64,
        role_user_id: u64,
    ) -> String {
        format!("user-res-{}-{}-{}", user_id, op_id, role_user_id)
    }
    fn find_role_sql_by_user_res(&self, user_id: u64, op_id: u64, role_user_id: u64) -> String {
        let time = now_time().unwrap_or(0);
        sql_format!(
            r#"
            SELECT CONVERT(rop.res_op_id,UNSIGNED) as res_op_id,rop.positivity as positivity,ro.*,ru.timeout
            FROM {rbac_role}  as ro 
            join {rbac_role_user} as ru on ro.user_range={role_user_range} and ro.status ={role_status} and ro.res_op_range={role_res_op_range} and ro.user_id={role_user_id}
                and ru.status ={role_user_status} and ru.user_id = {role_user_user_id}  and (ru.timeout>{timeout} or ru.timeout=0) and ro.id =ru.role_id
            join {rbac_role_op} as rop on rop.status ={role_op_status} and rop.res_op_id ={role_op_id}  and ro.id =rop.role_id
            order by ro.priority desc,ro.id desc  limit 1 "#,
            rbac_role = RbacRoleModel::table_name(),
            rbac_role_user = RbacRoleUserModel::table_name(),
            rbac_role_op = RbacRoleOpModel::table_name(),
            role_user_range = RbacRoleUserRange::User,
            role_status = RbacRoleStatus::Enable,
            role_res_op_range = RbacRoleResOpRange::AllowCustom,
            role_op_status = RbacRoleOpStatus::Enable,
            role_op_id = op_id,
            role_user_status = RbacRoleUserStatus::Enable,
            role_user_user_id = user_id,
            timeout = time,
            role_user_id = role_user_id,
        )
    }
    fn find_role_cache_key_by_user_global(&self, user_id: u64, role_user_id: u64) -> String {
        format!("user-global-{}-{}", user_id, role_user_id)
    }
    fn find_role_sql_by_user_global(&self, user_id: u64, role_user_id: u64) -> String {
        //非AllowCustom时 positivity 仅填充,无实际作用
        let time = now_time().unwrap_or(0);
        sql_format!(
            r#"
            SELECT CONVERT(0,UNSIGNED) as res_op_id,0 as positivity,ro.*,ru.timeout
            FROM {rbac_role}  as ro 
                join {rbac_role_user} as ru on ro.user_id={role_user_id} and ro.user_range={role_user_range} and ro.status ={role_status} and ro.res_op_range IN ({role_res_op_range}) 
                and ru.status ={role_user_status} and ru.user_id =  {role_user_user_id} and (ru.timeout>{timeout} or ru.timeout=0) and ro.id =ru.role_id 
            order by ro.priority desc,ro.id desc  limit 1  
             "#,
            rbac_role = RbacRoleModel::table_name(),
            rbac_role_user = RbacRoleUserModel::table_name(),
            role_user_range = RbacRoleUserRange::User,
            role_status = RbacRoleStatus::Enable,
            role_res_op_range = [
                RbacRoleResOpRange::AllowAll.sql_quote(),
                RbacRoleResOpRange::DenyAll.sql_quote()
            ],
            role_user_status = RbacRoleUserStatus::Enable,
            role_user_user_id = user_id,
            role_user_id = role_user_id,
            timeout = time
        )
    }

    /// 获取指定用户的待检测角色数据
    /// 并根据资源的优先级排序结果
    // 公开角色：资源key->资源id反查->角色且公开【纬度由访问用户决定】 得到资源数据库角色（建这样公开角色时自动建资源）
    // 用户-角色关系配置角色：访问用户通过角色配置得到数据库角色
    pub(crate) async fn find_role_by_user(
        &self,
        user_id: u64,
        check_vec: &[RbacResData],
    ) -> UserRbacResult<RoleCheckData> {
        if check_vec.is_empty() {
            return Ok(RoleCheckData::new(vec![]));
        }
        let mut sqls = Vec::with_capacity(
            check_vec.iter().fold(0, |acc, res| acc + res.ops.len()) + check_vec.len() + 1,
        );

        sqls.push(self.find_role_sql_by_user_global(user_id, 0));
        for res in check_vec {
            sqls.push(self.find_role_sql_by_user_global(user_id, res.res.user_id));
            for role_op in &res.ops {
                sqls.push(self.find_role_sql_by_user_res(user_id, role_op.id, 0));
                sqls.push(self.find_role_sql_by_user_res(user_id, role_op.id, res.res.user_id));
            }
        }
        self.filter_find_role(self.find_role_by_sqls(sqls, true).await?, check_vec)
    }
    pub fn cache(&self) -> RbacRoleCache<'_> {
        RbacRoleCache { role: self }
    }
}

pub struct RbacRoleCache<'t> {
    role: &'t RbacRole,
}

impl<'t> RbacRoleCache<'t> {
    pub(crate) async fn find_role_by_relation(
        &self,
        relation_role: &[RoleRelationKey],
        check_vec: &[RbacResData],
    ) -> UserRbacResult<RoleCheckData> {
        let mut get = vec![];
        let mut hash = std::collections::HashMap::with_capacity(relation_role.len());
        for id in relation_role {
            match self.role.cache_relation.get(&id.to_string()).await {
                Some(data) => {
                    hash.entry(id.to_string()).or_insert(data);
                }
                None => {
                    get.push(id.to_owned());
                }
            }
        }
        if !get.is_empty() {
            match self.role.find_role_detail_by_relation_key(&get).await {
                Ok(datas) => {
                    for row in datas.into_iter() {
                        let pk = RoleRelationKey {
                            relation_key: row.role.relation_key.clone(),
                            user_id: row.role.user_id,
                        }
                        .to_string();
                        self.role
                            .cache_relation
                            .set(pk.clone(), Some(row.clone()), 0)
                            .await;
                        hash.entry(pk).or_insert(Some(row));
                    }
                }
                Err(err) => return Err(err),
            }
            for pk in get {
                let pks = pk.to_string();
                if !hash.contains_key(&pks) {
                    self.role.cache_relation.set(pks.clone(), None, 0).await;
                }
            }
        }
        Ok(self
            .role
            .filter_relation_role(
                hash.into_values().flatten().collect::<Vec<RoleDetailRow>>(),
                check_vec,
            )
            .await)
    }
    async fn find_role_by_public(
        &self,
        user_range: RbacRoleUserRange,
        check_vec: &[RbacResData],
    ) -> UserRbacResult<RoleCheckData> {
        if check_vec.is_empty() {
            return Ok(RoleCheckData::new(vec![]));
        }

        let mut access_data = vec![];
        let mut sqls = vec![];
        let global_key = self
            .role
            .find_role_cache_key_by_public_global(user_range as i8, 0);
        let global_keys = self.role.cache_access.get(&global_key).await;
        match &global_keys {
            Some(data) => {
                if let Some(tmp) = data {
                    access_data = vec![tmp.to_owned()];
                }
            }
            None => {
                sqls.push(
                    self.role
                        .find_role_sql_by_public_global(user_range as i8, 0),
                );
            }
        }
        let mut global_user_keys = HashMap::new();
        let mut res_keys = HashMap::new();
        let mut res_user_keys = HashMap::new();
        for res in check_vec {
            let user_key = self
                .role
                .find_role_cache_key_by_public_global(user_range as i8, res.res.user_id);
            match self.role.cache_access.get(&user_key).await {
                Some(data) => {
                    if let Some(tmp) = data {
                        access_data.push(tmp);
                    }
                }
                None => {
                    global_user_keys.entry(user_key).or_insert(res.res.user_id);
                    sqls.push(
                        self.role
                            .find_role_sql_by_public_global(user_range as i8, res.res.user_id),
                    );
                }
            }

            for role_op in &res.ops {
                let res_key =
                    self.role
                        .find_role_cache_key_by_public_res(user_range as i8, role_op.id, 0);
                match self.role.cache_access.get(&res_key).await {
                    Some(data) => {
                        if let Some(tmp) = data {
                            access_data.push(tmp);
                        }
                    }
                    None => {
                        res_keys.entry(res_key).or_insert(role_op.id);
                        sqls.push(self.role.find_role_sql_by_public_res(
                            user_range as i8,
                            role_op.id,
                            0,
                        ));
                    }
                }
                let res_user_key = self.role.find_role_cache_key_by_public_res(
                    user_range as i8,
                    role_op.id,
                    res.res.user_id,
                );
                match self.role.cache_access.get(&res_user_key).await {
                    Some(data) => {
                        if let Some(tmp) = data {
                            access_data.push(tmp);
                        }
                    }
                    None => {
                        res_user_keys.entry(res_user_key).or_insert(role_op.id);
                        sqls.push(self.role.find_role_sql_by_public_res(
                            user_range as i8,
                            role_op.id,
                            res.res.user_id,
                        ));
                    }
                }
            }
        }

        if !sqls.is_empty() {
            let data = self.role.find_role_by_sqls(sqls, false).await?;
            if global_keys.is_none() {
                let tmp = data
                    .iter()
                    .find(|e| {
                        e.role.user_id == 0
                            && (RbacRoleResOpRange::AllowAll.eq(e.role.res_op_range)
                                || RbacRoleResOpRange::DenyAll.eq(e.role.res_op_range))
                    })
                    .map(|e| e.to_owned());
                self.role.cache_access.set(global_key, tmp, 0).await;
            }
            for tkey in res_keys {
                let tmp = data
                    .iter()
                    .find(|e| {
                        e.role.user_id == 0
                            && (RbacRoleResOpRange::AllowCustom.eq(e.role.res_op_range)
                                && e.res_op_id == tkey.1)
                    })
                    .map(|e| e.to_owned());
                self.role.cache_access.set(tkey.0, tmp, 0).await;
            }
            for tkey in global_user_keys {
                let tmp = data
                    .iter()
                    .find(|e| {
                        e.role.user_id > 0
                            && (RbacRoleResOpRange::AllowAll.eq(e.role.res_op_range)
                                || RbacRoleResOpRange::DenyAll.eq(e.role.res_op_range))
                    })
                    .map(|e| e.to_owned());
                self.role.cache_access.set(tkey.0, tmp, 0).await;
            }
            for tkey in res_user_keys {
                let tmp = data
                    .iter()
                    .find(|e| {
                        e.role.user_id > 0
                            && (RbacRoleResOpRange::AllowCustom.eq(e.role.res_op_range)
                                && e.res_op_id == tkey.1)
                    })
                    .map(|e| e.to_owned());
                self.role.cache_access.set(tkey.0, tmp, 0).await;
            }
            access_data.extend(data);
        }
        self.role.filter_find_role(access_data, check_vec)
    }
    pub(crate) async fn find_role_by_all_user(
        &self,
        check_vec: &[RbacResData],
    ) -> UserRbacResult<RoleCheckData> {
        self.find_role_by_public(RbacRoleUserRange::AllUser, check_vec)
            .await
    }
    pub(crate) async fn find_role_by_login_user(
        &self,
        check_vec: &[RbacResData],
    ) -> UserRbacResult<RoleCheckData> {
        self.find_role_by_public(RbacRoleUserRange::Login, check_vec)
            .await
    }
    pub(crate) async fn find_role_by_user(
        &self,
        user_id: u64,
        check_vec: &[RbacResData],
    ) -> UserRbacResult<RoleCheckData> {
        if check_vec.is_empty() {
            return Ok(RoleCheckData::new(vec![]));
        }

        let mut access_data = vec![];

        let mut sqls = vec![];
        let global_key = self.role.find_role_cache_key_by_user_global(user_id, 0);
        let global_keys = self.role.cache_access.get(&global_key).await;

        match &global_keys {
            Some(data) => {
                if let Some(tmp) = data {
                    access_data = vec![tmp.to_owned()];
                }
            }
            None => {
                sqls.push(self.role.find_role_sql_by_user_global(user_id, 0));
            }
        }
        let mut global_user_keys = HashMap::new();
        let mut res_keys = HashMap::new();
        let mut res_user_keys = HashMap::new();

        for res in check_vec {
            let global_user_key = self
                .role
                .find_role_cache_key_by_user_global(user_id, res.res.user_id);
            match self.role.cache_access.get(&global_user_key).await {
                Some(data) => {
                    if let Some(tmp) = data {
                        access_data.push(tmp);
                    }
                }
                None => {
                    global_user_keys
                        .entry(global_user_key)
                        .or_insert(res.res.user_id);
                    sqls.push(
                        self.role
                            .find_role_sql_by_user_global(user_id, res.res.user_id),
                    )
                }
            }

            for role_op in &res.ops {
                let res_key = self
                    .role
                    .find_role_cache_key_by_user_res(user_id, role_op.id, 0);
                match self.role.cache_access.get(&res_key).await {
                    Some(data) => {
                        if let Some(tmp) = data {
                            access_data.push(tmp);
                        }
                    }
                    None => {
                        res_keys.entry(res_key).or_insert(role_op.id);
                        sqls.push(self.role.find_role_sql_by_user_res(user_id, role_op.id, 0));
                    }
                }
                let res_user_key =
                    self.role
                        .find_role_cache_key_by_user_res(user_id, role_op.id, res.res.user_id);
                match self.role.cache_access.get(&res_user_key).await {
                    Some(data) => {
                        if let Some(tmp) = data {
                            access_data.push(tmp);
                        }
                    }
                    None => {
                        res_user_keys.entry(res_user_key).or_insert(role_op.id);
                        sqls.push(self.role.find_role_sql_by_user_res(
                            user_id,
                            role_op.id,
                            res.res.user_id,
                        ));
                    }
                }
            }
        }

        let nowtime = now_time().unwrap_or(0);
        if !sqls.is_empty() {
            let data = self.role.find_role_by_sqls(sqls, true).await?;
            if global_keys.is_none() {
                let mut set_time = 0;
                let tmp = data
                    .iter()
                    .find(|e| {
                        if e.role.user_id == 0
                            && (RbacRoleResOpRange::AllowAll.eq(e.role.res_op_range)
                                || RbacRoleResOpRange::DenyAll.eq(e.role.res_op_range))
                        {
                            if e.timeout > 0 && e.timeout < set_time {
                                set_time = e.timeout;
                            }
                            true
                        } else {
                            false
                        }
                    })
                    .map(|e| e.to_owned());
                if set_time == 0 || set_time > nowtime {
                    self.role
                        .cache_access
                        .set(
                            global_key,
                            tmp,
                            if set_time > nowtime {
                                set_time - nowtime
                            } else {
                                0
                            },
                        )
                        .await;
                }
            }
            for tkey in global_user_keys {
                let mut set_time = 0;
                let tmp = data
                    .iter()
                    .find(|e| {
                        if e.role.user_id > 0
                            && (RbacRoleResOpRange::AllowAll.eq(e.role.res_op_range)
                                || RbacRoleResOpRange::DenyAll.eq(e.role.res_op_range))
                            && e.role.user_id == tkey.1
                        {
                            if e.timeout > 0 && e.timeout < set_time {
                                set_time = e.timeout;
                            }
                            true
                        } else {
                            false
                        }
                    })
                    .map(|e| e.to_owned());
                if set_time == 0 || set_time > nowtime {
                    self.role
                        .cache_access
                        .set(
                            tkey.0,
                            tmp,
                            if set_time > nowtime {
                                set_time - nowtime
                            } else {
                                0
                            },
                        )
                        .await;
                }
            }

            for tkey in res_keys {
                let mut set_time = 0;
                let tmp = data
                    .iter()
                    .find(|e| {
                        if e.role.user_id == 0
                            && RbacRoleResOpRange::AllowCustom.eq(e.role.res_op_range)
                            && e.res_op_id == tkey.1
                        {
                            if e.timeout > 0 && e.timeout < set_time {
                                set_time = e.timeout;
                            }
                            true
                        } else {
                            false
                        }
                    })
                    .map(|e| e.to_owned());
                if set_time == 0 || set_time > nowtime {
                    self.role
                        .cache_access
                        .set(
                            tkey.0,
                            tmp,
                            if set_time > nowtime {
                                set_time - nowtime
                            } else {
                                0
                            },
                        )
                        .await;
                }
            }

            for tkey in res_user_keys {
                let mut set_time = 0;
                let tmp = data
                    .iter()
                    .find(|e| {
                        if e.role.user_id > 0
                            && RbacRoleResOpRange::AllowCustom.eq(e.role.res_op_range)
                            && e.res_op_id == tkey.1
                        {
                            if e.timeout > 0 && e.timeout < set_time {
                                set_time = e.timeout;
                            }
                            true
                        } else {
                            false
                        }
                    })
                    .map(|e| e.to_owned());
                if set_time == 0 || set_time > nowtime {
                    self.role
                        .cache_access
                        .set(
                            tkey.0,
                            tmp,
                            if set_time > nowtime {
                                set_time - nowtime
                            } else {
                                0
                            },
                        )
                        .await;
                }
            }
            access_data.extend(data);
        }
        self.role.filter_find_role(access_data, check_vec)
    }
}
