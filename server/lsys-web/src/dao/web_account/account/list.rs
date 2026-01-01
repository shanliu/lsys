use lsys_user::model::{
    AccountAddressModel, AccountEmailModel, AccountEmailStatus, AccountExternalModel,
    AccountInfoModel, AccountMobileModel, AccountMobileStatus, AccountModel, AccountNameModel,
};

use crate::common::JsonResult;

use super::{AccountOptionData, WebUserAccount};
use std::collections::HashMap;

macro_rules! list_data_map_status_filter {
    ($vec_data:expr,$use_status:expr,$type:ty,$def_status:expr) => {
        $vec_data
            .into_iter()
            .map(|(i, data)| {
                (
                    i,
                    data.into_iter()
                        .filter(|e| {
                            let _status = &e.status.try_into().unwrap_or_else(|e| {
                                tracing::error!("user data into fail:{}", e);
                                $def_status
                            });
                            return $use_status.contains(_status);
                        })
                        .collect::<Vec<$type>>(),
                )
            })
            .collect::<HashMap<u64, Vec<$type>>>()
    };
}

impl WebUserAccount {
    /// 用户列表
    pub async fn list_user(
        &self,
        ids: &[u64],
        data_option: &AccountOptionData<'_>,
    ) -> JsonResult<
        HashMap<
            u64,
            (
                Option<AccountModel>,
                Option<AccountNameModel>,
                Option<AccountInfoModel>,
                Option<Vec<AccountAddressModel>>,
                Option<Vec<AccountEmailModel>>,
                Option<Vec<AccountExternalModel>>,
                Option<Vec<AccountMobileModel>>,
            ),
        >,
    > {
        let mut out_user = None;
        if data_option.user {
            out_user = Some(
                self.user_dao
                    .account_dao
                    .account
                    .cache()
                    .find_by_ids(ids)
                    .await?,
            );
        }
        let mut out_user_name = None;
        if data_option.name {
            out_user_name = Some(
                self.user_dao
                    .account_dao
                    .account_name
                    .cache()
                    .find_by_account_ids(ids)
                    .await?,
            );
        }
        let mut out_user_info = None;
        if data_option.info {
            out_user_info = Some(
                self.user_dao
                    .account_dao
                    .account_info
                    .cache()
                    .find_by_account_ids(ids)
                    .await?,
            );
        }

        let mut out_user_address = None;
        if data_option.address {
            out_user_address = Some(
                self.user_dao
                    .account_dao
                    .account_address
                    .cache()
                    .find_by_account_ids_vec(ids)
                    .await?,
            );
        }
        let mut out_user_email = None;
        if let Some(mut status) = data_option.email {
            if status.is_empty() {
                status = &[AccountEmailStatus::Init, AccountEmailStatus::Valid];
            }
            let tmp = self
                .user_dao
                .account_dao
                .account_email
                .cache()
                .find_by_account_ids_vec(ids)
                .await?;

            out_user_email = Some(list_data_map_status_filter!(
                tmp,
                status,
                AccountEmailModel,
                AccountEmailStatus::Init
            ));
        }
        let mut out_user_external = None;
        if let Some(exttype) = data_option.external {
            let tmp = self
                .user_dao
                .account_dao
                .account_external
                .cache()
                .find_by_account_ids_vec(ids)
                .await?;
            out_user_external = Some(if exttype.is_empty() {
                tmp
            } else {
                tmp.into_iter()
                    .map(|(i, data)| {
                        (
                            i,
                            data.into_iter()
                                .filter(|e| exttype.contains(&e.external_type.as_str()))
                                .collect::<Vec<AccountExternalModel>>(),
                        )
                    })
                    .collect::<HashMap<u64, Vec<AccountExternalModel>>>()
            });
        }
        let mut out_user_mobile = None;
        if let Some(mut status) = data_option.mobile {
            if status.is_empty() {
                status = &[AccountMobileStatus::Init, AccountMobileStatus::Valid];
            }
            let tmp = self
                .user_dao
                .account_dao
                .account_mobile
                .cache()
                .find_by_account_ids_vec(ids)
                .await?;
            out_user_mobile = Some(list_data_map_status_filter!(
                tmp,
                status,
                AccountMobileModel,
                AccountMobileStatus::Init
            ));
        }

        let mut out = HashMap::with_capacity(ids.len());
        for uid in ids {
            let mut user_ = None;
            if let Some(ref data) = out_user {
                for (u, tmp) in data {
                    if *u == *uid {
                        user_ = Some(tmp.to_owned());
                        break;
                    }
                }
            }
            let mut name_ = None;
            if let Some(ref data) = out_user_name {
                for (u, tmp) in data {
                    if *u == *uid {
                        name_ = Some(tmp.to_owned());
                        break;
                    }
                }
            }
            let mut info_ = None;
            if let Some(ref data) = out_user_info {
                for (u, tmp) in data {
                    if *u == *uid {
                        info_ = Some(tmp.to_owned());
                        break;
                    }
                }
            }
            let mut address_ = None;
            if let Some(ref data) = out_user_address {
                for (u, tmp) in data {
                    if *u == *uid {
                        address_ = Some(tmp.to_owned());
                        break;
                    }
                }
            }
            let mut email_ = None;
            if let Some(ref data) = out_user_email {
                for (u, tmp) in data {
                    if *u == *uid {
                        email_ = Some(tmp.to_owned());
                        break;
                    }
                }
            }
            let mut external_ = None;
            if let Some(ref data) = out_user_external {
                for (u, tmp) in data {
                    if *u == *uid {
                        external_ = Some(tmp.to_owned());
                        break;
                    }
                }
            }
            let mut mobile_ = None;
            if let Some(ref data) = out_user_mobile {
                for (u, tmp) in data {
                    if *u == *uid {
                        mobile_ = Some(tmp.to_owned());
                        break;
                    }
                }
            }
            out.entry(*uid)
                .or_insert((user_, name_, info_, address_, email_, external_, mobile_));
        }
        Ok(out)
    }
}
