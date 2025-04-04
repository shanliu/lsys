mod access;
mod cache;
mod data;
mod logger;
mod res_type;
use logger::LogRes;
//RBAC中资源相关实现
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{fluent_message, RemoteNotify};

use crate::model::RbacResModel;
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use std::vec;

use lsys_core::{now_time, RequestEnv};

use super::result::{RbacError, RbacResult};
use super::role::RbacRole;
use crate::model::{RbacResModelRef, RbacResStatus};
pub use access::ResInfo;
pub(crate) use cache::*;
pub use data::*;
use lsys_core::db::{Insert, ModelTableName, SqlQuote, Update, WhereOption};
use lsys_core::{db_option_executor, model_option_set, sql_format};
pub use res_type::*;
use sqlx::{Acquire, Transaction};
//资源的操作相关实现

pub struct RbacRes {
    db: Pool<MySql>,
    pub(crate) cache_res_data: Arc<LocalCache<ResCacheKey, Option<RbacResModel>>>, // res_key:res edit,res_op all
    role: Arc<RbacRole>,
    logger: Arc<ChangeLoggerDao>,
}

//资源管理
impl RbacRes {
    pub fn new(
        db: Pool<MySql>,
        role: Arc<RbacRole>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        Self {
            cache_res_data: Arc::from(LocalCache::new(remote_notify.clone(), config)),
            db,
            role,
            logger,
        }
    }
}
pub struct RbacResData<'t> {
    pub res_name: Option<&'t str>,
    pub res_type: &'t str,
    pub res_data: &'t str,
}

pub struct RbacResAddData<'t> {
    pub user_id: u64,
    pub app_id: Option<u64>,
    pub res_info: RbacResData<'t>,
}

