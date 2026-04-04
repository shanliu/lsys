mod access;
mod cache;
mod data;
pub(crate) mod logger;
use logger::LogOp;
//RBAC中资源相关实现
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::fluent_message;
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::valid_key;
use lsys_core::valid_param::{ValidParam, ValidParamCheck, ValidPattern, ValidStrlen};

use crate::model::{RbacOpModel, RbacOpStatus};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use std::vec;

use lsys_core::utils::{now_time, RequestEnv};

use lsys_core::db::{
    utils::FetchField, Insert, OptionTxExecutor, QueryBuilderExt, TableMeta, Update,
};
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
        let fetch_field = FetchField::new(&self.db);
        let op_key_max = fetch_field.string_max::<RbacOpModel>( &RbacOpModel::OP_KEY)
            .await
            .len_or(32);
        let op_name_max = fetch_field.string_max::<RbacOpModel>( &RbacOpModel::OP_NAME)
            .await
            .len_or(32);

        ValidParam::default()
            .add(
                valid_key!("op_key"),
                &param.op_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, op_key_max)),
            )
            .add(
                valid_key!("op_name"),
                &param.op_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(0, op_name_max)),
            )
            .check()?;
        Ok(())
    }
    /// 添加资源
    pub async fn add_op(
        &self,
        param: &RbacOpAddData<'_>,
        add_user_id: u64,
        mut transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<u64> {
        self.op_param_valid(&param.op_info).await?;
        let op_key = param.op_info.op_key.to_owned();
        let op_name = param
            .op_info
            .op_name
            .map(|e| e.to_owned())
            .unwrap_or_default();
        let find_res = sqlx::query_as::<_, RbacOpModel>(&format!(
            "select * from {} where user_id=? and op_key=? and app_id=? and status=?",
            RbacOpModel::table_name(),
        ))
        .bind(param.user_id)
        .bind(&op_key)
        .bind(param.app_id.unwrap_or_default())
        .bind(RbacOpStatus::Enable as i8)
        .fetch_one(&self.db)
        .await;
        match find_res {
            Ok(rm) => Err(RbacError::System(fluent_message!("rbac-op-exits",{
                "op_type":op_key,
                "old_name":rm.op_name
            }))),
            Err(sqlx::Error::RowNotFound) => {
                let app_id = param.app_id.unwrap_or_default();
                let time = now_time().unwrap_or_default();
                let res = Insert::<_, RbacOpModel>::new()
                    .set(RbacOpModel::OP_KEY, &op_key)
                    .set(RbacOpModel::OP_NAME, &op_name)
                    .set(RbacOpModel::USER_ID, param.user_id)
                    .set(RbacOpModel::APP_ID, app_id)
                    .set(RbacOpModel::CHANGE_TIME, time)
                    .set(RbacOpModel::CHANGE_USER_ID, add_user_id)
                    .set(RbacOpModel::STATUS, RbacOpStatus::Enable as i8)
                    .execute(OptionTxExecutor::new(transaction.as_deref_mut(), &self.db))
                    .await?;
                let add_id = res.last_insert_id();
                Update::<_, RbacOpModel>::new()
                    .set(RbacOpModel::CHANGE_TIME, time)
                    .set(RbacOpModel::CHANGE_USER_ID, add_user_id)
                    .set(RbacOpModel::STATUS, RbacOpStatus::Enable as i8)
                    .execute(
                        OptionTxExecutor::new(transaction, &self.db),
                        |qb| {
                            qb.push_where().field_eq("user_id", param.user_id);
                            qb.push_and().field_eq("op_key", op_key.to_owned());
                            qb.push_and().field_eq("app_id", app_id);
                            qb.push_and().field_eq("status", RbacOpStatus::Enable as i8);
                            qb.push_and().field_ne("id", add_id);
                        },
                    )
                    .await?;
                let id = add_id;
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

        let res = sqlx::query_as::<_, RbacOpModel>(&format!(
            "select * from {} where user_id=? and op_key=? and app_id=? and status=? and id!=?",
            RbacOpModel::table_name(),
        ))
        .bind(op.user_id)
        .bind(op_info.op_key)
        .bind(op.app_id)
        .bind(RbacOpStatus::Enable as i8)
        .bind(op.id)
        .fetch_one(&self.db)
        .await;
        match res {
            Ok(rm) => {
                return Err(RbacError::System(fluent_message!("rbac-op-exits",{
                    "op_type":op_info.op_key,
                    "old_name":rm.op_name
                })))
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(e) => return Err(e.into()),
        }
        let op_key = op_info.op_key.to_string();
        let opt_name = op_info.op_name.map(|e| e.to_owned());
        let time = now_time().unwrap_or_default();
        let opt_key = Some(op_key);
        let mut update = Update::<_, RbacOpModel>::new()
            .set(RbacOpModel::CHANGE_USER_ID, change_user_id)
            .set(RbacOpModel::CHANGE_TIME, time);
        if let Some(ref key) = opt_key {
            update = update.set(RbacOpModel::OP_KEY, key as &str);
        }
        if let Some(ref name) = opt_name {
            update = update.set(RbacOpModel::OP_NAME, name as &str);
        }
        let out = update
            .execute(
                OptionTxExecutor::new(transaction, &self.db),
                |qb| {
                    qb.push_where().field_eq("id", op.id);
                },
            )
            .await?;
        let fout = out.rows_affected();
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
        Ok(fout)
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

        let tmp = Update::<_, RbacOpModel>::new()
            .set(RbacOpModel::CHANGE_USER_ID, delete_user_id)
            .set(RbacOpModel::CHANGE_TIME, time)
            .set(RbacOpModel::STATUS, RbacOpStatus::Delete as i8)
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("id", op.id);
            })
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
