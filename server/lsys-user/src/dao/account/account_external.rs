use std::collections::HashMap;
use std::sync::Arc;

use crate::dao::{AccountError, AccountResult};

use crate::model::{
    AccountExternalModel, AccountExternalStatus, AccountModel,
};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{
    db::utils::FetchField, fluent_message, valid_key,
};
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::utils::{
    now_time, string_clear, RequestEnv, StringClear, STRING_CLEAR_FORMAT,
};
use lsys_core::valid_param::{ValidParam, ValidParamCheck, ValidPattern, ValidStrlen, ValidUrl};

use super::logger::LogAccountExternal;
use super::AccountIndex;
use lsys_core::db::{Insert, TableMeta, QueryBuilderExt, Update, OptionTxExecutor};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{Acquire, MySql, Pool, Transaction};

pub struct AccountExternal {
    db: Pool<MySql>,
    index: Arc<AccountIndex>,
    // fluent: Arc<FluentBuild>,
    pub(crate) cache: Arc<LocalCache<u64, AccountExternalModel>>,
    pub(crate) account_cache: Arc<LocalCache<u64, Vec<u64>>>,
    logger: Arc<ChangeLoggerDao>,
}

impl AccountExternal {
    pub fn new(
        db: Pool<MySql>,
        index: Arc<AccountIndex>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        Self {
            cache: Arc::new(LocalCache::new(remote_notify.clone(), config)),
            account_cache: Arc::new(LocalCache::new(remote_notify, config)),
            db,
            index,
            logger,
        }
    }

