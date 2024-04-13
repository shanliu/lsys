mod admin;
mod app_oauth;
mod app_rbac;
mod app_sender;
mod app_sender_mail;
mod app_sender_sms;
mod barcode;
mod rbac;
mod relation;
mod setting;
mod user;
pub use admin::*;
pub use app_oauth::*;
pub use app_rbac::*;
pub use app_sender::*;
pub use app_sender_mail::*;
pub use app_sender_sms::*;
pub use barcode::*;
use lsys_rbac::{
    access_relation_tpl, access_res_tpl,
    dao::{RelationTpl, ResTpl},
};
pub use rbac::*;
pub use relation::*;
pub use setting::*;
pub use user::*;

pub fn relation_tpls() -> Vec<RelationTpl> {
    access_relation_tpl!(RelationApp)
}

pub fn res_tpls() -> Vec<ResTpl> {
    access_res_tpl!(
        AccessAdminMailConfig,
        AccessAppSenderMailConfig,
        AccessAppSenderMailMsg,
        AccessAppSenderDoMail,
        AccessAdminManage,
        AccessAdminChangeLogsView,
        AccessAdminDocsEdit,
        AccessAdminSetting,
        AccessAdminUserFull,
        AccessAdminUserBase,
        AccessAppSenderDoSms,
        AccessSubAppRbacCheck,
        AccessOauthUserInfo,
        AccessOauthUserEmail,
        AccessOauthUserMobile,
        AccessOauthUserAddress,
        AccessAdminSmsConfig,
        AccessAppSenderSmsConfig,
        AccessAppSenderSmsMsg,
        AccessResView,
        AccessResEdit,
        AccessRoleView,
        AccessRoleEdit,
        AccessRoleViewList,
        AccessUserAppConfirm,
        AccessUserMobileEdit,
        AccessUserMobileView,
        AccessUserAppEdit,
        AccessUserAppView,
        AccessUserEmailEdit,
        AccessUserEmailView,
        AccessUserInfoEdit,
        AccessUserNameEdit,
        AccessUserAddressEdit,
        AccessUserAddressView,
        AccessSystemLogin,
        AccessSystemEmailConfirm,
        AccessSystemMobileConfirm,
        AccessSystemReSetPassword,
        AccessUserExternalEdit,
        AccessUserSetPassword,
        AccessAdminSenderTplView,
        AccessAdminSenderTplEdit,
        AccessSiteSetting,
        AccessBarCodeView,
        AccessBarCodeEdit
    )
}
