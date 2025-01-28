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
    fluent_message, now_time, RemoteNotify, RequestEnv,
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

impl RbacRole {
    //添加角色
    #[allow(clippy::too_many_arguments)]
    pub async fn add_role(
        &self,
        user_id: u64,
        role_key: &str,
        role_name: &str,
        user_range: RbacRoleUserRange,
        res_range: RbacRoleResRange,
        add_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<RbacRoleModel> {
        let role_key = check_length!(role_key, "key", 32);
        let role_name = check_length!(role_name, "name", 32);
        let user_range = user_range as i8;
        let res_range = res_range as i8;
        let mut sql=vec![sql_format!(
            "select * from {} where user_id={user_id} and role_name={role_name} and status={} limit 1",
            RbacRoleModel::table_name(),
            RbacRoleStatus::Enable
        )];
        if !role_key.is_empty() {
            sql.push(sql_format!(
                "select * from {} where user_id={user_id} and role_key={role_key} and status={} limit 1",
                RbacRoleModel::table_name(),
                RbacRoleStatus::Enable
            ));
        }
        let res = sqlx::query_as::<_, RbacRoleModel>(&sql.join(" union all  "))
            .fetch_one(&self.db)
            .await;
        match res {
            Ok(rm) => Err(RbacError::System(fluent_message!("rbac-role-exist",{
                "name":rm.role_name,
                "key":rm.role_key
            }))),
            Err(sqlx::Error::RowNotFound) => {
                let time = now_time().unwrap_or_default();
                let idata = model_option_set!(RbacRoleModelRef,{
                    role_key:role_key,
                    user_range:user_range,
                    res_range:res_range,
                    role_name:role_name,
                    user_id:user_id,
                    change_time:time,
                    change_user_id:add_user_id,
                    status:(RbacRoleStatus::Enable as i8),
                });
                let id = db_option_executor!(
                    db,
                    {
                        let res = Insert::<sqlx::MySql, RbacRoleModel, _>::new(idata)
                            .execute(db.as_executor())
                            .await?;
                        res.last_insert_id()
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
    #[allow(clippy::too_many_arguments)]
    pub async fn edit_role(
        &self,
        role: &RbacRoleModel,
        role_key: Option<&str>,
        role_name: Option<&str>,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<u64> {
        let time = now_time().unwrap_or_default();
        let mut change = lsys_core::model_option_set!(RbacRoleModelRef,{
            change_user_id:change_user_id,
            change_time:time,
        });
        let opt_name = if let Some(tval) = role_name {
            Some(check_length!(tval, "name", 32))
        } else {
            None
        };
        let opt_key = if let Some(tval) = role_key {
            Some(check_length!(tval, "key", 32))
        } else {
            None
        };
        change.role_name = opt_name.as_ref();
        change.role_key = opt_key.as_ref();
        let db = &self.db;
        let fout = db_option_executor!(
            db,
            {
                let out = Update::<sqlx::MySql, RbacRoleModel, _>::new(change)
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
        let tmp = Update::<sqlx::MySql, RbacRoleModel, _>::new(change)
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
        let tmp = Update::<sqlx::MySql, RbacPermModel, _>::new(change)
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
        let tmp = Update::<sqlx::MySql, RbacRoleUserModel, _>::new(change)
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
