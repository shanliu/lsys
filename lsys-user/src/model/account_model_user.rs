use crate::dao::account::{UserAccountError, UserAccountResult};

use super::{
    UserAddressModel, UserAddressStatus, UserEmailModel, UserEmailStatus, UserExternalModel,
    UserExternalStatus, UserMobileModel, UserMobileStatus, UserModel, UserStatus,
};

macro_rules! model_enable_method {
    ($status:expr,$id_field:ident,$status_field:ident,$name:literal) => {
        //判断是否正常启用
        pub fn is_enable(&self) -> UserAccountResult<()> {
            if !$status.eq(self.$status_field) {
                return Err(UserAccountError::Status((self.$id_field,format!(
                    "{} status invalid:[{}]",
                    $name, self.$status_field
                ))));
            }
            Ok(())
        }
    };
}
impl UserModel {
    model_enable_method!(UserStatus::Enable,id, status, "user");
    pub fn show_name(&self) -> String {
        format!("{}-{}", self.nickname, self.id)
    }
}
impl UserEmailModel {
    model_enable_method!(UserEmailStatus::Valid, id,status, "user email");
}
impl UserMobileModel {
    model_enable_method!(UserMobileStatus::Valid, id,status, "user mobile");
}
impl UserAddressModel {
    model_enable_method!(UserAddressStatus::Enable, id,status, "user address");
}
impl UserExternalModel {
    model_enable_method!(UserExternalStatus::Enable, id,status, "user external");
}
