use std::sync::Arc;

use crate::dao::AccountResult;

use crate::model::{AccountModel, AccountModelRef, AccountPasswordModel, AccountPasswordModelRef};
use lsys_core::{fluent_message, now_time, IntoFluentMessage, RequestEnv};

use lsys_core::db::{Insert, Update};
use lsys_core::db::{ModelTableName, SqlQuote};
use lsys_core::{model_option_set, sql_format};
use lsys_logger::dao::ChangeLoggerDao;
use lsys_setting::dao::{NotFoundResult, SingleSetting};
use sqlx::{Acquire, MySql, Pool, Transaction};
use tracing::warn;

use super::logger::LogAccountPassWrod;
use super::{AccountError, AccountPasswordConfig, AccountPasswordHash};

pub struct AccountPassword {
    db: Pool<MySql>,
    // fluent: Arc<FluentBuild>,
    redis: deadpool_redis::Pool,
    account_passwrd_hash: Arc<AccountPasswordHash>,
    setting: Arc<SingleSetting>,
    logger: Arc<ChangeLoggerDao>,
}

impl AccountPassword {
    pub fn new(
        db: Pool<MySql>,
        setting: Arc<SingleSetting>,
        //fluent: Arc<FluentBuild>,
        redis: deadpool_redis::Pool,
        logger: Arc<ChangeLoggerDao>,
        account_passwrd_hash: Arc<AccountPasswordHash>,
    ) -> Self {
        Self {
            db,
            // fluent,
            redis,
            account_passwrd_hash,
            setting,
            logger,
        }
    }
    impl_account_valid_code_method!("passwrod",{
        account_id: &u64,
        from_type: &str,
    },{
        format!("{}-{}",account_id,from_type)
    },5*60);
    /// 校验验证码并设置新密码
    #[allow(clippy::too_many_arguments)]
    pub async fn set_passwrod_from_code(
        &self,
        account: &AccountModel,
        new_password: &str,
        from_type: &str,
        code: &str,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        self.valid_code_check(code, &account.id, from_type).await?;
        let res = self
            .set_passwrod(account, new_password, op_user_id, transaction, env_data)
            .await;
        if res.is_ok() {
            if let Err(err) = self.valid_code_clear(&account.id, from_type).await {
                warn!(
                    "email {} valid clear fail:{}",
                    &account.id,
                    err.to_fluent_message().default_format()
                );
            }
        }
        res
    }
    /// 设置新密码
    pub async fn set_passwrod(
        &self,
        account: &AccountModel,
        new_password: &str,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        let new_password = new_password.trim().to_string();
        if new_password.len() < 6 || new_password.len() > 32 {
            return Err(AccountError::System(
                fluent_message!("account-passwrod-wrong",
                    {
                        "len":new_password.len(),
                        "min":6,
                        "max":32
                    }
                ),
            )); //"password length need 6-32 char"
        }

        let db = &self.db;
        let time = now_time()?;
        let mut ta;
        if account.password_id > 0 {
            let account_pass_res = sqlx::query_as::<_, AccountPasswordModel>(&sql_format!(
                "select * from {} where account_id={} and id={}",
                AccountPasswordModel::table_name(),
                account.id,
                account.password_id,
            ))
            .fetch_one(db)
            .await;

            match account_pass_res {
                Err(sqlx::Error::RowNotFound) => {
                    ta = match transaction {
                        Some(pb) => pb.begin().await?,
                        None => db.begin().await?,
                    };
                }
                Ok(account_pass) => {
                    ta = match transaction {
                        Some(pb) => pb.begin().await?,
                        None => db.begin().await?,
                    };
                    let change = lsys_core::model_option_set!(AccountPasswordModelRef, { disable_time: time });
                    //ta.execute(query)
                    Update::<sqlx::MySql, AccountPasswordModel, _>::new(change)
                        .execute_by_pk(&account_pass, &mut *ta)
                        .await?;
                }
                Err(err) => {
                    return Err(err.into());
                }
            }
        } else {
            ta = match transaction {
                Some(pb) => pb.begin().await?,
                None => db.begin().await?,
            };
        }
        let nh_passwrod = self.account_passwrd_hash.hash_password(&new_password).await;

        let config = self
            .setting
            .load::<AccountPasswordConfig>(None)
            .await
            .notfound_default()?;

        if config.disable_old_password {
            let old_pass_res = sqlx::query_as::<_, AccountPasswordModel>(&sql_format!(
                "select * from {} where account_id={} and password={}",
                AccountPasswordModel::table_name(),
                account.id,
                nh_passwrod
            ))
            .fetch_one(db)
            .await;

            if old_pass_res.is_ok() {
                ta.rollback().await?;

                return Err(AccountError::System(fluent_message!(
                    "account-old-passwrod"
                ))); //                    "can't old password"
            }
        }

        let new_data = model_option_set!(AccountPasswordModelRef,{
            account_id:account.id,
            password:nh_passwrod,
            disable_time: 0,
            add_time: time,
        });
        let res = Insert::<sqlx::MySql, AccountPasswordModel, _>::new(new_data)
            .execute(&mut *ta)
            .await;
        match res {
            Err(e) => {
                ta.rollback().await?;
                Err(e.into())
            }
            Ok(data) => {
                let pid = data.last_insert_id();
                let change = lsys_core::model_option_set!(AccountModelRef,{
                    password_id:pid,
                    change_time:time,
                });
                let u_res = Update::<sqlx::MySql, AccountModel, _>::new(change)
                    .execute_by_pk(account, &mut *ta)
                    .await;
                match u_res {
                    Err(e) => {
                        ta.rollback().await?;
                        Err(e.into())
                    }
                    Ok(_) => {
                        ta.commit().await?;

                        self.logger
                            .add(
                                &LogAccountPassWrod {
                                    account_id: account.id,
                                },
                                Some(pid),
                                Some(op_user_id),
                                None,
                                env_data,
                            )
                            .await;

                        Ok(pid)
                    }
                }
            }
        }
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        AccountPasswordModel,
        AccountResult<AccountPasswordModel>,
        id,
        "id = {id} "
    );
    /// 检测密码是否正确
    pub async fn check_password(
        &self,
        account: &AccountModel,
        check_password: &str,
    ) -> AccountResult<bool> {
        let account_password = match self.find_by_id(&account.password_id).await {
            Ok(up) => up,
            Err(err) => match err {
                AccountError::Sqlx(sqlx::Error::RowNotFound) => {
                    return Err(AccountError::System(fluent_message!(
                        "account-passwrod-delete"
                    ))); //"can't password,may be is delete"
                }
                _ => return Err(err),
            },
        };
        Ok(self
            .account_passwrd_hash
            .hash_password(check_password)
            .await
            == account_password.password)
    }
    /// 检测指定ID密码是否超时
    pub async fn password_timeout(&self, account_id: &u64) -> AccountResult<bool> {
        if let Ok(set) = self
            .setting
            .load::<AccountPasswordConfig>(None)
            .await
            .notfound_default()
        {
            if set.timeout == 0 {
                return Ok(false);
            }
            let sql = sql_format!(
                "select p.add_time from {} as p join {} as u
                on p.id=u.password_id 
                where u.id={}",
                AccountPasswordModel::table_name(),
                AccountModel::table_name(),
                account_id
            );
            let add_time = sqlx::query_scalar::<_, u64>(&sql)
                .fetch_one(&self.db)
                .await?;
            if add_time + set.timeout < now_time().unwrap_or_default() {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
