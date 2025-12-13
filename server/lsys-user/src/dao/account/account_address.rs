use lsys_core::{
    RemoteNotify, RequestEnv, ValidMobile, ValidNumber, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen, cache::{LocalCache, LocalCacheConfig}, db::WhereOption, now_time, valid_key
};

use lsys_core::db::{Insert, ModelTableName, SqlQuote, Update};
use lsys_core::sql_format;
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{Acquire, MySql, Pool, Transaction};
use std::{collections::HashMap, sync::Arc};

use crate::model::{
    AccountAddressModel, AccountAddressModelRef, AccountAddressStatus, AccountModel,
};

use super::{logger::LogAccountAddress, AccountIndex, AccountResult};

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
}
pub struct AccountAddressParam<'t> {
    pub country_code: &'t str,

    pub address_code: &'t str,

    pub address_info: &'t str,

    pub address_detail: &'t str,

    pub name: &'t str,

    pub mobile: &'t str,
}
impl AccountAddress {
    async fn address_param_valid(
        &self,
        statis: Option<i8>,
        address_data: &AccountAddressParam<'_>,
    ) -> AccountResult<()> {
        let mut valid_param = ValidParam::default();
        if let Some(statis) = statis {
            valid_param.add(
                valid_key!("address_status"),
                &statis,
                &ValidParamCheck::default()
                    .add_rule(ValidNumber::eq(AccountAddressStatus::Enable as i8)),
            );
        }

        valid_param.add(
            valid_key!("address_name"),
            &address_data.name,
            &ValidParamCheck::default()
                .add_rule(ValidPattern::NotFormat)
                .add_rule(ValidStrlen::range(4, 16)),
        );
        valid_param.add(
            valid_key!("address_country_code"),
            &address_data.country_code,
            &ValidParamCheck::default().add_rule(ValidStrlen::range(1, 21)),
        );
        valid_param.add(
            valid_key!("address_code"),
            &address_data.address_code,
            &ValidParamCheck::default()
                .add_rule(ValidPattern::Numeric)
                .add_rule(ValidStrlen::range(4, 21)),
        );
        valid_param.add(
            valid_key!("address_mobile"),
            &address_data.mobile,
            &ValidParamCheck::default().add_rule(ValidMobile::default()),
        );

        valid_param.add(
            valid_key!("address_info"),
            &address_data.address_info,
            &ValidParamCheck::default()
                .add_rule(ValidPattern::NotFormat)
                .add_rule(ValidStrlen::range(1, 64)),
        );

        valid_param.add(
            valid_key!("address_detail"),
            &address_data.address_detail,
            &ValidParamCheck::default()
                .add_rule(ValidPattern::NotFormat)
                .add_rule(ValidStrlen::range(1, 128)),
        );

        Ok(())
    }

    /// 添加用户地址
    pub async fn edit_address(
        &self,
        address: &AccountAddressModel,
        address_param: &AccountAddressParam<'_>,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<()> {
        self.address_param_valid(Some(address.status), address_param)
            .await?;

        let time = now_time()?;
        let country_code = address_param.country_code.to_owned();
        let address_code = address_param.address_code.to_owned();
        let address_info = address_param.address_info.to_owned();
        let address_detail = address_param.address_detail.to_owned();
        let name = address_param.name.to_owned();
        let mobile = address_param.mobile.to_owned();
        let address_data = lsys_core::model_option_set!(AccountAddressModelRef,{
            change_time:time,
            country_code:country_code,
            address_code:address_code,
            address_info:address_info,
            address_detail:address_detail,
            name:name,
            mobile:mobile,
            account_id:address.account_id
        });

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::<AccountAddressModel, _>::new(address_data)
            .execute_by_where(
                &WhereOption::Where(sql_format!("id={}", address.id)),
                &mut *db,
            )
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
                            action: "edit",
                            address_code: &address.address_code,
                            address_info: &address.address_info,
                            address_detail: &address.address_detail,
                            name: &address.name,
                            mobile: &address.mobile,
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
        address_param: &AccountAddressParam<'_>,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        self.address_param_valid(None, address_param).await?;

        let time = now_time()?;
        let country_code = address_param.country_code.to_owned();
        let address_code = address_param.address_code.to_owned();
        let address_info = address_param.address_info.to_owned();
        let address_detail = address_param.address_detail.to_owned();
        let name = address_param.name.to_owned();
        let mobile = address_param.mobile.to_owned();
        let address_data = lsys_core::model_option_set!(AccountAddressModelRef,{
            status:AccountAddressStatus::Enable as i8,
            change_time:time,
            country_code:country_code,
            address_code:address_code,
            address_info:address_info,
            address_detail:address_detail,
            name:name,
            mobile:mobile,
            account_id:account.id,
        });
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

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let res = Insert::<AccountAddressModel, _>::new(address_data)
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
        let res = Update::<AccountAddressModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("id={}", address.id)),
                &mut *db,
            )
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
