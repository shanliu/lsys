use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    get_message, now_time, FluentMessage,
};

use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{sql_format, Insert, ModelTableName, Select, SqlQuote, Update};
use std::{collections::HashMap, sync::Arc};

use crate::model::{UserAddressModel, UserAddressModelRef, UserAddressStatus, UserModel};

use super::{check_mobile, user_index::UserIndex, UserAccountError, UserAccountResult};

pub struct UserAddress {
    db: Pool<MySql>,
    fluent: Arc<FluentMessage>,
    index: Arc<UserIndex>,
    pub cache: Arc<LocalCache<u64, Vec<UserAddressModel>>>,
}

impl UserAddress {
    pub fn new(
        db: Pool<MySql>,
        fluent: Arc<FluentMessage>,
        redis: deadpool_redis::Pool,
        index: Arc<UserIndex>,
    ) -> Self {
        Self {
            cache: Arc::from(LocalCache::new(
                redis,
                LocalCacheConfig::new("user-address"),
            )),
            db,
            fluent,
            index,
        }
    }
    /// 添加用户地址
    pub async fn add_address<'t, 't2>(
        &self,
        user: &UserModel,
        mut address: UserAddressModelRef<'t2>,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        macro_rules! check_data {
            ($addr_var:ident,$name:literal) => {
                let $addr_var = if let Some($addr_var) = address.$addr_var {
                    $addr_var.trim().to_owned()
                } else {
                    "".to_string()
                };
                if $addr_var.is_empty() {
                    return Err(UserAccountError::System(get_message!(
                        &self.fluent,
                        concat!("user-address-", $name, "-empty"),
                        concat!("address ", $name, " can't be nul")
                    )));
                }
                address.$addr_var = Some(&$addr_var);
            };
        }
        check_data!(address_info, "info");
        check_data!(address_code, "code");
        check_data!(address_detail, "detail");
        check_data!(name, "name");
        check_data!(mobile, "mobile");
        check_mobile(&self.fluent, "", &mobile)?;

        let address_res = Select::type_new::<UserAddressModel>()
        .fetch_one_by_where_call::<UserAddressModel, _, _>(
            " user_id=? and address_code=? and address_info=? and address_detail=? and name=? and mobile=? and status=?",
            |mut res, _| {
                res = res.bind(user.id);
                res = res.bind(address_code.clone());
                res = res.bind(address_info.clone());
                res = res.bind(address_detail.clone());
                res = res.bind(name.clone());
                res = res.bind(mobile.clone());
                res = res.bind(UserAddressStatus::Enable as i8);
                res
            },
            &self.db,
        )
        .await;
        if let Ok(address) = address_res {
            return Ok(address.id);
        }

        let time = now_time()?;
        address.add_time = Some(&time);
        address.status = Some(&(UserAddressStatus::Enable as i8));
        address.user_id = Some(&user.id);

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let res = Insert::<sqlx::MySql, UserAddressModel, _>::new(address)
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
                                &[address_info],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }

                        db.commit().await?;
                        self.cache.clear(&user.id).await;
                        Ok(mr.last_insert_id())
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
    ) -> UserAccountResult<u64> {
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(UserAddressModelRef,{
            status:UserAddressStatus::Delete as i8,
            delete_time:time,
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
