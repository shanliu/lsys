use lsys_user::model::{
    AccountAddressModel, AccountEmailModel, AccountEmailStatus, AccountExternalModel,
    AccountInfoModel, AccountMobileModel, AccountMobileStatus, AccountModel, AccountNameModel,
};

use crate::common::JsonResult;

use super::WebUserAccount;
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

impl WebUserAccount {
    //用户地址列表
    pub async fn user_address(&self, id: u64) -> JsonResult<Vec<AccountAddressModel>> {
        Ok(self
            .user_dao
            .account_dao
            .account_address
            .cache()
            .find_by_account_id_vec(&id)
            .await?)
    }
    //用户手机列表
    pub async fn user_mobile(
        &self,
        id: u64,
        status: Option<&[AccountMobileStatus]>,
    ) -> JsonResult<Vec<AccountMobileModel>> {
        let mut tmp = self
            .user_dao
            .account_dao
            .account_mobile
            .cache()
            .find_by_account_id_vec(id)
            .await?;
        if let Some(mut st) = status {
            if st.is_empty() {
                st = &[AccountMobileStatus::Init, AccountMobileStatus::Valid];
            }
            tmp = list_data_status_filter!(tmp, st, AccountMobileStatus::Init)
        }
        Ok(tmp)
    }
    //用户邮箱列表
    pub async fn user_email(
        &self,
        id: u64,
        status: Option<&[AccountEmailStatus]>,
    ) -> JsonResult<Vec<AccountEmailModel>> {
        let mut tmp = self
            .user_dao
            .account_dao
            .account_email
            .cache()
            .find_by_account_id_vec(id)
            .await?;
        if let Some(mut st) = status {
            if st.is_empty() {
                st = &[AccountEmailStatus::Init, AccountEmailStatus::Valid];
            }
            tmp = list_data_status_filter!(tmp, st, AccountEmailStatus::Init)
        }
        Ok(tmp)
    }
    pub async fn user_external(
        &self,
        id: u64,
        oauth_type: Option<&[&str]>,
    ) -> JsonResult<Vec<AccountExternalModel>> {
        let mut res = self
            .user_dao
            .account_dao
            .account_external
            .cache()
            .find_by_account_id_vec(id)
            .await?;
        if let Some(st) = oauth_type {
            if !st.is_empty() {
                res = res
                    .into_iter()
                    .filter(|e| st.contains(&e.external_type.as_str()))
                    .collect::<_>();
            }
        }
        Ok(res)
    }
}

#[derive(Default)]
pub struct AccountOptionData<'a> {
    pub user: bool,
    pub name: bool,
    pub info: bool,
    pub address: bool,
    pub external: Option<&'a [&'a str]>,
    pub email: Option<&'a [AccountEmailStatus]>,
    pub mobile: Option<&'a [AccountMobileStatus]>,
}
impl WebUserAccount {
    //指定用户信息
    pub async fn user_detail(
        &self,
        id: u64,
        data_option: &AccountOptionData<'_>,
    ) -> JsonResult<(
        Option<AccountModel>,
        Option<AccountNameModel>,
        Option<AccountInfoModel>,
        Option<Vec<AccountAddressModel>>,
        Option<Vec<AccountEmailModel>>,
        Option<Vec<AccountExternalModel>>,
        Option<Vec<AccountMobileModel>>,
    )> {
        let mut out_user = None;
        if data_option.user {
            out_user = Some(
                self.user_dao
                    .account_dao
                    .account
                    .cache()
                    .find_by_id(&id)
                    .await?,
            );
        }
        let mut out_user_name = None;
        if data_option.name {
            let user_name = self
                .user_dao
                .account_dao
                .account_name
                .cache()
                .find_by_account_id(&id)
                .await;
            match user_name {
                Ok(user_data) => {
                    out_user_name = Some(user_data);
                }
                Err(err) => {
                    if !err.is_not_found() {
                        return Err(err.into());
                    }
                }
            }
        }
        let mut out_user_info = None;
        if data_option.info {
            let user_info = self
                .user_dao
                .account_dao
                .account_info
                .cache()
                .find_by_account_id(&id)
                .await;
            match user_info {
                Ok(user_data) => {
                    out_user_info = Some(user_data);
                }
                Err(err) => {
                    if !err.is_not_found() {
                        return Err(err.into());
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
