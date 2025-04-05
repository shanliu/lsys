use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;

use crate::common::JsonData;
use crate::common::JsonError;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::dao::access::api::system::{
    CheckAdminBase, CheckAdminChangeLogsView, CheckAdminDocs, CheckAdminMailConfig,
    CheckAdminSmsConfig,
};
use crate::dao::access::api::user::CheckUserAppSenderSmsConfig;
use lsys_access::dao::AccessSession;
use lsys_core::fluent_message;
#[derive(Debug, Deserialize)]
pub struct RbacAccessData {
    pub name: String,
    pub data: Value,
}

pub async fn perm_check(
    check_res: &RbacAccessData,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    macro_rules! check {
        //$key 以接口为维度的关键字,外部传入
        //$data 该接口涉及的权限,参考具体结构的权限校验代码
        ($key:literal,$data:expr) => {
            if check_res.name == $key {
                let _ = req_dao
                    .web_dao
                    .web_rbac
                    .check(&req_dao.req_env, Some(&auth_data), &$data)
                    .await?;
            }
        };
    }
    check!("admin-sms-config", CheckAdminSmsConfig {});
    check!(
        "admin-sender-config",
        CheckUserAppSenderSmsConfig { res_user_id: 0 }
    );
    check!("admin-mail-config", CheckAdminMailConfig {});

    check!("admin-main", CheckAdminBase {});

    check!("admin-logs", CheckAdminChangeLogsView {});
    check!("docs-edit", CheckAdminDocs {});

    Err(JsonError::Message(fluent_message!("rbac-unkown-res",{
        "res":&check_res.name
    })))
}

#[derive(Debug, Deserialize)]
pub struct RbacAccessMenuParam {
    pub check_res: Vec<RbacAccessData>,
}

#[derive(Debug, Serialize)]
pub struct RbacMenuStatus {
    pub status: bool, //是否授权成功
    pub name: String, //key,参见perm_check 定义
}

pub async fn perm_menu_check(
    param: &RbacAccessMenuParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let mut out = Vec::with_capacity(param.check_res.len());
    for e in param.check_res.iter() {
        out.push(RbacMenuStatus {
            status: perm_check(e, req_dao).await.map(|_| true).unwrap_or(false),
            name: e.name.to_owned(),
        })
    }
    Ok(JsonData::data(json!({"result":out})))
}
