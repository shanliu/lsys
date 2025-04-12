use crate::common::{JsonData, JsonError, JsonResult};

use lsys_access::dao::SessionBody;
use lsys_core::model_option_set;
use lsys_core::{fluent_message, RequestEnv};
use lsys_user::model::{AccountAddressModel, AccountAddressModelRef};

use super::WebUserAccount;

pub struct AddressData<'t> {
    pub code: &'t str,
    pub info: &'t str,
    pub detail: &'t str,
    pub name: &'t str,
    pub mobile: &'t str,
}
impl WebUserAccount {
    //添加用户地址
    pub async fn user_address_add(
        &self,
        param: &AddressData<'_>,
        session_body: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<u64> {
        let account = self
            .user_dao
            .account_dao
            .session_account(session_body)
            .await?;

        if param.code.trim().len() < 6 {
            return Err(JsonError::JsonResponse(
                JsonData::default().set_code(500).set_sub_code("bad_code"),
                fluent_message!("address-miss-city"), // JsonResponse::message("your submit area miss city").set_code("bad_code")
            ));
        }
        let area = self.area.code_related(param.code)?;
        if area.is_empty() {
            return Err(JsonError::JsonResponse(
                JsonData::default().set_code(500).set_sub_code("bad_code"),
                fluent_message!("address-bad-area"), // JsonResponse::message("your submit area miss city").set_code("bad_code")
            ));
        }
        let country_code = "CHN".to_string();
        let address_code = param.code.to_string();
        let address_info = param.info.to_string();
        let address_detail = param.detail.to_string();
        let name = param.name.to_string();
        let mobile = param.mobile.to_string();
        let adm = model_option_set!(AccountAddressModelRef, {
            country_code:country_code,
            address_code: address_code,
            address_info: address_info,
            address_detail: address_detail,
            name: name,
            mobile: mobile,
            account_id:account.id,
        });
        let id = self
            .user_dao
            .account_dao
            .account_address
            .add_address(&account, adm, session_body.user_id(), None, env_data)
            .await?;
        Ok(id)
    }
    //编辑用户地址
    pub async fn user_address_edit(
        &self,
        address: &AccountAddressModel,
        param: &AddressData<'_>,
        session_body: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<()> {
        let country_code = "CHN".to_string();
        if param.code.trim().len() < 6 {
            return Err(JsonError::JsonResponse(
                JsonData::default().set_code(500).set_sub_code("bad_code"),
                fluent_message!("address-miss-city"),
            ));
        }
        let area = self.area.code_find(param.code)?;
        if area.is_empty() {
            return Err(JsonError::JsonResponse(
                JsonData::default().set_code(500).set_sub_code("bad_code"),
                fluent_message!("address-bad-area"),
            ));
        }
        let address_code = param.code.to_string();
        let address_info = param.info.to_string();
        let address_detail = param.detail.to_string();
        let name = param.name.to_string();
        let mobile = param.mobile.to_string();
        let adm = model_option_set!(AccountAddressModelRef, {
            country_code:country_code,
            address_code: address_code,
            address_info: address_info,
            address_detail: address_detail,
            name: name,
            mobile: mobile,
        });
        self.user_dao
            .account_dao
            .account_address
            .edit_address(address, adm, session_body.user_id(), None, env_data)
            .await?;
        Ok(())
    }
}
