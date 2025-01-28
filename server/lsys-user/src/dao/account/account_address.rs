use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    fluent_message, now_time, RemoteNotify, RequestEnv,
};

use lsys_core::db::{Insert, ModelTableName, SqlQuote, Update};
use lsys_core::sql_format;
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{Acquire, MySql, Pool, Transaction};
use std::{collections::HashMap, sync::Arc};

use crate::model::{
    AccountAddressModel, AccountAddressModelRef, AccountAddressStatus, AccountModel,
};

use super::{check_mobile, logger::LogAccountAddress, AccountError, AccountIndex, AccountResult};

pub struct AccountAddress {
    db: Pool<MySql>,
    // fluent: Arc<FluentBuild>,
    index: Arc<AccountIndex>,
    pub(crate) cache: Arc<LocalCache<u64, Vec<AccountAddressModel>>>,
    logger: Arc<ChangeLoggerDao>,
}

impl AccountAddress {
    pub fn new(
        db: Pool<MySql>,
        // fluent: Arc<FluentBuild>,
        index: Arc<AccountIndex>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        Self {
            cache: Arc::new(LocalCache::new(remote_notify, config)),
            db,
            //  fluent,
            index,
            logger,
        }
    }

    /// 添加用户地址
    pub async fn edit_address(
        &self,
        address: &AccountAddressModel,
        mut address_data: AccountAddressModelRef<'_>,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<()> {
        if !AccountAddressStatus::Enable.eq(address.status) {
            return Err(AccountError::System(
                fluent_message!("account-address-is-delete",
                    {
                        "id": address.id
                    }
                ),
            )); //format!("address is delete:{}", address.id)
        }
        macro_rules! check_data {
            ($addr_var:ident,$name:literal) => {
                let $addr_var = if let Some($addr_var) = address_data.$addr_var {
                    $addr_var.trim().to_owned()
                } else {
                    "".to_string()
                };
                if $addr_var.is_empty() {
                    return Err(AccountError::System(fluent_message!("account-address-not-empty",
                        {"name":$name}
                    )));//concat!("address {$name} can't be nul")
                }
                address_data.$addr_var = Some(&$addr_var);
            };
        }
        check_data!(address_info, "info");
        check_data!(address_code, "code");
        check_data!(address_detail, "detail");
        check_data!(name, "name");
        check_data!(mobile, "mobile");
        check_mobile("", &mobile)?;

        let time = now_time()?;
        address_data.change_time = Some(&time);
        address_data.account_id = Some(&address.account_id); //防止被外面给改了
        address_data.id = None; //防止被外面给改了

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::<sqlx::MySql, AccountAddressModel, _>::new(address_data)
            .execute_by_pk(address, &mut *db)
            .await;
        match tmp {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(_) => {
                if let Err(ie) = self
                    .index
                    .del(
                        crate::model::AccountIndexCat::Address,
                        address.account_id,
                        &[&address.address_info],
                        Some(&mut db),
                    )
                    .await
                {
                    db.rollback().await?;
                    return Err(ie);
                }
                if let Err(ie) = self
                    .index
                    .add(
                        crate::model::AccountIndexCat::Address,
                        address.account_id,
                        &[&address_info],
                        Some(&mut db),
                    )
                    .await
                {
                    db.rollback().await?;
                    return Err(ie);
                }
                db.commit().await?;
                self.cache.clear(&address.account_id).await;

                self.logger
                    .add(
                        &LogAccountAddress {
                            action: "edit",
                            address_code: &address_code,
                            address_info: &address_info,
                            address_detail: &address_detail,
                            name: &name,
                            mobile: &mobile,
                            account_id: address.account_id,
                        },
                        Some(address.id),
                        Some(op_user_id),
                        None,
                        env_data,
                    )
                    .await;

                Ok(())
            }
        }
    }
    /// 添加用户地址
    pub async fn add_address(
        &self,
        account: &AccountModel,
        mut address_data: AccountAddressModelRef<'_>,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        macro_rules! check_data {
            ($addr_var:ident,$name:literal) => {
                let $addr_var = if let Some($addr_var) = address_data.$addr_var {
                    $addr_var.trim().to_owned()
                } else {
                    "".to_string()
                };
                if $addr_var.is_empty() {
                    return Err(AccountError::System(fluent_message!("account-address-not-empty",
                        {"name":$name}
                    )));
                    // concat!("account-address-", $name, "-empty"),
                    // concat!("address ", $name, " can't be nul")
                }
                address_data.$addr_var = Some(&$addr_var);
            };
        }
        check_data!(address_info, "info");
        check_data!(address_code, "code");
        check_data!(address_detail, "detail");
        check_data!(name, "name");
        check_data!(mobile, "mobile");
        check_mobile("", &mobile)?;

