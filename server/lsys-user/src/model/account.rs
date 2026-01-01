use crate::dao::{AccountError, AccountResult};

use super::{
    AccountAddressModel, AccountAddressStatus, AccountEmailModel, AccountEmailStatus,
    AccountExternalModel, AccountExternalStatus, AccountMobileModel, AccountMobileStatus,
    AccountModel, AccountStatus,
};

macro_rules! model_enable_method {
    ($status:expr,$id_field:ident,$status_field:ident,$name:literal) => {
        //判断是否正常启用
        pub fn is_enable(&self) -> AccountResult<()> {
            if !$status.eq(self.$status_field) {
                return Err(AccountError::Status((self.$id_field,lsys_core::fluent_message!("user-status-invalid",{
                        "user":$name,
                        "status":self.$status_field
                    }
                ))));
                // "{} status invalid:[{}]",
                // $name, self.$status_field
            }
            Ok(())
        }
    };
}
impl AccountModel {
    model_enable_method!(AccountStatus::Enable, id, status, "user");
    pub fn show_name(&self) -> String {
        format!("{}-{}", self.nickname, self.id)
    }
}
impl AccountEmailModel {
    model_enable_method!(AccountEmailStatus::Valid, id, status, "user email");
}
impl AccountMobileModel {
    model_enable_method!(AccountMobileStatus::Valid, id, status, "user mobile");
}
impl AccountAddressModel {
    model_enable_method!(AccountAddressStatus::Enable, id, status, "user address");
}
impl AccountExternalModel {
    model_enable_method!(AccountExternalStatus::Enable, id, status, "user external");
}