    /// 根据第三方信息查找记录
    pub async fn find_by_external(
        &self,
        config_name: &str,
        external_type: &str,
        external_id: &str,
    ) -> AccountResult<AccountExternalModel> {
        let config_name = string_clear(
            config_name,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(33),
        );
        let external_type = string_clear(
            external_type,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(65),
        );
        let external_id = string_clear(
            external_id,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(126),
        );
        if config_name.is_empty() || external_type.is_empty() || external_id.is_empty() {
            return Err(sqlx::Error::RowNotFound.into());
        }
        let res = sqlx::query_as::<_, AccountExternalModel>(&format!(
            "select * from {} where config_name=? and external_type=? and external_id=? and status=? order by id desc",
            AccountExternalModel::table_name(),
        ))
        .bind(&config_name)
        .bind(&external_type)
        .bind(&external_id)
        .bind(AccountExternalStatus::Enable as i8)
        .fetch_one(&self.db)
        .await?;

        Ok(res)
    }
    /// 根据用户跟第三方id查找记录
    pub async fn find_by_account_external(
        &self,
        account: &AccountModel,
        config_name: &str,
        external_type: &str,
        external_id: &str,
    ) -> AccountResult<AccountExternalModel> {
        let config_name = string_clear(
            config_name,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(33),
        );
        let external_type = string_clear(
            external_type,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(65),
        );
        let external_id = string_clear(
            external_id,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(126),
        );
        if config_name.is_empty() || external_type.is_empty() || external_id.is_empty() {
            return Err(sqlx::Error::RowNotFound.into());
        }
        let res = sqlx::query_as::<_, AccountExternalModel>(&format!(
            "select * from {} where account_id=? and config_name=? and external_type=? and external_id=? and status=? order by id desc",
            AccountExternalModel::table_name(),
        ))
        .bind(account.id)
        .bind(&config_name)
        .bind(&external_type)
        .bind(&external_id)
        .bind(AccountExternalStatus::Enable as i8)
        .fetch_one(&self.db)
        .await?;

        Ok(res)
    }
    async fn external_param_valid(
        &self,
        config_name: &str,
        external_type: &str,
        external_id: &str,
        external_name: &str,
    ) -> AccountResult<()> {
        let fetch_field = FetchField::new(&self.db);
        let config_name_max = fetch_field.string_max::<AccountExternalModel>(&AccountExternalModel::CONFIG_NAME)
            .await
            .len_or(32);
        let external_type_max = fetch_field.string_max::<AccountExternalModel>(&AccountExternalModel::EXTERNAL_TYPE)
            .await
            .len_or(64);
        let external_id_max = fetch_field.string_max::<AccountExternalModel>(&AccountExternalModel::EXTERNAL_ID)
            .await
            .len_or(125);
        let external_name_max = fetch_field.string_max::<AccountExternalModel>(&AccountExternalModel::EXTERNAL_NAME)
            .await
            .len_or(256);

        ValidParam::default()
            .add(
                valid_key!("external_config_name"),
                &config_name,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, config_name_max)),
            )
            .add(
                valid_key!("external_type"),
                &external_type,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, external_type_max)),
            )
            .add(
                valid_key!("external_id"),
                &external_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, external_id_max)),
            )
            .add(
                valid_key!("external_name"),
                &external_name,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, external_name_max)),
            )
            .check()?;
        Ok(())
    }
    /// 新增第三方登录信息
    #[allow(clippy::too_many_arguments)]
    pub async fn add_external(
        &self,
        account: &AccountModel,
        config_name: &str,
        external_type: &str,
        external_id: &str,
        external_name: &str,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        self.external_param_valid(config_name, external_type, external_id, external_name)
            .await?;
        let db = &self.db;
        let account_ext_res = sqlx::query_as::<_, AccountExternalModel>(&format!(
            "select * from {} where config_name=? and  external_type=? and external_id=? and status=?",
            AccountExternalModel::table_name(),
        ))
        .bind(config_name)
        .bind(external_type)
        .bind(external_id)
        .bind(AccountExternalStatus::Enable as i8)
        .fetch_one(&self.db)
        .await;

        let time = now_time()?;
        let aid = match account_ext_res {
            Ok(account_ext) => {
                if account_ext.account_id != account.id {
                    return Err(AccountError::System(
                        fluent_message!("account-external-other-bind",
                            {"name":external_name,"id":account.id }
                        ),
                    )); //"this account {$name} bind in other account[{$id}]",
                }
                let external_name_ow = external_name.to_owned();
                Update::<_,AccountExternalModel>::new()
                    .set(AccountExternalModel::STATUS, AccountExternalStatus::Enable as i8)
                    .set(AccountExternalModel::EXTERNAL_NAME, external_name_ow)
                    .set(AccountExternalModel::CHANGE_TIME, time)
                    .execute(OptionTxExecutor::new(transaction, &self.db), |qb| {
                        qb.push_where().field_eq("id", account_ext.id);
                    })
                    .await?;
                account_ext.id
            }
            Err(sqlx::Error::RowNotFound) => {
                let external_name_ow = external_name.to_owned();
                let external_id_ow = external_id.to_owned();
                let external_type_ow = external_type.to_owned();
                let config_name_ow = config_name.to_owned();

                let mut db = match transaction {
                    Some(pb) => pb.begin().await?,
                    None => db.begin().await?,
                };
                let res = Insert::<_,AccountExternalModel>::new()
                    .set(AccountExternalModel::ACCOUNT_ID, account.id)
                    .set(AccountExternalModel::STATUS, AccountExternalStatus::Enable as i8)
                    .set(AccountExternalModel::CONFIG_NAME, config_name_ow)
                    .set(AccountExternalModel::EXTERNAL_TYPE, external_type_ow)
                    .set(AccountExternalModel::EXTERNAL_ID, external_id_ow)
                    .set(AccountExternalModel::EXTERNAL_NAME, external_name_ow)
                    .set(AccountExternalModel::CHANGE_TIME, time)
                    .execute(&mut *db)
                    .await;
                match res {
                    Err(e) => {
                        db.rollback().await?;
                        return Err(e.into());
                    }
                    Ok(mr) => {
                        let res = sqlx::query(
                            format!(
                                "UPDATE {} SET external_count=external_count+1 WHERE id=?",
                                AccountModel::table_name(),
                            )
                            .as_str(),
                        )
                        .bind(account.id)
                        .execute(&mut *db)
                        .await;
                        match res {
                            Err(e) => {
                                db.rollback().await?;
                                return Err(e.into());
                            }
                            Ok(_) => {
                                if let Err(ie) = self
                                    .index
                                    .add(
                                        crate::model::AccountIndexCat::ExternalType,
                                        account.id,
                                        &[external_type],
                                        Some(&mut db),
                                    )
                                    .await
                                {
                                    db.rollback().await?;
                                    return Err(ie);
                                }

                                db.commit().await?;
                                self.account_cache.clear(&account.id).await;
                                mr.last_insert_id()
                            }
                        }
                    }
                }
            }
            Err(err) => return Err(err.into()),
        };

        self.logger
            .add(
                &LogAccountExternal {
                    action: "add",
                    config_name,
                    external_type,
                    external_id,
                    external_name,
                    external_gender: "",
                    external_link: "",
                    external_pic: "",
                    external_nikename: "",
                    status: AccountExternalStatus::Enable as i8,
                    token_data: "",
                    token_timeout: 0,
                    account_id: account.id,
                },
                Some(aid),
                Some(op_user_id),
                None,
                env_data,
            )
            .await;

        Ok(aid)
    }
    async fn token_update_param_valid(
        &self,
        external_name: &str,
        token_data: &str,
        external_nikename: Option<&str>,
        external_gender: Option<&str>,
        external_link: Option<&str>,
        external_pic: Option<&str>,
    ) -> AccountResult<()> {
        let fetch_field = FetchField::new(&self.db);
        let external_name_max = fetch_field.string_max::<AccountExternalModel>(&AccountExternalModel::EXTERNAL_NAME)
            .await
            .len_or(256);
        let token_data_max = fetch_field.string_max::<AccountExternalModel>(&AccountExternalModel::TOKEN_DATA)
            .await
            .len_or(256);
        let external_nikename_max = fetch_field.string_max::<AccountExternalModel>(&AccountExternalModel::EXTERNAL_NIKENAME)
            .await
            .len_or(65);
        let external_gender_max = fetch_field.string_max::<AccountExternalModel>(&AccountExternalModel::EXTERNAL_GENDER)
            .await
            .len_or(8);
        let external_link_max = fetch_field.string_max::<AccountExternalModel>(&AccountExternalModel::EXTERNAL_LINK)
            .await
            .len_or(255);
        let external_pic_max = fetch_field.string_max::<AccountExternalModel>(&AccountExternalModel::EXTERNAL_PIC)
            .await
            .len_or(512);

        let mut param_valid = ValidParam::default();
        param_valid
            .add(
                valid_key!("external_name"),
                &external_name,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, external_name_max)),
            )
            .add(
                valid_key!("external_token_data"),
                &token_data,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, token_data_max)),
            );
        if let Some(external_nikename) = external_nikename {
            param_valid.add(
                valid_key!("external_nikename"),
                &external_nikename,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(0, external_nikename_max)),
            );
        }
        if let Some(external_gender) = external_gender {
            param_valid.add(
                valid_key!("external_gender"),
                &external_gender,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, external_gender_max)),
            );
        }
        if let Some(external_link) = external_link {
            param_valid.add(
                valid_key!("external_link"),
                &external_link,
                &ValidParamCheck::default()
                    .add_rule(ValidUrl::default())
                    .add_rule(ValidStrlen::range(9, external_link_max)),
            );
        }
        if let Some(external_pic) = external_pic {
            param_valid.add(
                valid_key!("external_pic"),
                &external_pic,
                &ValidParamCheck::default()
                    .add_rule(ValidUrl::default())
                    .add_rule(ValidStrlen::range(9, external_pic_max)),
            );
        }
        param_valid.check()?;
        Ok(())
    }
    /// 刷新第三方登录token
    #[allow(clippy::too_many_arguments)]
    pub async fn token_update(
        &self,
        account_ext: &AccountExternalModel,
        external_name: &str,
        token_data: &str,
        token_timeout: u64,
        external_nikename: Option<&str>,
        external_gender: Option<&str>,
        external_link: Option<&str>,
        external_pic: Option<&str>,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<()> {
        self.token_update_param_valid(
            external_name,
            token_data,
            external_nikename,
            external_gender,
            external_link,
            external_pic,
        )
        .await?;
        let time = now_time()?;
        let external_name_ow = external_name.to_string();
        let token_data_ow = token_data.to_string();
        let mut update = Update::<_,AccountExternalModel>::new()
            .set(AccountExternalModel::EXTERNAL_NAME, external_name_ow)
            .set(AccountExternalModel::TOKEN_DATA, token_data_ow)
            .set(AccountExternalModel::TOKEN_TIMEOUT, token_timeout)
            .set(AccountExternalModel::CHANGE_TIME, time);
        if let Some(link) = external_link {
            update = update.set(AccountExternalModel::EXTERNAL_LINK, link.to_string());
        }
        if let Some(gender) = external_gender {
            update = update.set(AccountExternalModel::EXTERNAL_GENDER, gender.to_string());
        }
        if let Some(pic) = external_pic {
            update = update.set(AccountExternalModel::EXTERNAL_PIC, pic.to_string());
        }
        if let Some(nikename) = external_nikename {
            update = update.set(AccountExternalModel::EXTERNAL_NIKENAME, nikename.to_string());
        }
        update
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", account_ext.id);
            })
            .await?;
        self.cache.clear(&account_ext.id).await;
        self.account_cache.clear(&account_ext.account_id).await;

        self.logger
            .add(
                &LogAccountExternal {
                    action: "update",
                    config_name: &account_ext.config_name,
                    external_type: &account_ext.external_type,
                    external_id: &account_ext.external_id,
                    external_name,
                    external_gender: external_gender.unwrap_or_default(),
                    external_link: external_link.unwrap_or_default(),
                    external_pic: external_pic.unwrap_or_default(),
                    external_nikename: external_nikename.unwrap_or_default(),
                    status: AccountExternalStatus::Enable as i8,
                    token_data,
                    account_id: account_ext.account_id,
                    token_timeout,
                },
                Some(account_ext.id),
                Some(op_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
    /// 删除用户外部账号
    pub async fn del_external(
        &self,
        account_ext: &AccountExternalModel,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        if AccountExternalStatus::Delete.eq(account_ext.status) {
            return Ok(0_u64);
        }
        let time = now_time()?;
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<_,AccountExternalModel>::new()
            .set(AccountExternalModel::STATUS, AccountExternalStatus::Delete as i8)
            .set(AccountExternalModel::CHANGE_TIME, time)
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("id", account_ext.id);
            })
            .await;
        let out = match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                let res=sqlx::query(format!(
                        "UPDATE {} SET external_count=external_count-1 WHERE id=? and external_count-1>=0",
                        AccountModel::table_name(),
                    ).as_str())
                    .bind(account_ext.account_id)
                    .execute(&mut *db).await;
                match res {
                    Err(e) => {
                        db.rollback().await?;
                        Err(e.into())
                    }
                    Ok(_) => {
                        if let Err(ie) = self
                            .index
                            .del(
                                crate::model::AccountIndexCat::ExternalType,
                                account_ext.account_id,
                                &[&account_ext.external_type],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }

                        db.commit().await?;
                        self.cache.clear(&account_ext.id).await;
                        self.account_cache.clear(&account_ext.account_id).await;

                        Ok(mr.rows_affected())
                    }
                }
            }
        };

        self.logger
            .add(
                &LogAccountExternal {
                    action: "del",
                    config_name: &account_ext.config_name,
                    external_type: &account_ext.external_type,
                    external_id: &account_ext.external_id,
                    external_name: &account_ext.external_name,
                    external_gender: &account_ext.external_gender,
                    external_link: &account_ext.external_link,
                    external_pic: &account_ext.external_pic,
                    external_nikename: &account_ext.external_nikename,
                    status: AccountExternalStatus::Delete as i8,
                    token_data: &account_ext.token_data,
                    token_timeout: account_ext.token_timeout,
                    account_id: account_ext.account_id,
                },
                Some(account_ext.id),
                Some(op_user_id),
                None,
                env_data,
            )
            .await;

        out
    }
    pub async fn find_by_id(&self, id: &u64) -> AccountResult<AccountExternalModel> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountExternalModel>::one(
            &self.db,
            |qb| {
                qb.field_eq("id", *id);
                qb.push_and().field_eq("status", AccountExternalStatus::Enable as i8);
            },
        ).await?)
    }
    pub async fn find_by_ids(&self, ids: &[u64]) -> AccountResult<HashMap<u64, AccountExternalModel>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountExternalModel>::map(
            &self.db,
            |qb| {
                qb.field_in_copied("id", ids);
                qb.push_and().field_eq("status", AccountExternalStatus::Enable as i8);
            },
            |v| v.id,
        ).await?)
    }
    pub async fn find_by_account_id_vec(&self, id: &u64) -> AccountResult<Vec<AccountExternalModel>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountExternalModel>::vec(
            &self.db,
            |qb| {
                qb.field_eq("account_id", *id);
                qb.push_and().field_eq("status", AccountExternalStatus::Enable as i8);
            },
        ).await?)
    }
    pub async fn find_by_account_ids_vec(&self, ids: &[u64]) -> AccountResult<HashMap<u64, Vec<AccountExternalModel>>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountExternalModel>::group(
            &self.db,
            |qb| {
                qb.field_in_copied("account_id", ids);
                qb.push_and().field_eq("status", AccountExternalStatus::Enable as i8);
            },
            |v| v.account_id,
        ).await?)
    }
    pub fn cache(&'_ self) -> AccountExternalCache<'_> {
        AccountExternalCache { dao: self }
    }
}

