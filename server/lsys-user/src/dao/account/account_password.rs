use std::sync::Arc;

use crate::dao::AccountResult;

use crate::model::{AccountModel, AccountPasswordModel};
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::utils::{now_time, RequestEnv};
use lsys_core::valid_code::{CheckCodeData, ValidCode, ValidCodeData, ValidCodeDataRandom};
use lsys_core::valid_param::{ValidParam, ValidParamCheck, ValidPassword};
use lsys_core::{fluent_message, valid_key};

use lsys_core::db::{Insert, QueryBuilderExt, Update};
use lsys_core::db::TableMeta;
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
}
impl AccountPassword {
    /// 验证码生成
    pub fn valid_code(&self) -> ValidCode {
        ValidCode::new(self.redis.clone(), "passwrod", true, Some(6))
    }
    /// 获取验证码
    pub async fn valid_code_set<T: ValidCodeData>(
        &self,
        valid_code_data: &mut T,
        account_id: u64,
        from_type: &str,
    ) -> AccountResult<(String, usize)> {
        let out = self
            .valid_code()
            .set_code(&format!("{}-{}", account_id, from_type), valid_code_data)
            .await?;
        Ok(out)
    }
    /// 验证码构造器
    pub fn valid_code_builder(&self) -> ValidCodeDataRandom {
        ValidCodeDataRandom::new(300, 30)
    }
    /// 检测验证码
    pub async fn valid_code_check(
        &self,
        code: &str,
        account_id: u64,
        from_type: &str,
    ) -> AccountResult<()> {
        self.valid_code()
            .check_code(&CheckCodeData::new(
                &format!("{}-{}", account_id, from_type),
                code,
            ))
            .await?;
        Ok(())
    }
    pub async fn valid_code_clear(&self, account_id: u64, from_type: &str) -> AccountResult<()> {
        let mut builder = self.valid_code_builder();
        self.valid_code()
            .destroy_code(&format!("{}-{}", account_id, from_type), &mut builder)
            .await?;
        Ok(())
    }
}
impl AccountPassword {
    async fn passwrod_param_valid(&self, new_password: &str) -> AccountResult<()> {
        ValidParam::default()
            .add(
                valid_key!("user_password"),
                &new_password,
                &ValidParamCheck::default().add_rule(ValidPassword::Medium),
            )
            .check()?;

        Ok(())
    }
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
        self.passwrod_param_valid(new_password).await?;
        self.valid_code_check(code, account.id, from_type).await?;
        let res = self
            .set_passwrod(account, new_password, op_user_id, transaction, env_data)
            .await;
        if res.is_ok()
            && let Err(err) = self.valid_code_clear(account.id, from_type).await {
                warn!(
                    "email {} valid clear fail:{}",
                    account.id,
                    err.to_fluent_message().default_format()
                );
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
        self.passwrod_param_valid(new_password).await?;
        let new_password = new_password.to_string();
        let db = &self.db;
        let time = now_time()?;
        let mut ta;

        let config = self
            .setting
            .load::<AccountPasswordConfig>(None)
            .await
            .notfound_default()?;
        if account.password_id > 0 {
            let account_pass_res = sqlx::query_as::<_, AccountPasswordModel>(&format!(
                "select * from {} where account_id=? and id=?",
                AccountPasswordModel::table_name(),
            ))
            .bind(account.id)
            .bind(account.password_id)
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
                    Update::<_,AccountPasswordModel>::new()
                        .set(AccountPasswordModel::DISABLE_TIME, time)
                        .execute(&mut *ta, |qb| {
                            qb.push_where().field_eq("id", account_pass.id);
                        })
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
        if config.disable_old_password {
            let old_pass_res: Result<AccountPasswordModel, sqlx::Error> = sqlx::query_as::<_, AccountPasswordModel>(&format!(
                "select * from {} where account_id=? and password=?",
                AccountPasswordModel::table_name(),
            ))
            .bind(account.id)
            .bind(&nh_passwrod)
            .fetch_one(&self.db)
            .await;

            if old_pass_res.is_ok() {
                ta.rollback().await?;

                return Err(AccountError::System(fluent_message!(
                    "account-old-passwrod"
                ))); //                    "can't old password"
            }
        }

        let res = Insert::<_,AccountPasswordModel>::new()
            .set(AccountPasswordModel::ACCOUNT_ID, account.id)
            .set(AccountPasswordModel::PASSWORD, nh_passwrod)
            .set(AccountPasswordModel::DISABLE_TIME, 0_u64)
            .set(AccountPasswordModel::ADD_TIME, time)
            .execute(&mut *ta)
            .await;
        match res {
            Err(e) => {
                ta.rollback().await?;
                Err(e.into())
            }
            Ok(data) => {
                let pid = data.last_insert_id();
                let u_res = Update::<_,AccountModel>::new()
                    .set(AccountModel::PASSWORD_ID, pid)
                    .set(AccountModel::CHANGE_TIME, time)
                    .execute(&mut *ta, |qb| {
                        qb.push_where().field_eq("id", account.id);
                    })
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
    pub async fn find_by_id(&self, id: &u64) -> AccountResult<AccountPasswordModel> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountPasswordModel>::one(
            &self.db,
            |qb| { qb.field_eq("id", *id); },
        ).await?)
    }
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
    /// 返回 (是否超时, 密码有效期配置)
    pub async fn password_timeout(&self, account_id: u64) -> AccountResult<(bool, u64)> {
        if let Ok(set) = self
            .setting
            .load::<AccountPasswordConfig>(None)
            .await
            .notfound_default()
        {
            let timeout_value = set.timeout;
            if set.timeout == 0 {
                return Ok((false, timeout_value));
            }
            let sql = format!(
                "select p.add_time from {} as p join {} as u
                on p.id=u.password_id
                where u.id=?",
                AccountPasswordModel::table_name(),
                AccountModel::table_name(),
            );
            let add_time = sqlx::query_scalar::<_, u64>(&sql)
                .bind(account_id)
                .fetch_one(&self.db)
                .await?;
            if add_time + set.timeout < now_time().unwrap_or_default() {
                return Ok((true, timeout_value));
            }
            return Ok((false, timeout_value));
        }
        Ok((false, 0))
    }
}
