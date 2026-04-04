use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    db::utils::FetchField,
    valid_key,
};
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::utils::{now_time, RequestEnv};
use lsys_core::valid_param::{
    ValidMobile, ValidNumber, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen,
};

use lsys_core::db::{Insert, TableMeta, QueryBuilderExt, Update};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{Acquire, MySql, Pool, Transaction};
use std::{collections::HashMap, sync::Arc};

use crate::model::{
    AccountAddressModel, AccountAddressStatus, AccountModel,
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
        let fetch_field = FetchField::new(&self.db);
        let name_max = fetch_field.string_max::<AccountAddressModel>(&AccountAddressModel::NAME)
            .await
            .len_or(16);
        let country_code_max = fetch_field.string_max::<AccountAddressModel>(&AccountAddressModel::COUNTRY_CODE)
            .await
            .len_or(21);
        let address_code_max = fetch_field.string_max::<AccountAddressModel>(&AccountAddressModel::ADDRESS_CODE)
            .await
            .len_or(21);
        let address_info_max = fetch_field.string_max::<AccountAddressModel>(&AccountAddressModel::ADDRESS_INFO)
            .await
            .len_or(64);
        let address_detail_max = fetch_field.string_max::<AccountAddressModel>(&AccountAddressModel::ADDRESS_DETAIL)
            .await
            .len_or(128);

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
                .add_rule(ValidStrlen::range(4, name_max)),
        );
        valid_param.add(
            valid_key!("address_country_code"),
            &address_data.country_code,
            &ValidParamCheck::default().add_rule(ValidStrlen::range(1, country_code_max)),
        );
        valid_param.add(
            valid_key!("address_code"),
            &address_data.address_code,
            &ValidParamCheck::default()
                .add_rule(ValidPattern::Numeric)
                .add_rule(ValidStrlen::range(4, address_code_max)),
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
                .add_rule(ValidStrlen::range(1, address_info_max)),
        );

        valid_param.add(
            valid_key!("address_detail"),
            &address_data.address_detail,
            &ValidParamCheck::default()
                .add_rule(ValidPattern::NotFormat)
                .add_rule(ValidStrlen::range(1, address_detail_max)),
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

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::<_,AccountAddressModel>::new()
            .set(AccountAddressModel::CHANGE_TIME, time)
            .set(AccountAddressModel::COUNTRY_CODE, country_code)
            .set(AccountAddressModel::ADDRESS_CODE, address_code)
            .set(AccountAddressModel::ADDRESS_INFO, &address_info)
            .set(AccountAddressModel::ADDRESS_DETAIL, address_detail)
            .set(AccountAddressModel::NAME, name)
            .set(AccountAddressModel::MOBILE, mobile)
            .set(AccountAddressModel::ACCOUNT_ID, address.account_id)
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("id", address.id);
            })
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
        let address_res = sqlx::query_as::<_, AccountAddressModel>(&format!(
            "select * from {} where  account_id=? and address_code=? and address_info=? and address_detail=? and name=? and mobile=? and status=?",
            AccountAddressModel::table_name(),
        ))
        .bind(account.id)
        .bind(&address_code)
        .bind(&address_info)
        .bind(&address_detail)
        .bind(&name)
        .bind(&mobile)
        .bind(AccountAddressStatus::Enable as i8)
        .fetch_one(&self.db)
        .await;

        if let Ok(address) = address_res {
            return Ok(address.id);
        }

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let res = Insert::<_,AccountAddressModel>::new()
            .set(AccountAddressModel::STATUS, AccountAddressStatus::Enable as i8)
            .set(AccountAddressModel::CHANGE_TIME, time)
            .set(AccountAddressModel::COUNTRY_CODE, country_code)
            .set(AccountAddressModel::ADDRESS_CODE, &address_code)
            .set(AccountAddressModel::ADDRESS_INFO, &address_info)
            .set(AccountAddressModel::ADDRESS_DETAIL, &address_detail)
            .set(AccountAddressModel::NAME, &name)
            .set(AccountAddressModel::MOBILE, &mobile)
            .set(AccountAddressModel::ACCOUNT_ID, account.id)
            .execute(&mut *db)
            .await;
        match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                let res = sqlx::query(
                    format!(
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
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<_,AccountAddressModel>::new()
            .set(AccountAddressModel::STATUS, AccountAddressStatus::Delete as i8)
            .set(AccountAddressModel::CHANGE_TIME, time)
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("id", address.id);
            })
            .await;
        match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                let res = sqlx::query(
                    format!(
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
    pub async fn find_by_id(&self, id: &u64) -> AccountResult<AccountAddressModel> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountAddressModel>::one(
            &self.db,
            |qb| { qb.field_eq("id", *id); },
        ).await?)
    }
    pub async fn find_by_account_id_vec(&self, id: &u64) -> AccountResult<Vec<AccountAddressModel>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountAddressModel>::vec(
            &self.db,
            |qb| {
                qb.field_eq("account_id", *id);
                qb.push_and().field_eq("status", AccountAddressStatus::Enable as i8);
            },
        ).await?)
    }
    pub async fn find_by_account_ids_vec(&self, ids: &[u64]) -> AccountResult<HashMap<u64, Vec<AccountAddressModel>>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountAddressModel>::group(
            &self.db,
            |qb| {
                qb.field_in_copied("account_id", ids);
                qb.push_and().field_eq("status", AccountAddressStatus::Enable as i8);
            },
            |v| v.account_id,
        ).await?)
    }
    pub fn cache(&'_ self) -> AccountAddressCache<'_> {
        AccountAddressCache { dao: self }
    }
}
pub struct AccountAddressCache<'t> {
    pub dao: &'t AccountAddress,
}
impl AccountAddressCache<'_> {
    pub async fn find_by_account_id_vec(&self, id: &u64) -> AccountResult<Vec<AccountAddressModel>> {
        self.dao
            .cache
            .get_or_fetch(id, || self.dao.find_by_account_id_vec(id))
            .await
    }
    pub async fn find_by_account_ids_vec(&self, ids: &[u64]) -> AccountResult<HashMap<u64, Vec<AccountAddressModel>>> {
        self.dao
            .cache
            .get_or_fetch_many(ids, |missing| async move {
                self.dao.find_by_account_ids_vec(&missing).await
            })
            .await
    }
}