pub struct AccountExternalCache<'t> {
    pub dao: &'t AccountExternal,
}
impl AccountExternalCache<'_> {
    pub async fn find_by_id(&self, id: &u64) -> AccountResult<AccountExternalModel> {
        self.dao
            .cache
            .get_or_fetch(id, || self.dao.find_by_id(id))
            .await
    }
    pub async fn find_by_ids(&self, ids: &[u64]) -> AccountResult<HashMap<u64, AccountExternalModel>> {
        self.dao
            .cache
            .get_or_fetch_many(ids, |missing| async move {
                self.dao.find_by_ids(&missing).await
            })
            .await
    }
    pub async fn find_by_account_id_vec(
        &self,
        account_id: u64,
    ) -> AccountResult<Vec<AccountExternalModel>> {
        match self.dao.account_cache.get(&account_id).await {
            Some(ids) => Ok(self
                .find_by_ids(&ids)
                .await?
                .into_iter()
                .map(|e| e.1)
                .collect::<Vec<_>>()),
            None => {
                let rows = self.dao.find_by_account_id_vec(&account_id).await?;
                for tmp in rows.clone() {
                    self.dao.cache.set(tmp.id, tmp, 0).await;
                }
                let ids = rows.iter().map(|e| e.id).collect::<Vec<_>>();
                self.dao.account_cache.set(account_id, ids, 0).await;
                Ok(rows)
            }
        }
    }
    pub async fn find_by_account_ids_vec(
        &self,
        account_ids: &[u64],
    ) -> AccountResult<HashMap<u64, Vec<AccountExternalModel>>> {
        let mut get = vec![];
        let mut hash = std::collections::HashMap::with_capacity(account_ids.len());
        for account_id in account_ids {
            match self.dao.account_cache.get(account_id).await {
                Some(ids) => {
                    let data = self
                        .find_by_ids(&ids)
                        .await?
                        .into_iter()
                        .map(|e| e.1)
                        .collect::<Vec<AccountExternalModel>>();
                    hash.entry(*account_id).or_insert(data);
                }
                None => {
                    get.push(*account_id);
                }
            }
        }
        if !get.is_empty() {
            match self.dao.find_by_account_ids_vec(&get).await {
                Ok(datas) => {
                    for (pk, rows) in datas.into_iter() {
                        hash.entry(pk).or_default().extend(rows.clone());
                        for tmp in rows.clone() {
                            self.dao.cache.set(tmp.id, tmp, 0).await;
                        }
                        let ids = rows.iter().map(|e| e.id).collect::<Vec<_>>();
                        self.dao.account_cache.set(pk, ids, 0).await;
                    }
                }
                Err(err) => return Err(err),
            }
        }
        Ok(hash)
    }
}
