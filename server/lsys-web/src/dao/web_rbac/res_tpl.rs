use crate::dao::access::api::auth::{CheckSystemLogin, CheckSystemRegister};
use crate::dao::access::api::system::{
    CheckAdminApp, CheckAdminBase, CheckAdminChangeLogsView, CheckAdminDocs, CheckAdminMailConfig,
    CheckAdminMailMgr, CheckAdminRbacEdit, CheckAdminRbacView, CheckAdminSiteSetting,
    CheckAdminSmsConfig, CheckAdminSmsMgr, CheckAdminUserManage,
};
use crate::dao::access::api::user::{
    CheckUserAddressBase, CheckUserAddressEdit, CheckUserAppEdit, CheckUserAppSenderMailConfig,
    CheckUserAppSenderMailMsg, CheckUserAppSenderMailSend, CheckUserAppSenderSmsConfig,
    CheckUserAppSenderSmsMsg, CheckUserAppSenderSmsSend, CheckUserAppView, CheckUserBarCodeEdit,
    CheckUserBarCodeView, CheckUserEmailBase, CheckUserEmailEdit, CheckUserExternalEdit,
    CheckUserInfoEdit, CheckUserMobileBase, CheckUserMobileEdit, CheckUserNotifyView,
    CheckUserRbacEdit, CheckUserRbacView,
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
            CheckUserAddressBase,
            CheckUserAddressEdit,
            CheckUserEmailBase,
            CheckUserEmailEdit,
            CheckUserExternalEdit,
            CheckUserInfoEdit,
            CheckUserMobileBase,
            CheckUserMobileEdit,
            CheckUserAppView,
            CheckUserAppEdit,
            CheckUserBarCodeView,
            CheckUserBarCodeEdit,
            CheckUserNotifyView,
            CheckUserRbacView,
            CheckUserRbacEdit,
            CheckUserAppSenderMailConfig,
            CheckUserAppSenderMailMsg,
            CheckUserAppSenderMailSend,
            CheckUserAppSenderSmsConfig,
            CheckUserAppSenderSmsMsg,
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
