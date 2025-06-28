use crate::dao::access::api::system::admin::{
    CheckAdminApp, CheckAdminBase, CheckAdminChangeLogsView, CheckAdminDocs, CheckAdminMailConfig,
    CheckAdminMailMgr, CheckAdminRbacEdit, CheckAdminRbacView, CheckAdminSiteSetting,
    CheckAdminSmsConfig, CheckAdminSmsMgr, CheckAdminUserManage,
};
use crate::dao::access::api::system::auth::{CheckSystemLogin, CheckSystemRegister};
use crate::dao::access::api::system::user::{
    CheckUserAddressEdit, CheckUserAppEdit, CheckUserAppSenderMailConfig,
    CheckUserAppSenderMailSend, CheckUserAppSenderMailView, CheckUserAppSenderSmsConfig,
    CheckUserAppSenderSmsSend, CheckUserAppSenderSmsView, CheckUserAppView, CheckUserBarCodeEdit,
    CheckUserBarCodeView, CheckUserEmailEdit, CheckUserExternalEdit, CheckUserInfoEdit,
    CheckUserMobileEdit, CheckUserNotifyView, CheckUserRbacEdit, CheckUserRbacView,
};
use crate::dao::access::rest::CheckRestApp;
use crate::dao::{CheckResTpl, WebRbac};

impl WebRbac {
    pub fn res_tpl_data(&self, user: bool, data: bool) -> Vec<CheckResTpl> {
        let res_tpl = access_res_tpl!(
            CheckAdminMailConfig,
            CheckAdminMailMgr,
            CheckAdminBase,
            CheckAdminSmsConfig,
            CheckAdminSmsMgr,
            CheckAdminMailMgr,
            CheckAdminDocs,
            CheckAdminRbacView,
            CheckAdminRbacEdit,
            CheckAdminApp,
            CheckAdminSiteSetting,
            CheckAdminUserManage,
            CheckAdminChangeLogsView,
            CheckUserAddressEdit,
            CheckUserEmailEdit,
            CheckUserExternalEdit,
            CheckUserInfoEdit,
            CheckUserMobileEdit,
            CheckUserAppView,
            CheckUserAppEdit,
            CheckUserBarCodeView,
            CheckUserBarCodeEdit,
            CheckUserNotifyView,
            CheckUserRbacView,
            CheckUserRbacEdit,
            CheckUserAppSenderMailConfig,
            CheckUserAppSenderMailView,
            CheckUserAppSenderMailSend,
            CheckUserAppSenderSmsConfig,
            CheckUserAppSenderSmsView,
            CheckUserAppSenderSmsSend,
            CheckSystemLogin,
            CheckSystemRegister,
            CheckRestApp,
        );
        res_tpl
            .into_iter()
            .filter(|e| e.user == user && e.data == data)
            .collect::<Vec<_>>()
    }
}
