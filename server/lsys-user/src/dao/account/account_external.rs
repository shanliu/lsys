use std::collections::HashMap;
use std::sync::Arc;

use crate::dao::{AccountError, AccountResult};

use crate::model::{
    AccountExternalModel, AccountExternalModelRef, AccountExternalStatus, AccountModel,
};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{fluent_message, now_time, RemoteNotify, RequestEnv};

use super::logger::LogAccountExternal;
use super::AccountIndex;
use lsys_core::db::{Insert, ModelTableName, SqlQuote, Update};
use lsys_core::{db_option_executor, model_option_set, sql_format};
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
        let res = sqlx::query_as::<_, AccountExternalModel>(&sql_format!(
            "select * from {} where config_name={} and external_type={} and external_id={} and status={} order by id desc",
            AccountExternalModel::table_name(),
            config_name,external_type,external_id,AccountExternalStatus::Enable
        ))
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
        let res = sqlx::query_as::<_, AccountExternalModel>(&sql_format!(
            "select * from {} where account_id={} and config_name={} and external_type={} and external_id={} and status = {} order by id desc",
            AccountExternalModel::table_name(),
            account.id,
                    config_name,
                    external_type,
                    external_id,
                    AccountExternalStatus::Enable
        ))
        .fetch_one(&self.db)
        .await?;

        Ok(res)
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
        let db = &self.db;
        let account_ext_res = sqlx::query_as::<_, AccountExternalModel>(&sql_format!(
            "select * from {} where config_name={} and  external_type={} and external_id={} and status = {}",
            AccountExternalModel::table_name(),
            config_name,
            external_type,
            external_id,
            AccountExternalStatus::Enable
        ))
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
                let change = lsys_core::model_option_set!(AccountExternalModelRef,{
                    status:AccountExternalStatus::Enable as i8,
                    external_name:external_name_ow,
                    change_time:time
                });
                db_option_executor!(
                    db,
                    {
                        Update::< AccountExternalModel, _>::new(change)
                            .execute_by_pk(&account_ext, db.as_executor())
                            .await?;
                    },
                    transaction,
                    db
                );
                account_ext.id
            }
            Err(sqlx::Error::RowNotFound) => {
                let external_name_ow = external_name.to_owned();
                let external_id_ow = external_id.to_owned();
                let external_type_ow = external_type.to_owned();
                let config_name_ow = config_name.to_owned();

                let new_data = model_option_set!(AccountExternalModelRef,{
                    account_id:account.id,
                    status:AccountExternalStatus::Enable as i8,
                    config_name:config_name_ow,
                    external_type:external_type_ow,
                    external_id:external_id_ow,
                    external_name:external_name_ow,
                    change_time:time,
                });

                let mut db = match transaction {
                    Some(pb) => pb.begin().await?,
                    None => db.begin().await?,
                };
                let res = Insert::<AccountExternalModel, _>::new(new_data)
                    .execute(&mut *db)
                    .await;
                match res {
                    Err(e) => {
                        db.rollback().await?;
                        return Err(e.into());
                    }
                    Ok(mr) => {
                        let res = sqlx::query(
                            sql_format!(
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
        let time = now_time()?;
        let external_name_ow = external_name.to_string();
        let token_data_ow = token_data.to_string();
        let mut change = lsys_core::model_option_set!(AccountExternalModelRef,{
            external_name:external_name_ow,
            token_data:token_data_ow,
            token_timeout:token_timeout,
            change_time:time,
        });
        let external_link_ow = external_link.map(|e| e.to_string());
        change.external_link = external_link_ow.as_ref();
        let external_gender_ow = external_link.map(|e| e.to_string());
        change.external_gender = external_gender_ow.as_ref();
        let external_pic_ow = external_link.map(|e| e.to_string());
        change.external_pic = external_pic_ow.as_ref();
        let external_nikename_ow = external_link.map(|e| e.to_string());
        change.external_nikename = external_nikename_ow.as_ref();
        Update::< AccountExternalModel, _>::new(change)
            .execute_by_pk(account_ext, &self.db)
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
        let change = lsys_core::model_option_set!(AccountExternalModelRef,{
            status:AccountExternalStatus::Delete as i8,
            change_time:time
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::< AccountExternalModel, _>::new(change)
            .execute_by_pk(account_ext, &mut *db)
            .await;
        let out = match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                let res=sqlx::query(sql_format!(
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
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        AccountExternalModel,
        AccountResult<AccountExternalModel>,
        id,
        "id={id}  and status = {status}",
        status = AccountExternalStatus::Enable as i8
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        AccountExternalModel,
        AccountResult<HashMap<u64, AccountExternalModel>>,
        id,
        ids,
        "id in ({ids}) and status = {status}",
        status = AccountExternalStatus::Enable as i8
    );
    lsys_core::impl_dao_fetch_vec_by_one!(
        db,
        find_by_account_id_vec,
        u64,
        AccountExternalModel,
        AccountResult<Vec<AccountExternalModel>>,
        uid,
        "account_id = {uid} and status = {status}",
        status = AccountExternalStatus::Enable as i8
    );
    lsys_core::impl_dao_fetch_vec_by_vec!(
        db,
        find_by_account_ids_vec,
        u64,
        AccountExternalModel,
        AccountResult<HashMap<u64, Vec<AccountExternalModel>>>,
        account_id,
        uid,
        "account_id in ({uid}) and status = {status}",
        status = AccountExternalStatus::Enable as i8
    );
    pub fn cache(&'_ self) -> AccountExternalCache<'_> {
        AccountExternalCache { dao: self }
    }
}

pub struct AccountExternalCache<'t> {
    pub dao: &'t AccountExternal,
}
impl AccountExternalCache<'_> {
    lsys_core::impl_cache_fetch_one!(
        find_by_id,
        dao,
        cache,
        u64,
        AccountResult<AccountExternalModel>
    );
    lsys_core::impl_cache_fetch_vec!(
        find_by_ids,
        dao,
        cache,
        u64,
        AccountResult<HashMap<u64, AccountExternalModel>>
    );
    pub async fn find_by_account_id_vec(
        &self,
        account_id: &u64,
    ) -> AccountResult<Vec<AccountExternalModel>> {
        match self.dao.account_cache.get(account_id).await {
            Some(ids) => Ok(self
                .find_by_ids(&ids)
                .await?
                .into_iter()
                .map(|e| e.1)
                .collect::<Vec<_>>()),
            None => {
                let rows = self.dao.find_by_account_id_vec(account_id).await?;
                for tmp in rows.clone() {
                    self.dao.cache.set(tmp.id, tmp, 0).await;
                }
                let ids = rows.iter().map(|e| e.id).collect::<Vec<_>>();
                self.dao.account_cache.set(*account_id, ids, 0).await;
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
