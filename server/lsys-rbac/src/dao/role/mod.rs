mod access;
mod cache;
mod data;
pub(crate) mod logger;
mod perm;
mod user;
//RBAC中角色相关实现
use logger::LogRole;
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::fluent_message;
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::utils::{now_time, RequestEnv};
use lsys_core::valid_key;
use lsys_core::valid_param::{ValidParam, ValidParamCheck, ValidPattern, ValidStrlen};
use sqlx::Acquire;
use std::{sync::Arc, vec};

use lsys_core::db::{
    utils::FetchField, Insert, OptionTxExecutor, QueryBuilderExt, TableMeta, Update,
};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool, QueryBuilder, Transaction};

use crate::model::{
    RbacPermModel, RbacPermStatus, RbacRoleModel, RbacRoleResRange, RbacRoleStatus,
    RbacRoleUserModel, RbacRoleUserRange, RbacRoleUserStatus,
};

use super::result::{RbacError, RbacResult};
pub use access::AccessResInfo;
pub use access::AccessRoleData;
pub use access::AccessRoleInfo;
pub use access::AccessRoleRow;
pub use data::*;
pub use perm::*;
pub use user::*;

//角色相关操作的实现

//角色管理
pub struct RbacRole {
    db: Pool<MySql>,
    pub(crate) cache_access: Arc<LocalCache<String, Vec<AccessRoleRow>>>,
    logger: Arc<ChangeLoggerDao>,
}

impl RbacRole {
    pub fn new(
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        role_config: LocalCacheConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        Self {
            cache_access: Arc::from(LocalCache::new(remote_notify, role_config)),
            db,
            logger,
        }
    }
}

pub enum RbacRoleUserRangeData<'t> {
    Session {
        role_key: &'t str,
        role_name: Option<&'t str>,
    },
    Custom {
        role_name: &'t str,
    },
}

pub struct RbacRoleAddData<'t> {
    pub user_id: u64,
    pub app_id: Option<u64>,
    pub role_info: RbacRoleUserRangeData<'t>,
    pub res_range: RbacRoleResRange,
}

