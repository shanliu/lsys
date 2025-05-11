use crate::common::{JsonData, JsonError, JsonResult};
use lsys_access::dao::SessionBody;
use lsys_core::{fluent_message, RequestEnv};
use lsys_user::dao::AccountAddressParam;
use lsys_user::model::AccountAddressModel;

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

        let id = self
            .user_dao
            .account_dao
            .account_address
            .add_address(
                &account,
                &AccountAddressParam {
                    country_code: "CHN",
                    address_code: param.code,
                    address_info: param.info,
                    address_detail: param.detail,
                    name: param.name,
                    mobile: param.mobile,
                },
                session_body.user_id(),
                None,
                env_data,
            )
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

        self.user_dao
            .account_dao
            .account_address
            .edit_address(
                address,
                &AccountAddressParam {
                    country_code: "CHN",
                    address_code: param.code,
                    address_info: param.info,
                    address_detail: param.detail,
                    name: param.name,
                    mobile: param.mobile,
                },
                session_body.user_id(),
                None,
                env_data,
            )
            .await?;
        Ok(())
    }
}
