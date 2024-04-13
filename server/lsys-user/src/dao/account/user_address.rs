use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    fluent_message, now_time, RemoteNotify, RequestEnv,
};

use lsys_logger::dao::ChangeLogger;
use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{sql_format, Insert, ModelTableName, Select, SqlQuote, Update, WhereOption};
use std::{collections::HashMap, sync::Arc};

use crate::model::{UserAddressModel, UserAddressModelRef, UserAddressStatus, UserModel};

use super::{
    check_mobile, logger::LogUserAddress, user_index::UserIndex, UserAccountError,
    UserAccountResult,
};

pub struct UserAddress {
    db: Pool<MySql>,
    // fluent: Arc<FluentBuild>,
    index: Arc<UserIndex>,
    pub(crate) cache: Arc<LocalCache<u64, Vec<UserAddressModel>>>,
    logger: Arc<ChangeLogger>,
}

impl UserAddress {
    pub fn new(
        db: Pool<MySql>,
        // fluent: Arc<FluentBuild>,
       
        index: Arc<UserIndex>,
        remote_notify: Arc<RemoteNotify>,
        config:LocalCacheConfig,
        logger: Arc<ChangeLogger>,
    ) -> Self {
        Self {
            cache:Arc::new(LocalCache::new(remote_notify, config)),
            db,
            //  fluent,
            index,
            logger,
        }
    }

    /// 添加用户地址
    pub async fn edit_address<'t, 't2>(
        &self,
        address: &UserAddressModel,
        user: &UserModel,
        mut address_data: UserAddressModelRef<'t2>,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<()> {
        if !UserAddressStatus::Enable.eq(address.status) {
            return Err(UserAccountError::System(
                fluent_message!("user-address-is-delete",
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
                    return Err(UserAccountError::System(fluent_message!("user-address-not-empty",
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
        address_data.user_id = Some(&address.user_id); //防止被外面给改了
        address_data.id = None; //防止被外面给改了

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::<sqlx::MySql, UserAddressModel, _>::new(address_data)
            .execute_by_pk(address, &mut db)
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
                        crate::model::UserIndexCat::Address,
                        user.id,
                        &[address.address_info.clone()],
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
                        crate::model::UserIndexCat::Address,
                        user.id,
                        &[address_info.clone()],
                        Some(&mut db),
                    )
                    .await
                {
                    db.rollback().await?;
                    return Err(ie);
                }
                db.commit().await?;
                self.cache.clear(&user.id).await;

                self.logger
                    .add(
                        &LogUserAddress {
                            action: "edit",
                            address_code,
                            address_info,
                            address_detail,
                            name,
                            mobile,
                        },
                        &Some(address.id),
                        &Some(address.user_id),
                        &Some(user.id),
                        None,
                        env_data,
                    )
                    .await;

                Ok(())
            }
        }
    }
    /// 添加用户地址
    pub async fn add_address<'t, 't2>(
        &self,
        user: &UserModel,
        mut address_data: UserAddressModelRef<'t2>,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<u64> {
        macro_rules! check_data {
            ($addr_var:ident,$name:literal) => {
                let $addr_var = if let Some($addr_var) = address_data.$addr_var {
                    $addr_var.trim().to_owned()
                } else {
                    "".to_string()
                };
                if $addr_var.is_empty() {
                    return Err(UserAccountError::System(fluent_message!("user-address-not-empty",
                        {"name":$name}
                    )));
                    // concat!("user-address-", $name, "-empty"),
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

        let address_res = Select::type_new::<UserAddressModel>()
        .fetch_one_by_where::<UserAddressModel, _>(
            &WhereOption::Where(sql_format!(
            " user_id={} and address_code={} and address_info={} and address_detail={} and name={} and mobile={} and status={}",
            user.id,
                address_code,
                address_info,
                address_detail,
                name,
                mobile,
                UserAddressStatus::Enable)),
            &self.db,
        )
        .await;
        if let Ok(address) = address_res {
            return Ok(address.id);
        }

        let time = now_time()?;
        address_data.change_time = Some(&time);
        address_data.status = Some(&(UserAddressStatus::Enable as i8));

        if address_data.user_id.is_none() {
            address_data.user_id = Some(&user.id);
        }

        let address_user_id = *address_data.user_id.unwrap_or(&user.id);

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let res = Insert::<sqlx::MySql, UserAddressModel, _>::new(address_data)
            .execute(&mut db)
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
                        UserModel::table_name(),
                    )
                    .as_str(),
                )
                .bind(user.id)
                .execute(&mut db)
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
                                crate::model::UserIndexCat::Address,
                                user.id,
                                &[address_info.clone()],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }

                        db.commit().await?;
                        self.cache.clear(&user.id).await;

                        let aid = mr.last_insert_id();

                        self.logger
                            .add(
                                &LogUserAddress {
                                    action: "add",
                                    address_code,
                                    address_info,
                                    address_detail,
                                    name,
                                    mobile,
                                },
                                &Some(aid),
                                &Some(address_user_id),
                                &Some(user.id),
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
    pub async fn del_address<'t, 't2>(
        &self,
        address: &UserAddressModel,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<u64> {
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(UserAddressModelRef,{
            status:UserAddressStatus::Delete as i8,
            change_time:time,
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<sqlx::MySql, UserAddressModel, _>::new(change)
            .execute_by_pk(address, &mut db)
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
                    UserModel::table_name(),
                )
                    .as_str(),
                )
                .bind(address.user_id)
                .execute(&mut db)
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
                                crate::model::UserIndexCat::Address,
                                address.user_id,
                                &[address.address_info.to_owned()],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }

                        db.commit().await?;
                        self.cache.clear(&address.user_id).await;

                        self.logger
                            .add(
                                &LogUserAddress {
                                    action: "del",
                                    address_code: address.address_code.to_owned(),
                                    address_info: address.address_info.to_owned(),
                                    address_detail: address.address_detail.to_owned(),
                                    name: address.name.to_owned(),
                                    mobile: address.mobile.to_owned(),
                                },
                                &Some(address.id),
                                &Some(address.user_id),
                                &Some(address.user_id),
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
        UserAddressModel,
        UserAccountResult<UserAddressModel>,
        id,
        "id={id}"
    );
    lsys_core::impl_dao_fetch_vec_by_one!(
        db,
        find_by_user_id_vec,
        u64,
        UserAddressModel,
        UserAccountResult<Vec<UserAddressModel>>,
        uid,
        "user_id = {uid} and status = {status}",
        status = UserAddressStatus::Enable
    );
    lsys_core::impl_dao_fetch_vec_by_vec!(
        db,
        find_by_user_ids_vec,
        u64,
        UserAddressModel,
        UserAccountResult<HashMap<u64, Vec<UserAddressModel>>>,
        user_id,
        uid,
        "user_id in ({uid}) and status = {status}",
        status = UserAddressStatus::Enable
    );
    pub fn cache(&'_ self) -> UserAddressCache<'_> {
        UserAddressCache { dao: self }
    }
}
pub struct UserAddressCache<'t> {
    pub dao: &'t UserAddress,
}
impl<'t> UserAddressCache<'t> {
    lsys_core::impl_cache_fetch_one!(
        find_by_user_id_vec,
        dao,
        cache,
        u64,
        UserAccountResult<Vec<UserAddressModel>>
    );
    lsys_core::impl_cache_fetch_vec!(
        find_by_user_ids_vec,
        dao,
        cache,
        u64,
        UserAccountResult<HashMap<u64, Vec<UserAddressModel>>>
    );
}