impl RbacRes {
    pub async fn add_res(
        &self,
        param: &RbacResAddData<'_>,
        add_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<u64> {
        let res_type = param.res_info.res_type;
        let res_type = check_length!(res_type, "type", 32);
        let res_data = param.res_info.res_data;
        let res_data = check_length!(res_data, "data", 32);
        let res_name = match param.res_info.res_name {
            Some(res_name) => check_length!(res_name, "name", 32),
            None => "".to_string(),
        };

        let res = sqlx::query_as::<_,RbacResModel>(&sql_format!(
            "select * from {} where user_id={} and res_type={res_type} and res_data={res_data} and app_id={} and status={}",
            RbacResModel::table_name(),
            param.user_id,
            param.app_id.unwrap_or_default(),
            RbacResStatus::Enable
        )).fetch_one(&self.db).await;
        match res {
            Ok(rm) => Err(RbacError::System(
                fluent_message!("rbac-res-exits",{
                    "res_type":res_type,
                    "res_data":res_data,
                    "name":rm.res_name
                }), //"res [{$key}] already exists,name is:{$name}",
            )),
            Err(sqlx::Error::RowNotFound) => {
                let app_id = param.app_id.unwrap_or_default();
                let time = now_time().unwrap_or_default();
                let idata = model_option_set!(RbacResModelRef,{
                    res_name:res_name,
                    res_type:res_type,
                    res_data:res_data,
                    user_id:param.user_id,
                    app_id:app_id,
                    change_time:time,
                    change_user_id:add_user_id,
                    status:(RbacResStatus::Enable as i8),
                });
                let other_change = model_option_set!(RbacResModelRef,{
                    change_time:time,
                    change_user_id:add_user_id,
                    status:(RbacResStatus::Enable as i8),
                });
                let id = db_option_executor!(
                    db,
                    {
                        let res = Insert::<RbacResModel, _>::new(idata)
                            .execute(db.as_executor())
                            .await?;
                        let add_id = res.last_insert_id();
                        Update::< RbacResModel, _>::new(other_change)
                            .execute_by_where(&WhereOption::Where(sql_format!(
                                "user_id={} and res_type={res_type} and res_data={res_data} and app_id={app_id} and status={} and id!={add_id}",
                                param.user_id,
                                RbacResStatus::Enable as i8,
                            )), db.as_executor())
                            .await?;
                        add_id
                    },
                    transaction,
                    &self.db
                );
                self.cache_res_data
                    .clear(&ResCacheKey {
                        res_type: res_type.clone(),
                        res_data: res_data.clone(),
                        user_id: param.user_id,
                        app_id,
                    })
                    .await;

                self.logger
                    .add(
                        &LogRes {
                            action: "add",
                            user_id: param.user_id,
                            app_id,
                            res_name: &res_name,
                            res_type: &res_type,
                            res_data: &res_data,
                        },
                        Some(id),
                        Some(add_user_id),
                        None,
                        env_data,
                    )
                    .await;

                Ok(id)
            }
            Err(e) => Err(e)?,
        }
    }
    /// 编辑资源
    pub async fn edit_res(
        &self,
        res: &RbacResModel,
        res_info: &RbacResData<'_>,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<u64> {
        let time = now_time().unwrap_or_default();
        let mut change = lsys_core::model_option_set!(RbacResModelRef,{
            change_user_id:change_user_id,
            change_time:time,
        });

        let opt_name = if let Some(res_name) = res_info.res_name {
            Some(check_length!(res_name, "name", 32))
        } else {
            Some("".to_string())
        };
        let res_type = res_info.res_type;
        let opt_type = Some(check_length!(res_type, "type", 32));
        let res_data = res_info.res_data;
        let opt_data = Some(check_length!(res_data, "data", 32));
        change.res_name = opt_name.as_ref();
        change.res_data = opt_data.as_ref();
        change.res_type = opt_type.as_ref();
        let db = &self.db;
        let fout = db_option_executor!(
            db,
            {
                let out = Update::< RbacResModel, _>::new(change)
                    .execute_by_pk(res, db.as_executor())
                    .await?;
                Ok(out.rows_affected())
            },
            transaction,
            db
        );
        self.cache_res_data
            .clear(&ResCacheKey {
                res_type: opt_type.to_owned().unwrap_or(res.res_type.clone()),
                res_data: opt_data.to_owned().unwrap_or(res.res_data.clone()),
                user_id: res.user_id,
                app_id: res.app_id,
            })
            .await;
        self.cache_res_data
            .clear(&ResCacheKey {
                res_type: res.res_type.clone(),
                res_data: res.res_data.clone(),
                user_id: res.user_id,
                app_id: res.app_id,
            })
            .await;

        self.logger
            .add(
                &LogRes {
                    action: "edit",
                    user_id: res.user_id,
                    app_id: res.app_id,
                    res_data: opt_data.as_deref().unwrap_or(res.res_data.as_str()),
                    res_name: opt_name.as_deref().unwrap_or(res.res_name.as_str()),
                    res_type: opt_type.as_deref().unwrap_or(res.res_type.as_str()),
                },
                Some(res.id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        fout
    }
    // /// 删除资源
    pub async fn del_res(
        &self,
        res: &RbacResModel,
        delete_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<()> {
        let time = now_time().unwrap_or_default();
        let change = lsys_core::model_option_set!(RbacResModelRef,{
            change_user_id:delete_user_id,
            change_time:time,
            status:(RbacResStatus::Delete as i8)
        });

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::< RbacResModel, _>::new(change)
            .execute_by_pk(res, &mut *db)
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }
        let tmp = self
            .role
            .role_remove_all_perm(res, delete_user_id, Some(&mut db), env_data)
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }
        db.commit().await?;
        self.cache_res_data
            .clear(&ResCacheKey {
                user_id: res.user_id,
                res_type: res.res_type.to_owned(),
                res_data: res.res_data.to_owned(),
                app_id: res.app_id,
            })
            .await;

        self.logger
            .add(
                &LogRes {
                    action: "del",
                    user_id: res.user_id,
                    app_id: res.app_id,
                    res_type: res.res_type.as_str(),
                    res_data: res.res_data.as_str(),
                    res_name: res.res_name.as_str(),
                },
                Some(res.id),
                Some(delete_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
}
