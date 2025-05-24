mod access;
mod cache;
mod data;
mod logger;
mod perm;
mod user;
//RBAC中角色相关实现
use logger::LogRole;
use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    db::WhereOption,
    fluent_message, now_time, valid_key, RemoteNotify, RequestEnv, ValidParam, ValidParamCheck,
    ValidPattern, ValidStrlen,
};
use sqlx::Acquire;
use std::{sync::Arc, vec};

use lsys_core::db::{Insert, ModelTableName, SqlQuote, Update};
use lsys_core::{db_option_executor, model_option_set, sql_format};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool, Transaction};

use crate::model::{
    RbacPermModel, RbacPermModelRef, RbacPermStatus, RbacRoleModel, RbacRoleModelRef,
    RbacRoleResRange, RbacRoleStatus, RbacRoleUserModel, RbacRoleUserModelRef, RbacRoleUserRange,
    RbacRoleUserStatus,
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
                        .add_rule(ValidStrlen::range(1, 32)),
                );
                if let Some(name) = role_name {
                    param_valid.add(
                        valid_key!("role_name"),
                        name,
                        &ValidParamCheck::default()
                            .add_rule(ValidPattern::NotFormat)
                            .add_rule(ValidStrlen::range(0, 32)),
                    );
                }
            }
            RbacRoleUserRangeData::Custom { role_name } => {
                param_valid.add(
                    valid_key!("role_name"),
                    role_name,
                    &ValidParamCheck::default()
                        .add_rule(ValidPattern::NotFormat)
                        .add_rule(ValidStrlen::range(1, 32)),
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
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<RbacRoleModel> {
        self.role_param_valid(&param.role_info).await?;
        let (user_range, role_key, role_name, sql) = match param.role_info {
            RbacRoleUserRangeData::Session {
                role_key,
                role_name,
            } => {
                let role_name = role_name.map(|e| e.to_owned()).unwrap_or_default();
                let role_key = role_key.to_owned();

                let mut sql = vec![
                    sql_format!(
                        "select * from {} where user_id={} and role_key={} and app_id={} and status={} limit 1",
                        RbacRoleModel::table_name(),
                        param.user_id,
                        role_key,
                        param.app_id.unwrap_or_default(),
                        RbacRoleStatus::Enable,
                     )
                 ];
                if !role_name.is_empty() {
                    sql.push(sql_format!(
                        "select * from {} where user_id={} and role_name={} and app_id={} and status={} limit 1",
                        RbacRoleModel::table_name(),
                        param.user_id,
                        role_name,
                        param.app_id.unwrap_or_default(),
                        RbacRoleStatus::Enable
                    ));
                }
                (RbacRoleUserRange::Session as i8, role_key, role_name, sql)
            }
            RbacRoleUserRangeData::Custom { role_name } => {
                let role_name = role_name.to_owned();
                let sql=vec![
                    sql_format!(
                         "select * from {} where user_id={} and role_name={} and app_id={} and status={} limit 1",
                         RbacRoleModel::table_name(),
                         param.user_id,
                         role_name,
                         param.app_id.unwrap_or_default(),
                         RbacRoleStatus::Enable
                     )
                 ];
                (
                    RbacRoleUserRange::Custom as i8,
                    "".to_string(),
                    role_name,
                    sql,
                )
            }
        };
        let res_range = param.res_range as i8;
        let res = sqlx::query_as::<_, RbacRoleModel>(&sql.join(" union all  "))
            .fetch_one(&self.db)
            .await;
        match res {
            Ok(rm) => Err(RbacError::System(fluent_message!("rbac-role-exist",{
                "name":rm.role_name,
                "key":rm.role_key
            }))),
            Err(sqlx::Error::RowNotFound) => {
                let app_id = param.app_id.unwrap_or_default();
                let time = now_time().unwrap_or_default();
                let idata = model_option_set!(RbacRoleModelRef,{
                    role_key:role_key,
                    user_range:user_range,
                    res_range:res_range,
                    role_name:role_name,
                    user_id:param.user_id,
                    app_id:app_id,
                    change_time:time,
                    change_user_id:add_user_id,
                    status:(RbacRoleStatus::Enable as i8),
                });
                let other_change = model_option_set!(RbacRoleModelRef,{
                    change_time:time,
                    change_user_id:add_user_id,
                    status:(RbacRoleStatus::Enable as i8),
                });
                let id = db_option_executor!(
                    db,
                    {
                        let res = Insert::<RbacRoleModel, _>::new(idata)
                            .execute(db.as_executor())
                            .await?;
                        let add_id = res.last_insert_id();
                        Update::<RbacRoleModel, _>::new(other_change)
                            .execute_by_where(
                                &WhereOption::Where(sql_format!(
                                "user_id={} and role_key={} and app_id={} and status={} and id!={}",
                                param.user_id,
                                role_key,
                                app_id,
                                RbacRoleStatus::Enable  as i8,
                                add_id,
                            )),
                                db.as_executor(),
                            )
                            .await?;
                        add_id
                    },
                    transaction,
                    &self.db
                );
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
        let mut change = lsys_core::model_option_set!(RbacRoleModelRef,{
            change_user_id:change_user_id,
            change_time:time,
        });
        let (opt_name, opt_key, sql) = match role_info {
            RbacRoleUserRangeData::Session {
                role_key,
                role_name,
            } => {
                let mut sql = vec![
                    sql_format!(
                        "select * from {} where user_id={} and role_key={} and app_id={} and status={} and id!={} limit 1",
                        RbacRoleModel::table_name(),
                        role.user_id,
                        role_key,
                        role.app_id,
                        RbacRoleStatus::Enable,
                        role.id
                     )
                 ];
                if let Some(rname) = role_name {
                    if !rname.is_empty() {
                        sql.push(sql_format!(
                            "select * from {} where user_id={} and role_name={} and app_id={} and status={} and id!={}  limit 1",
                            RbacRoleModel::table_name(),
                            role.user_id,
                            rname,
                            role.app_id,
                            RbacRoleStatus::Enable,
                            role.id
                        ));
                    }
                }
                let role_name = role_name.map(|e| e.to_owned());
                let role_key = role_key.to_string();
                (role_name, Some(role_key), sql)
            }
            RbacRoleUserRangeData::Custom { role_name } => {
                let sql=vec![
                    sql_format!(
                         "select * from {} where user_id={} and role_name={} and app_id={} and status={}  and id!={}  limit 1",
                         RbacRoleModel::table_name(),
                       role.user_id,
                         role_name,
                         role.app_id,
                         RbacRoleStatus::Enable,
                          role.id
                     )
                 ];

                (Some(role_name.to_string()), None, sql)
            }
        };
        let res = sqlx::query_as::<_, RbacRoleModel>(&sql.join(" union all  "))
            .fetch_one(&self.db)
            .await;
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
        change.role_name = opt_name.as_ref();
        change.role_key = opt_key.as_ref();
        let db = &self.db;
        let fout = db_option_executor!(
            db,
            {
                let out = Update::<RbacRoleModel, _>::new(change)
                    .execute_by_pk(role, db.as_executor())
                    .await?;
                out.rows_affected()
            },
            transaction,
            db
        );
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
        let change = lsys_core::model_option_set!(RbacRoleModelRef,{
            change_user_id:delete_user_id,
            change_time:time,
            status:(RbacRoleStatus::Delete as i8)
        });
        let tmp = Update::<RbacRoleModel, _>::new(change)
            .execute_by_pk(role, &mut *db)
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }

        let change = lsys_core::model_option_set!(RbacPermModelRef,{
            change_user_id:delete_user_id,
            change_time:time,
            status:(RbacPermStatus::Delete as i8)
        });
        let tmp = Update::<RbacPermModel, _>::new(change)
            .execute_by_where(
                &lsys_core::db::WhereOption::Where(sql_format!("role_id={}", role.id)),
                &mut *db,
            )
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }

        let change = lsys_core::model_option_set!(RbacRoleUserModelRef,{
            change_user_id:delete_user_id,
            change_time:time,
            status:(RbacRoleUserStatus::Delete as i8)
        });
        let tmp = Update::<RbacRoleUserModel, _>::new(change)
            .execute_by_where(
                &lsys_core::db::WhereOption::Where(sql_format!("role_id={}", role.id)),
                &mut *db,
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