impl RbacRole {
    async fn role_param_valid(&self, param: &RbacRoleUserRangeData<'_>) -> RbacResult<()> {
        let fetch_field = FetchField::new(&self.db);
        let role_key_max =
           fetch_field.string_max::<RbacRoleModel>( &RbacRoleModel::ROLE_KEY)
                .await
                .len_or(32);
        let role_name_max =
           fetch_field.string_max::<RbacRoleModel>( &RbacRoleModel::ROLE_NAME)
                .await
                .len_or(32);

        let mut param_valid = ValidParam::default();
        match param {
            RbacRoleUserRangeData::Session {
                role_key,
                role_name,
            } => {
                param_valid.add(
                    valid_key!("role_key"),
                    role_key,
                    &ValidParamCheck::default()
                        .add_rule(ValidPattern::Ident)
                        .add_rule(ValidStrlen::range(1, role_key_max)),
                );
                if let Some(name) = role_name {
                    param_valid.add(
                        valid_key!("role_name"),
                        name,
                        &ValidParamCheck::default()
                            .add_rule(ValidPattern::NotFormat)
                            .add_rule(ValidStrlen::range(0, role_name_max)),
                    );
                }
            }
            RbacRoleUserRangeData::Custom { role_name } => {
                param_valid.add(
                    valid_key!("role_name"),
                    role_name,
                    &ValidParamCheck::default()
                        .add_rule(ValidPattern::NotFormat)
                        .add_rule(ValidStrlen::range(1, role_name_max)),
                );
            }
        };
        param_valid.check()?;
        Ok(())
    }
    //添加角色
    pub async fn add_role(
        &self,
        param: &RbacRoleAddData<'_>,
        add_user_id: u64,
        mut transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<RbacRoleModel> {
        self.role_param_valid(&param.role_info).await?;
        let (user_range, role_key, role_name, res) = match param.role_info {
            RbacRoleUserRangeData::Session {
                role_key,
                role_name,
            } => {
                let role_name = role_name.map(|e| e.to_owned()).unwrap_or_default();
                let role_key = role_key.to_owned();

                let mut qb = QueryBuilder::<MySql>::new(format!(
                    "select * from ((select * from {}",
                    RbacRoleModel::table_name(),
                ));
                qb.push_where().field_eq("user_id", param.user_id);
                qb.push_and().field_eq("role_key", role_key.clone());
                qb.push_and().field_eq("app_id", param.app_id.unwrap_or_default());
                qb.push_and().field_eq("status", RbacRoleStatus::Enable as i8).push(" limit 1");
                if !role_name.is_empty() {
                    qb.push(format!(
                        ") union all  (select * from {}",
                        RbacRoleModel::table_name(),
                    ));
                    qb.push_where().field_eq("user_id", param.user_id);
                    qb.push_and().field_eq("role_name", role_name.clone());
                    qb.push_and().field_eq("app_id", param.app_id.unwrap_or_default());
                    qb.push_and().field_eq("status", RbacRoleStatus::Enable as i8).push(" limit 1");
                }
                qb.push(")) as t");
                let res = qb.build_query_as::<RbacRoleModel>().fetch_one(&self.db)
                    .await;
                (RbacRoleUserRange::Session as i8, role_key, role_name, res)
            }
            RbacRoleUserRangeData::Custom { role_name } => {
                let role_name = role_name.to_owned();
                let mut qb = QueryBuilder::<MySql>::new(format!(
                    "select * from ((select * from {}",
                    RbacRoleModel::table_name(),
                ));
                qb.push_where().field_eq("user_id", param.user_id);
                qb.push_and().field_eq("role_name", role_name.clone());
                qb.push_and().field_eq("app_id", param.app_id.unwrap_or_default());
                qb.push_and().field_eq("status", RbacRoleStatus::Enable as i8).push(" limit 1");
                qb.push(")) as t");
                let res = qb.build_query_as::<RbacRoleModel>().fetch_one(&self.db)
                    .await;
                (
                    RbacRoleUserRange::Custom as i8,
                    "".to_string(),
                    role_name,
                    res,
                )
            }
        };
        let res_range = param.res_range as i8;
        match res {
            Ok(rm) => Err(RbacError::System(fluent_message!("rbac-role-exist",{
                "name":rm.role_name,
                "key":rm.role_key
            }))),
            Err(sqlx::Error::RowNotFound) => {
                let app_id = param.app_id.unwrap_or_default();
                let time = now_time().unwrap_or_default();
                let res = Insert::<_, RbacRoleModel>::new()
                    .set(RbacRoleModel::ROLE_KEY, &role_key)
                    .set(RbacRoleModel::USER_RANGE, user_range)
                    .set(RbacRoleModel::RES_RANGE, res_range)
                    .set(RbacRoleModel::ROLE_NAME, &role_name)
                    .set(RbacRoleModel::USER_ID, param.user_id)
                    .set(RbacRoleModel::APP_ID, app_id)
                    .set(RbacRoleModel::CHANGE_TIME, time)
                    .set(RbacRoleModel::CHANGE_USER_ID, add_user_id)
                    .set(RbacRoleModel::STATUS, RbacRoleStatus::Enable as i8)
                    .execute(OptionTxExecutor::new(transaction.as_deref_mut(), &self.db))
                    .await?;
                let add_id = res.last_insert_id();
                Update::<_, RbacRoleModel>::new()
                    .set(RbacRoleModel::CHANGE_TIME, time)
                    .set(RbacRoleModel::CHANGE_USER_ID, add_user_id)
                    .set(RbacRoleModel::STATUS, RbacRoleStatus::Enable as i8)
                    .execute(
                        OptionTxExecutor::new(transaction, &self.db),
                        |qb| {
                            qb.push_where().field_eq("user_id", param.user_id);
                            qb.push_and().field_eq("role_key", role_key.to_owned());
                            qb.push_and().field_eq("app_id", app_id);
                            qb.push_and().field_eq("status", RbacRoleStatus::Enable as i8);
                            qb.push_and().field_ne("id", add_id);
                        },
                    )
                    .await?;
                let id = add_id;
                let role = self.find_by_id(&id).await?;
                self.cache().clear_access(&role, Some(&[]), Some(&[])).await;
                self.logger
                    .add(
                        &LogRole {
                            action: "add",
                            user_id: role.user_id,
                            app_id,
                            role_name: &role_name,
                            role_key: &role_key,
                            user_range,
                            res_range,
                        },
                        Some(role.id),
                        Some(add_user_id),
                        None,
                        env_data,
                    )
                    .await;
                Ok(role)
            }
            Err(e) => Err(e)?,
        }
    }

    /// 编辑角色
    pub async fn edit_role(
        &self,
        role: &RbacRoleModel,
        role_info: &RbacRoleUserRangeData<'_>,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<u64> {
        self.role_param_valid(role_info).await?;
        let time = now_time().unwrap_or_default();
        let (opt_name, opt_key, res) = match role_info {
            RbacRoleUserRangeData::Session {
                role_key,
                role_name,
            } => {
                let mut qb = QueryBuilder::<MySql>::new(format!(
                    "select * from ((select * from {}",
                    RbacRoleModel::table_name(),
                ));
                qb.push_where().field_eq("user_id", role.user_id);
                qb.push_and().field_eq("role_key", role_key.to_owned());
                qb.push_and().field_eq("app_id", role.app_id);
                qb.push_and().field_eq("status", RbacRoleStatus::Enable as i8).push_and().field_ne("id", role.id).push(" limit 1");
                if let Some(rname) = role_name
                    && !rname.is_empty() {
                        qb.push(format!(
                            ") union all  (select * from {}",
                            RbacRoleModel::table_name(),
                        ));
                        qb.push_where().field_eq("user_id", role.user_id);
                        qb.push_and().field_eq("role_name", rname.to_owned());
                        qb.push_and().field_eq("app_id", role.app_id);
                        qb.push_and().field_eq("status", RbacRoleStatus::Enable as i8).push_and().field_ne("id", role.id).push(" limit 1");
                    }
                qb.push(")) as t");
                let res = qb.build_query_as::<RbacRoleModel>().fetch_one(&self.db)
                    .await;
                let role_name = role_name.map(|e| e.to_owned());
                let role_key = role_key.to_string();
                (role_name, Some(role_key), res)
            }
            RbacRoleUserRangeData::Custom { role_name } => {
                let mut qb = QueryBuilder::<MySql>::new(format!(
                    "select * from ((select * from {}",
                    RbacRoleModel::table_name(),
                ));
                qb.push_where().field_eq("user_id", role.user_id);
                qb.push_and().field_eq("role_name", role_name.to_owned());
                qb.push_and().field_eq("app_id", role.app_id);
                qb.push_and().field_eq("status", RbacRoleStatus::Enable as i8).push_and().field_ne("id", role.id).push(" limit 1");
                qb.push(")) as t");
                let res = qb.build_query_as::<RbacRoleModel>().fetch_one(&self.db)
                    .await;

                (Some(role_name.to_string()), None, res)
            }
        };
        let res = res;
        match res {
            Ok(rm) => {
                return Err(RbacError::System(fluent_message!("rbac-role-exist",{
                    "name":rm.role_name,
                    "key":rm.role_key
                })))
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(e) => return Err(e)?,
        }
        let mut update = Update::<_, RbacRoleModel>::new()
            .set(RbacRoleModel::CHANGE_USER_ID, change_user_id)
            .set(RbacRoleModel::CHANGE_TIME, time);
        if let Some(ref name) = opt_name {
            update = update.set(RbacRoleModel::ROLE_NAME, name as &str);
        }
        if let Some(ref key) = opt_key {
            update = update.set(RbacRoleModel::ROLE_KEY, key as &str);
        }
        let out = update
            .execute(
                OptionTxExecutor::new(transaction, &self.db),
                |qb| {
                    qb.push_where().field_eq("id", role.id);
                },
            )
            .await?;
        let fout = out.rows_affected();
        self.cache().clear_access(role, Some(&[]), Some(&[])).await;
        self.logger
            .add(
                &LogRole {
                    action: "edit",
                    role_name: opt_name.as_deref().unwrap_or(role.role_name.as_str()),
                    role_key: opt_key.as_deref().unwrap_or(role.role_key.as_str()),
                    user_range: role.user_range,
                    app_id: role.app_id,
                    res_range: role.res_range,
                    user_id: role.user_id,
                },
                Some(role.id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        Ok(fout)
    }
    // /// 删除角色
    pub async fn del_role(
        &self,
        role: &RbacRoleModel,
        delete_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<()> {
        let time = now_time().unwrap_or_default();

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::<_, RbacRoleModel>::new()
            .set(RbacRoleModel::CHANGE_USER_ID, delete_user_id)
            .set(RbacRoleModel::CHANGE_TIME, time)
            .set(RbacRoleModel::STATUS, RbacRoleStatus::Delete as i8)
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("id", role.id);
            })
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }

        let tmp = Update::<_, RbacPermModel>::new()
            .set(RbacPermModel::CHANGE_USER_ID, delete_user_id)
            .set(RbacPermModel::CHANGE_TIME, time)
            .set(RbacPermModel::STATUS, RbacPermStatus::Delete as i8)
            .execute(
                &mut *db,
                |qb| {
                    qb.push_where().field_eq("role_id", role.id);
                },
            )
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }

        let tmp = Update::<_, RbacRoleUserModel>::new()
            .set(RbacRoleUserModel::CHANGE_USER_ID, delete_user_id)
            .set(RbacRoleUserModel::CHANGE_TIME, time)
            .set(RbacRoleUserModel::STATUS, RbacRoleUserStatus::Delete as i8)
            .execute(
                &mut *db,
                |qb| {
                    qb.push_where().field_eq("role_id", role.id);
                },
            )
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }

        db.commit().await?;

        self.cache().clear_access(role, None, None).await;

        self.logger
            .add(
                &LogRole {
                    action: "del",
                    user_id: role.user_id,
                    role_name: role.role_name.as_str(),
                    role_key: role.role_key.as_str(),
                    app_id: role.app_id,
                    user_range: role.user_range,
                    res_range: role.res_range,
                },
                Some(role.id),
                Some(delete_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
}
