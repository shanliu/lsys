use lsys_user::{
    dao::account::UserAccountResult,
    model::{
        UserAddressModel, UserEmailModel, UserEmailStatus, UserExternalModel, UserInfoModel,
        UserMobileModel, UserMobileStatus, UserModel, UserNameModel,
    },
};

use super::WebUser;

macro_rules! list_data_status_filter {
    ($vec_data:expr,$use_status:expr,$def_status:expr) => {
        $vec_data
            .into_iter()
            .filter(|e| {
                let _status = &e.status.try_into().unwrap_or_else(|e| {
                    tracing::error!("user data into fail:{}", e);
                    $def_status
                });
                return $use_status.contains(_status);
            })
            .collect::<_>()
    };
}

#[derive(Default)]
pub struct UserDataOption<'a> {
    pub user: bool,
    pub name: bool,
    pub info: bool,
    pub address: bool,
    pub external: Option<&'a [String]>,
    pub email: Option<&'a [UserEmailStatus]>,
    pub mobile: Option<&'a [UserMobileStatus]>,
}

impl WebUser {
    pub async fn user_address(&self, id: u64) -> UserAccountResult<Vec<UserAddressModel>> {
        self.user_dao
            .user_account
            .user_address
            .cache()
            .find_by_user_id_vec(&id)
            .await
    }

    pub async fn user_mobile(
        &self,
        id: u64,
        status: Option<&[UserMobileStatus]>,
    ) -> UserAccountResult<Vec<UserMobileModel>> {
        let mut tmp = self
            .user_dao
            .user_account
            .user_mobile
            .cache()
            .find_by_user_id_vec(&id)
            .await?;
        if let Some(mut st) = status {
            if st.is_empty() {
                st = &[UserMobileStatus::Init, UserMobileStatus::Valid];
            }
            tmp = list_data_status_filter!(tmp, st, UserMobileStatus::Init)
        }
        Ok(tmp)
    }

    pub async fn user_email(
        &self,
        id: u64,
        status: Option<&[UserEmailStatus]>,
    ) -> UserAccountResult<Vec<UserEmailModel>> {
        let mut tmp = self
            .user_dao
            .user_account
            .user_email
            .cache()
            .find_by_user_id_vec(&id)
            .await?;
        if let Some(mut st) = status {
            if st.is_empty() {
                st = &[UserEmailStatus::Init, UserEmailStatus::Valid];
            }
            tmp = list_data_status_filter!(tmp, st, UserEmailStatus::Init)
        }
        Ok(tmp)
    }
    pub async fn user_external(
        &self,
        id: u64,
        oauth_type: Option<&[String]>,
    ) -> UserAccountResult<Vec<UserExternalModel>> {
        let mut res = self
            .user_dao
            .user_account
            .user_external
            .cache()
            .find_by_user_id_vec(&id)
            .await?;
        if let Some(st) = oauth_type {
            if !st.is_empty() {
                res = res
                    .into_iter()
                    .filter(|e| st.contains(&e.external_type))
                    .collect::<_>();
            }
        }
        Ok(res)
    }

    //指定用户信息
    pub async fn user_detail<'t>(
        &self,
        id: u64,
        data_option: UserDataOption<'t>,
    ) -> UserAccountResult<(
        Option<UserModel>,
        Option<UserNameModel>,
        Option<UserInfoModel>,
        Option<Vec<UserAddressModel>>,
        Option<Vec<UserEmailModel>>,
        Option<Vec<UserExternalModel>>,
        Option<Vec<UserMobileModel>>,
    )> {
        let mut out_user = None;
        if data_option.user {
            out_user = Some(
                self.user_dao
                    .user_account
                    .user
                    .cache()
                    .find_by_id(&id)
                    .await?,
            );
        }
        let mut out_user_name = None;
        if data_option.name {
            let user_name = self
                .user_dao
                .user_account
                .user_name
                .cache()
                .find_by_user_id(&id)
                .await;
            match user_name {
                Ok(user_data) => {
                    out_user_name = Some(user_data);
                }
                Err(err) => {
                    if !err.is_not_found() {
                        return Err(err);
                    }
                }
            }
        }
        let mut out_user_info = None;
        if data_option.info {
            let user_info = self
                .user_dao
                .user_account
                .user_info
                .cache()
                .find_by_user_id(&id)
                .await;
            match user_info {
                Ok(user_data) => {
                    out_user_info = Some(user_data);
                }
                Err(err) => {
                    if !err.is_not_found() {
                        return Err(err);
                    }
                }
            }
        }

        let mut out_user_address = None;
        if data_option.address {
            out_user_address = Some(self.user_address(id).await?);
        }
        let mut out_user_email = None;
        if let Some(status) = data_option.email {
            out_user_email = Some(self.user_email(id, Some(status)).await?);
        }
        let mut out_user_external = None;
        if let Some(otype) = data_option.external {
            out_user_external = Some(self.user_external(id, Some(otype)).await?);
        }
        let mut out_user_mobile = None;
        if let Some(status) = data_option.mobile {
            out_user_mobile = Some(self.user_mobile(id, Some(status)).await?);
        }
        Ok((
            out_user,
            out_user_name,
            out_user_info,
            out_user_address,
            out_user_email,
            out_user_external,
            out_user_mobile,
        ))
    }
}
