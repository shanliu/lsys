use lsys_core::RequestEnv;
use lsys_rbac::dao::ResTypeParam;

use super::user::RbacUserSyncOpParam;
use crate::common::JsonResult;
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
use crate::dao::user::RbacUserSyncResParam;
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

pub struct RbacUserSyncResRecrod {
    pub res_type: String,
    pub res_data: String,
    pub res_id: u64,
    pub op_data: Vec<(String, u64)>,
}

impl WebRbac {
    //根据
    pub async fn res_tpl_user_sync(
        &self,
        user_id: u64,
        res_type: &str,
        res_data: &[impl AsRef<str>],
        init_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<Vec<RbacUserSyncResRecrod>> {
        let tpl_data = self.res_tpl_data(true, true);
        let res_id_str = res_data.iter().map(|e| e.as_ref()).collect::<Vec<_>>();
        let res_param = res_id_str
            .iter()
            .map(|e| RbacUserSyncResParam {
                res_type,
                res_data: e,
                init_res_name: None,
            })
            .collect::<Vec<_>>();
        let res_db_data = self
            .user_sync_res_id(user_id, 0, &res_param, init_user_id, env_data)
            .await?;

        let op_data = if let Some(op_tpl) = tpl_data.iter().find(|e| e.key == res_type) {
            let key_data = op_tpl
                .ops
                .iter()
                .map(|e| RbacUserSyncOpParam {
                    op_key: e,
                    init_op_name: None,
                })
                .collect::<Vec<_>>();
            let key_data = key_data.iter().collect::<Vec<_>>();
            let tpl_data = self
                .user_sync_res_type_op_id(
                    &ResTypeParam {
                        res_type,
                        user_id,
                        app_id: 0,
                    },
                    &key_data,
                    init_user_id,
                    env_data,
                )
                .await?;
            tpl_data
                .iter()
                .map(|(e, op_id)| (e.op_key.to_string(), *op_id))
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let mut out_data = vec![];

        for (res, res_id) in res_db_data {
            out_data.push(RbacUserSyncResRecrod {
                res_type: res.res_type.to_string(),
                res_data: res.res_data.to_string(),
                res_id,
                op_data: op_data.clone(),
            })
        }
        Ok(out_data)
    }
}
