use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    fluent_message, now_time, string_clear, valid_key, RemoteNotify, RequestEnv, StringClear,
    ValidError, ValidParam, ValidParamCheck, ValidPattern, ValidStrMatch, ValidStrlen,
    STRING_CLEAR_FORMAT,
};

use lsys_logger::dao::ChangeLoggerDao;
// use rand::seq::SliceRandom;
use lsys_core::db::{Insert, ModelTableName, SqlQuote, Update, WhereOption};
use lsys_core::{model_option_set, sql_format};
use sqlx::{Acquire, MySql, Pool, Transaction};
use std::{collections::HashMap, sync::Arc};
use tracing::warn;

use crate::model::{AccountModel, AccountNameModel, AccountNameModelRef, AccountNameStatus};

use super::{logger::LogAccountName, AccountError, AccountIndex, AccountResult};
use lsys_access::dao::AccessDao;
use lsys_core::IntoFluentMessage;
pub struct AccountName {
    db: Pool<MySql>,
    // fluent: Arc<FluentBuild>,
    index: Arc<AccountIndex>,
    access: Arc<AccessDao>,
    pub(crate) cache: Arc<LocalCache<u64, AccountNameModel>>,
    logger: Arc<ChangeLoggerDao>,
}

impl AccountName {
    pub fn new(
        db: Pool<MySql>,
        index: Arc<AccountIndex>,
        access: Arc<AccessDao>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        Self {
            cache: Arc::new(LocalCache::new(remote_notify, config)),
            db,
            access,
            index,
            logger,
        }
    }
    /// 根据用户名查找记录
    pub async fn find_by_name(&self, name: &str) -> AccountResult<AccountNameModel> {
        let name = string_clear(name, StringClear::Option(STRING_CLEAR_FORMAT), Some(101));
        if name.is_empty() {
            return Err(sqlx::Error::RowNotFound.into());
        }
        let res = sqlx::query_as::<_, AccountNameModel>(&sql_format!(
            "select * from {} where username={} and status={}",
            AccountNameModel::table_name(),
            name,
            AccountNameStatus::Enable
        ))
        .fetch_one(&self.db)
        .await?;

        Ok(res)
    }
    /// 移除用户登录
    pub async fn remove_account_name(
        &self,
        account: &AccountModel,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<()> {
        //change name is del_**
        let ntime = now_time().unwrap_or_default();
        let username = "del_".to_string() + "_" + account.id.to_string().as_str();
        let status = AccountNameStatus::Delete as i8;
        let name_change = lsys_core::model_option_set!(AccountNameModelRef,{
            username:username,
            change_time:ntime,
            status:status
        });

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<AccountNameModel, _>::new(name_change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("account_id={}", account.id)),
                &mut *db,
            )
            .await;
        if let Err(e) = res {
            db.rollback().await?;
            return Err(e.into());
        }
        if let Err(ie) = self
            .index
            .cat_del(
                crate::model::AccountIndexCat::NikeName,
                account.id,
                Some(&mut db),
            )
            .await
        {
            db.rollback().await?;
            return Err(ie);
        }
        db.commit().await?;

        self.logger
            .add(
                &LogAccountName {
                    action: "del",
                    username: &username,
                },
                Some(account.id),
                Some(op_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    async fn name_param_valid(&self, username: &str) -> AccountResult<()> {
        ValidParam::default()
            .add(
                valid_key!("username"),
                &username,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(3, 32))
                    .add_rule(ValidStrMatch::StartNotWith("del_")),
            )
            .check()?;

        Ok(())
    }
    /// 更改用户名
    pub async fn change_account_name(
        &self,
        account: &AccountModel,
        username: &str,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<()> {
        self.name_param_valid(username).await?;
        let username = username.to_string();
        let time = now_time()?;
        let db = &self.db;

        let account_name_res = sqlx::query_as::<_, AccountNameModel>(&sql_format!(
            "select * from {} where username={}",
            AccountNameModel::table_name(),
            username,
        ))
        .fetch_one(&self.db)
        .await;

        let out = match account_name_res {
            Err(sqlx::Error::RowNotFound) => {
                let account_name_res = sqlx::query_as::<_, AccountNameModel>(&sql_format!(
                    "select * from {} where account_id={}",
                    AccountNameModel::table_name(),
                    account.id,
                ))
                .fetch_one(db)
                .await;

                match account_name_res {
                    Err(sqlx::Error::RowNotFound) => {
                        let status = AccountNameStatus::Enable as i8;
                        let new_data = model_option_set!(AccountNameModelRef,{
                            account_id:account.id,
                            username:username,
                            status:status,
                            change_time: time,
                        });
                        let mut db = match transaction {
                            Some(pb) => pb.begin().await?,
                            None => self.db.begin().await?,
                        };
                        let tmp = Insert::<AccountNameModel, _>::new(new_data)
                            .execute(&mut *db)
                            .await;
                        if let Err(ie) = tmp {
                            db.rollback().await?;
                            return Err(ie.into());
                        }
                        let tmp = sqlx::query(
                            sql_format!(
                                "UPDATE {} SET use_name=1 WHERE id=?",
                                AccountModel::table_name(),
                            )
                            .as_str(),
                        )
                        .bind(account.id)
                        .execute(&mut *db)
                        .await;
                        if let Err(ie) = tmp {
                            db.rollback().await?;
                            return Err(ie.into());
                        }
                        if let Err(ie) = self
                            .index
                            .cat_one_add(
                                crate::model::AccountIndexCat::NikeName,
                                account.id,
                                &username,
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }
                        db.commit().await?;
                        self.cache.clear(&account.id).await;
                        Ok(())
                    }
                    Ok(account_name) => {
                        let status = AccountNameStatus::Enable as i8;
                        let change = lsys_core::model_option_set!(AccountNameModelRef,{
                            status:status,
                            username:username,
                            change_time:time
                        });
                        let mut db = match transaction {
                            Some(pb) => pb.begin().await?,
                            None => self.db.begin().await?,
                        };
                        let tmp = Update::<AccountNameModel, _>::new(change)
                            .execute_by_pk(&account_name, &mut *db)
                            .await;
                        if let Err(ie) = tmp {
                            db.rollback().await?;
                            return Err(ie.into());
                        }
                        if let Err(ie) = self
                            .index
                            .cat_one_add(
                                crate::model::AccountIndexCat::NikeName,
                                account_name.account_id,
                                &username,
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }
                        db.commit().await?;
                        self.cache.clear(&account.id).await;
                        Ok(())
                    }
                    Err(err) => Err(err.into()),
                }
            }
            Ok(account_name) => {
                if account_name.account_id == account.id {
                    Ok(())
                } else {
                    Err(AccountError::Vaild(ValidError::message(
                        valid_key!("account_name"),
                        fluent_message!("account-name-exits",{

                            "name":&username
                        }),
                    )))
                }
            }
            Err(err) => Err(err.into()),
        };
        if out.is_ok() {
            //此过程必须,通过过去好查数据
            if let Err(err) = self
                .access
                .user
                .sync_user(0, account.id, None, Some(&username))
                .await
            {
                warn!(
                    "sync user account to access fail:{}",
                    err.to_fluent_message().default_format()
                );
            };

            self.logger
                .add(
                    &LogAccountName {
                        action: "set",
                        username: &username,
                    },
                    Some(account.id),
                    Some(op_user_id),
                    None,
                    env_data,
                )
                .await;
        }
        out
    }

    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_account_id,
        u64,
        AccountNameModel,
        AccountResult<AccountNameModel>,
        id,
        "account_id = {id}  order by id desc"
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_account_ids,
        u64,
        AccountNameModel,
        AccountResult<HashMap<u64, AccountNameModel>>,
        account_id,
        ids,
        "account_id in ({ids})  order by id desc"
    );
    pub fn cache(&'_ self) -> AccountNameCache<'_> {
        AccountNameCache { dao: self }
    }
}

pub struct AccountNameCache<'t> {
    pub dao: &'t AccountName,
}
impl AccountNameCache<'_> {
    lsys_core::impl_cache_fetch_one!(
        find_by_account_id,
        dao,
        cache,
        u64,
        AccountResult<AccountNameModel>
    );
    lsys_core::impl_cache_fetch_vec!(
        find_by_account_ids,
        dao,
        cache,
        u64,
        AccountResult<HashMap<u64, AccountNameModel>>
    );
}
