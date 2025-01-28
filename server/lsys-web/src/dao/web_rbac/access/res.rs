//可用资源数据
use crate::access_res_tpl;
use crate::dao::CheckResTpl;

use super::api::system::CheckAdminBase;
use super::api::system::CheckAdminChangeLogsView;

use super::api::system::CheckAdminMailConfig;
use super::api::system::CheckAdminSmsConfig;

pub fn res_tpls() -> Vec<CheckResTpl> {
    access_res_tpl!(
        CheckAdminMailConfig,
        CheckAdminMailConfig,
        CheckAdminChangeLogsView,
        CheckAdminBase,
        CheckAdminSmsConfig
    )
}