        let address_res = sqlx::query_as::<_, AccountAddressModel>(&sql_format!(
            "select * from {} where  account_id={} and address_code={} and address_info={} and address_detail={} and name={} and mobile={} and status={}",
            AccountAddressModel::table_name(),
            account.id,
            address_code,
            address_info,
            address_detail,
            name,
            mobile,
            AccountAddressStatus::Enable
        ))
        .fetch_one(&self.db)
        .await;

        if let Ok(address) = address_res {
            return Ok(address.id);
        }

        let time = now_time()?;
        address_data.change_time = Some(&time);
        address_data.status = Some(&(AccountAddressStatus::Enable as i8));

        if address_data.account_id.is_none() {
            address_data.account_id = Some(&account.id);
        }

        let address_account_id = *address_data.account_id.unwrap_or(&account.id);

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let res = Insert::<sqlx::MySql, AccountAddressModel, _>::new(address_data)
            .execute(&mut *db)
            .await;
        match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                let res = sqlx::query(
                    sql_format!(
                        "UPDATE {} SET address_count=address_count+1 WHERE id=?",
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
                        Err(e.into())
                    }
                    Ok(_) => {
                        if let Err(ie) = self
                            .index
                            .add(
                                crate::model::AccountIndexCat::Address,
                                account.id,
                                &[&address_info],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }

                        db.commit().await?;
                        self.cache.clear(&account.id).await;

                        let aid = mr.last_insert_id();

                        self.logger
                            .add(
                                &LogAccountAddress {
                                    action: "add",
                                    address_code: &address_code,
                                    address_info: &address_info,
                                    address_detail: &address_detail,
                                    name: &name,
                                    mobile: &mobile,
                                    account_id: address_account_id,
                                },
                                Some(aid),
                                Some(op_user_id),
                                None,
                                env_data,
                            )
                            .await;

                        Ok(aid)
                    }
                }
            }
        }
    }
    /// 删除用户地址
    pub async fn del_address(
        &self,
        address: &AccountAddressModel,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        let time = now_time()?;
        let change = lsys_core::model_option_set!(AccountAddressModelRef,{
            status:AccountAddressStatus::Delete as i8,
            change_time:time,
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<sqlx::MySql, AccountAddressModel, _>::new(change)
            .execute_by_pk(address, &mut *db)
            .await;
        match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                let res = sqlx::query(
                    sql_format!(
                    "UPDATE {} SET address_count=address_count-1 WHERE id=? and address_count-1>=0",
                    AccountModel::table_name(),
                )
                    .as_str(),
                )
                .bind(address.account_id)
                .execute(&mut *db)
                .await;
                match res {
                    Err(e) => {
                        db.rollback().await?;
                        Err(e.into())
                    }
                    Ok(_) => {
                        if let Err(ie) = self
                            .index
                            .del(
                                crate::model::AccountIndexCat::Address,
                                address.account_id,
                                &[&address.address_info],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }

                        db.commit().await?;
                        self.cache.clear(&address.account_id).await;

                        self.logger
                            .add(
                                &LogAccountAddress {
                                    account_id: address.account_id,
                                    action: "del",
                                    address_code: &address.address_code,
                                    address_info: &address.address_info,
                                    address_detail: &address.address_detail,
                                    name: &address.name,
                                    mobile: &address.mobile,
                                },
                                Some(address.id),
                                Some(op_user_id),
                                None,
                                env_data,
                            )
                            .await;

                        Ok(mr.rows_affected())
                    }
                }
            }
        }
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        AccountAddressModel,
        AccountResult<AccountAddressModel>,
        id,
        "id={id}"
    );
    lsys_core::impl_dao_fetch_vec_by_one!(
        db,
        find_by_account_id_vec,
        u64,
        AccountAddressModel,
        AccountResult<Vec<AccountAddressModel>>,
        uid,
        "account_id = {uid} and status = {status}",
        status = AccountAddressStatus::Enable
    );
    lsys_core::impl_dao_fetch_vec_by_vec!(
        db,
        find_by_account_ids_vec,
        u64,
        AccountAddressModel,
        AccountResult<HashMap<u64, Vec<AccountAddressModel>>>,
        account_id,
        uid,
        "account_id in ({uid}) and status = {status}",
        status = AccountAddressStatus::Enable
    );
    pub fn cache(&'_ self) -> AccountAddressCache<'_> {
        AccountAddressCache { dao: self }
    }
}
pub struct AccountAddressCache<'t> {
    pub dao: &'t AccountAddress,
}
impl AccountAddressCache<'_> {
    lsys_core::impl_cache_fetch_one!(
        find_by_account_id_vec,
        dao,
        cache,
        u64,
        AccountResult<Vec<AccountAddressModel>>
    );
    lsys_core::impl_cache_fetch_vec!(
        find_by_account_ids_vec,
        dao,
        cache,
        u64,
        AccountResult<HashMap<u64, Vec<AccountAddressModel>>>
    );
}
