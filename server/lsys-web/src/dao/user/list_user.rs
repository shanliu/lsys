use lsys_user::{
    dao::account::UserAccountResult,
    model::{
        UserAddressModel, UserEmailModel, UserEmailStatus, UserExternalModel, UserInfoModel,
        UserMobileModel, UserMobileStatus, UserModel, UserNameModel,
    },
};

use std::collections::HashMap;

use super::{UserDataOption, WebUser};

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

impl WebUser {
    /// 用户列表
    pub async fn list_user<'t>(
        &self,
        ids: &[u64],
        data_option: &UserDataOption<'t>,
    ) -> UserAccountResult<
        HashMap<
            u64,
            (
                Option<UserModel>,
                Option<UserNameModel>,
                Option<UserInfoModel>,
                Option<Vec<UserAddressModel>>,
                Option<Vec<UserEmailModel>>,
                Option<Vec<UserExternalModel>>,
                Option<Vec<UserMobileModel>>,
            ),
        >,
    > {
        let mut out_user = None;
        if data_option.user {
            out_user = Some(
                self.user_dao
                    .user_account
                    .user
                    .cache()
                    .find_by_ids(ids)
                    .await?,
            );
        }
        let mut out_user_name = None;
        if data_option.name {
            out_user_name = Some(
                self.user_dao
                    .user_account
                    .user_name
                    .cache()
                    .find_by_user_ids(ids)
                    .await?,
            );
        }
        let mut out_user_info = None;
        if data_option.info {
            out_user_info = Some(
                self.user_dao
                    .user_account
                    .user_info
                    .cache()
                    .find_by_user_ids(ids)
                    .await?,
            );
        }

        let mut out_user_address = None;
        if data_option.address {
            out_user_address = Some(
                self.user_dao
                    .user_account
                    .user_address
                    .cache()
                    .find_by_user_ids_vec(ids)
                    .await?,
            );
        }
        let mut out_user_email = None;
        if let Some(mut status) = data_option.email {
            if status.is_empty() {
                status = &[UserEmailStatus::Init, UserEmailStatus::Valid];
            }
            let tmp = self
                .user_dao
                .user_account
                .user_email
                .cache()
                .find_by_user_ids_vec(ids)
                .await?;
            out_user_email = Some(list_data_map_status_filter!(
                tmp,
                status,
                UserEmailModel,
                UserEmailStatus::Init
            ));
        }
        let mut out_user_external = None;
        if let Some(exttype) = data_option.external {
            let tmp = self
                .user_dao
                .user_account
                .user_external
                .cache()
                .find_by_user_ids_vec(ids)
                .await?;
            out_user_external = Some(if exttype.is_empty() {
                tmp
            } else {
                tmp.into_iter()
                    .map(|(i, data)| {
                        (
                            i,
                            data.into_iter()
                                .filter(|e| exttype.contains(&e.external_type))
                                .collect::<Vec<UserExternalModel>>(),
                        )
                    })
                    .collect::<HashMap<u64, Vec<UserExternalModel>>>()
            });
        }
        let mut out_user_mobile = None;
        if let Some(mut status) = data_option.mobile {
            if status.is_empty() {
                status = &[UserMobileStatus::Init, UserMobileStatus::Valid];
            }
            let tmp = self
                .user_dao
                .user_account
                .user_mobile
                .cache()
                .find_by_user_ids_vec(ids)
                .await?;
            out_user_mobile = Some(list_data_map_status_filter!(
                tmp,
                status,
                UserMobileModel,
                UserMobileStatus::Init
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
