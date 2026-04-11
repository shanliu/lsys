use lsys_core::remote_notify::RemoteNotify;
use lsys_core::utils::{RequestEnv, STRING_CLEAR_FORMAT, StringClear, now_time, string_clear};
use lsys_core::valid_param::{
    ValidError, ValidParam, ValidParamCheck, ValidPattern, ValidStrMatch, ValidStrlen,
};
use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    db::utils::FetchField,
    fluent_message, valid_key,
};

use lsys_logger::dao::ChangeLoggerDao;
// use rand::seq::SliceRandom;
use lsys_core::db::{Insert, QueryBuilderExt, TableMeta, Update};
use sqlx::{Acquire, MySql, Pool, Transaction};
use std::{collections::HashMap, sync::Arc};

use crate::model::{AccountModel, AccountNameModel, AccountNameStatus};

use super::{AccountError, AccountIndex, AccountResult, logger::LogAccountName};

pub struct AccountName {
    db: Pool<MySql>,
    // fluent: Arc<FluentBuild>,
    index: Arc<AccountIndex>,
    pub(crate) cache: Arc<LocalCache<u64, AccountNameModel>>,
    logger: Arc<ChangeLoggerDao>,
}

impl AccountName {
    pub fn new(
        db: Pool<MySql>,
        index: Arc<AccountIndex>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        Self {
            cache: Arc::new(LocalCache::new(remote_notify, config)),
            db,
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
        let res = sqlx::query_as::<_, AccountNameModel>(&format!(
            "select * from {} where username=? and status=?",
            AccountNameModel::table_name(),
        ))
        .bind(&name)
        .bind(AccountNameStatus::Enable as i8)
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

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<_, AccountNameModel>::new()
            .set(AccountNameModel::USERNAME, &username)
            .set(AccountNameModel::CHANGE_TIME, ntime)
            .set(AccountNameModel::STATUS, status)
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("account_id", account.id);
            })
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
        let username_max = FetchField::new(&self.db)
            .string_max::<AccountNameModel>(&AccountNameModel::USERNAME)
            .await
            .len_or(32);

        ValidParam::default()
            .add(
                valid_key!("username"),
                &username,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(3, username_max))
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

        let account_name_res = sqlx::query_as::<_, AccountNameModel>(&format!(
            "select * from {} where username=?",
            AccountNameModel::table_name(),
        ))
        .bind(&username)
        .fetch_one(&self.db)
        .await;

        let out = match account_name_res {
            Err(sqlx::Error::RowNotFound) => {
                let account_name_res = sqlx::query_as::<_, AccountNameModel>(&format!(
                    "select * from {} where account_id=?",
                    AccountNameModel::table_name(),
                ))
                .bind(account.id)
                .fetch_one(db)
                .await;

                match account_name_res {
                    Err(sqlx::Error::RowNotFound) => {
                        let status = AccountNameStatus::Enable as i8;
                        let mut db = match transaction {
                            Some(pb) => pb.begin().await?,
                            None => self.db.begin().await?,
                        };
                        let tmp = Insert::<_, AccountNameModel>::new()
                            .set(AccountNameModel::ACCOUNT_ID, account.id)
                            .set(AccountNameModel::USERNAME, &username)
                            .set(AccountNameModel::STATUS, status)
                            .set(AccountNameModel::CHANGE_TIME, time)
                            .execute(&mut *db)
                            .await;
                        if let Err(ie) = tmp {
                            db.rollback().await?;
                            return Err(ie.into());
                        }
                        use lsys_core::db::Update;
                        let tmp = Update::<_, AccountModel>::new()
                            .set(AccountModel::USE_NAME, 1_i8)
                            .execute(&mut *db, |qb| {
                                qb.push_where().field_eq("id", account.id);
                            })
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
                        let mut db = match transaction {
                            Some(pb) => pb.begin().await?,
                            None => self.db.begin().await?,
                        };
                        let tmp = Update::<_, AccountNameModel>::new()
                            .set(AccountNameModel::STATUS, status)
                            .set(AccountNameModel::USERNAME, &username)
                            .set(AccountNameModel::CHANGE_TIME, time)
                            .execute(&mut *db, |qb| {
                                qb.push_where().field_eq("id", account_name.id);
                            })
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

    pub async fn find_by_account_id(&self, id: &u64) -> AccountResult<AccountNameModel> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountNameModel>::one(&self.db, |qb| {
            qb.field_eq("account_id", *id);
            qb.push(" ORDER BY id DESC");
        })
        .await?)
    }
    pub async fn find_by_account_ids(
        &self,
        ids: &[u64],
    ) -> AccountResult<HashMap<u64, AccountNameModel>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountNameModel>::map(
            &self.db,
            |qb| {
                qb.field_in_copied("account_id", ids);
                qb.push(" ORDER BY id DESC");
            },
            |v| v.account_id,
        )
        .await?)
    }
    pub fn cache(&'_ self) -> AccountNameCache<'_> {
        AccountNameCache { dao: self }
    }
}

pub struct AccountNameCache<'t> {
    pub dao: &'t AccountName,
}
impl AccountNameCache<'_> {
    pub async fn find_by_account_id(&self, id: &u64) -> AccountResult<AccountNameModel> {
        self.dao
            .cache
            .get_or_fetch(id, || self.dao.find_by_account_id(id))
            .await
    }
    pub async fn find_by_account_ids(
        &self,
        ids: &[u64],
    ) -> AccountResult<HashMap<u64, AccountNameModel>> {
        self.dao
            .cache
            .get_or_fetch_many(ids, |missing| async move {
                self.dao.find_by_account_ids(&missing).await
            })
            .await
    }
}
