mod access;
mod cache;
mod data;
mod logger;
use logger::LogOp;
//RBAC中资源相关实现
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{
    fluent_message, valid_key, RemoteNotify, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen,
};

use crate::model::{RbacOpModel, RbacOpModelRef, RbacOpStatus, RbacResModel};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use std::vec;

use lsys_core::{now_time, RequestEnv};

use crate::model::RbacResStatus;
use lsys_core::db::{Insert, ModelTableName, SqlQuote, Update, WhereOption};
use lsys_core::{db_option_executor, model_option_set, sql_format};
use sqlx::{Acquire, Transaction};

use super::res::RbacRes;
use super::result::{RbacError, RbacResult};
pub use access::OpInfo;
pub(crate) use cache::*;
pub use data::*;

//资源操作的相关实现

pub struct RbacOp {
    db: Pool<MySql>,
    pub(crate) cache_op_data: Arc<LocalCache<OpCacheKey, Option<RbacOpModel>>>, // res_key:res edit,res_op all
    res: Arc<RbacRes>,
    logger: Arc<ChangeLoggerDao>,
}

//资源管理
impl RbacOp {
    pub fn new(
        db: Pool<MySql>,
        res: Arc<RbacRes>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        Self {
            cache_op_data: Arc::from(LocalCache::new(remote_notify.clone(), config)),
            db,
            res,
            logger,
        }
    }
}

pub struct RbacOpData<'t> {
    pub op_key: &'t str,
    pub op_name: Option<&'t str>,
}

pub struct RbacOpAddData<'t> {
    pub user_id: u64,
    pub app_id: Option<u64>,
    pub op_info: RbacOpData<'t>,
}

impl RbacOp {
    async fn op_param_valid(&self, param: &RbacOpData<'_>) -> RbacResult<()> {
        ValidParam::default()
            .add(
                valid_key!("op_key"),
                &param.op_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, 32)),
            )
            .add(
                valid_key!("op_name"),
                &param.op_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(0, 32)),
            )
            .check()?;
        Ok(())
    }
    /// 添加资源
    pub async fn add_op(
        &self,
        param: &RbacOpAddData<'_>,
        add_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<u64> {
        self.op_param_valid(&param.op_info).await?;
        let op_key = param.op_info.op_key.to_owned();
        let op_name = param
            .op_info
            .op_name
            .map(|e| e.to_owned())
            .unwrap_or_default();
        let res = sqlx::query_as::<_, RbacOpModel>(&sql_format!(
            "select * from {} where user_id={} and op_key={} and app_id={} and status={}",
            RbacResModel::table_name(),
            param.user_id,
            op_key,
            param.app_id.unwrap_or_default(),
            RbacResStatus::Enable,
        ))
        .fetch_one(&self.db)
        .await;
        match res {
            Ok(rm) => Err(RbacError::System(
                fluent_message!("rbac-op-exits",{
                    "res_type":op_key,
                    "res_data":op_name,
                    "name":rm.op_name
                }), //"res [{$key}] already exists,name is:{$name}",
            )),
            Err(sqlx::Error::RowNotFound) => {
                let app_id = param.app_id.unwrap_or_default();
                let time = now_time().unwrap_or_default();
                let idata = model_option_set!(RbacOpModelRef,{
                    op_key:op_key,
                    op_name:op_name,
                    user_id:param.user_id,
                    app_id:app_id,
                    change_time:time,
                    change_user_id:add_user_id,
                    status:(RbacOpStatus::Enable as i8),
                });
                let other_change = model_option_set!(RbacOpModelRef,{
                    change_time:time,
                    change_user_id:add_user_id,
                    status:(RbacOpStatus::Enable as i8),
                });
                let id = db_option_executor!(
                    db,
                    {
                        let res = Insert::<RbacOpModel, _>::new(idata)
                            .execute(db.as_executor())
                            .await?;
                        let add_id = res.last_insert_id();
                        Update::<RbacOpModel, _>::new(other_change)
                            .execute_by_where(
                                &WhereOption::Where(sql_format!(
                                "user_id={} and op_key={} and  app_id={} and status={} and id!={}",
                                param.user_id,
                                op_key,
                                app_id,
                                RbacOpStatus::Enable as i8,
                                add_id
                            )),
                                db.as_executor(),
                            )
                            .await?;
                        add_id
                    },
                    transaction,
                    &self.db
                );
                self.cache_op_data
                    .clear(&OpCacheKey {
                        op_key: op_key.to_owned(),
                        user_id: param.user_id,
                        app_id,
                    })
                    .await;

                self.logger
                    .add(
                        &LogOp {
                            action: "add",
                            app_id,
                            op_key: &op_key,
                            op_name: op_name.as_ref(),
                            user_id: param.user_id,
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
}

impl RbacOp {
    /// 编辑资源
    pub async fn edit_op(
        &self,
        op: &RbacOpModel,
        op_info: &RbacOpData<'_>,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<u64> {
        self.op_param_valid(op_info).await?;
        let time = now_time().unwrap_or_default();
        let mut change = lsys_core::model_option_set!(RbacOpModelRef,{
            change_user_id:change_user_id,
            change_time:time,
        });
        let opt_name = op_info.op_name.map(|e| e.to_owned());
        let op_key = op_info.op_key.to_string();
        let opt_key = Some(op_key);
        change.op_key = opt_key.as_ref();
        change.op_name = opt_name.as_ref();
        let db = &self.db;
        let fout = db_option_executor!(
            db,
            {
                let out = Update::<RbacOpModel, _>::new(change)
                    .execute_by_pk(op, db.as_executor())
                    .await?;
                Ok(out.rows_affected())
            },
            transaction,
            db
        );
        self.cache_op_data
            .clear(&OpCacheKey {
                op_key: opt_key.to_owned().unwrap_or(op.op_key.clone()),
                user_id: op.user_id,
                app_id: op.app_id,
            })
            .await;
        self.cache_op_data
            .clear(&OpCacheKey {
                op_key: op.op_key.to_owned(),
                user_id: op.user_id,
                app_id: op.app_id,
            })
            .await;

        self.logger
            .add(
                &LogOp {
                    action: "edit",
                    user_id: op.user_id,
                    app_id: op.app_id,
                    op_name: opt_name.as_deref().unwrap_or(op.op_name.as_str()),
                    op_key: opt_key.as_deref().unwrap_or(op.op_key.as_str()),
                },
                Some(op.id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        fout
    }
    // /// 删除资源
    pub async fn del_op(
        &self,
        op: &RbacOpModel,
        delete_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<()> {
        if RbacOpStatus::Delete.eq(op.status) {
            return Ok(());
        }
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let time = now_time().unwrap_or_default();
        let change = lsys_core::model_option_set!(RbacOpModelRef,{
            change_user_id:delete_user_id,
            change_time:time,
            status:(RbacOpStatus::Delete as i8)
        });

        let tmp = Update::<RbacOpModel, _>::new(change)
            .execute_by_pk(op, &mut *db)
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }
        let tmp = self
            .res
            .res_type_remove_op(op, delete_user_id, Some(&mut db), env_data)
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }
        db.commit().await?;
        self.cache_op_data
            .clear(&OpCacheKey {
                user_id: op.user_id,
                op_key: op.op_key.to_owned(),
                app_id: op.app_id,
            })
            .await;

        self.logger
            .add(
                &LogOp {
                    action: "del",
                    user_id: op.user_id,
                    app_id: op.app_id,
                    op_name: op.op_name.as_str(),
                    op_key: op.op_key.as_str(),
                },
                Some(op.id),
                Some(delete_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
}
