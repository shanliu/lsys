use crate::common::JsonData;
use crate::common::JsonError;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::dao::access::api::system::admin::{
    CheckAdminBase, CheckAdminChangeLogsView, CheckAdminMailConfig, CheckAdminSmsConfig,
};
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_core::fluent_message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
#[derive(Debug, Deserialize)]
pub struct RbacAccessParam {
    pub name: String,
    pub data: Value,
}

pub async fn perm_check(
    check_res: &RbacAccessParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    macro_rules! check {
        //$key 以接口为维度的关键字,外部传入
        //$data 该接口涉及的权限,参考具体结构的权限校验代码
        ($key:literal,$data:expr) => {
            if check_res.name == $key {
                let _ = req_dao
                    .web_dao
                    .web_rbac
                    .check(
                        &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
                        &$data,
                    )
                    .await?;
            }
        };
    }
    check!("admin-sms-config", CheckAdminSmsConfig {});
    check!("admin-mail-config", CheckAdminMailConfig {});
    check!("admin-main", CheckAdminBase {});
    check!("admin-logs", CheckAdminChangeLogsView {});

    Err(JsonError::Message(fluent_message!("rbac-unkown-res",{
        "res":&check_res.name
    })))
}

#[derive(Debug, Deserialize)]
pub struct RbacAccessMenuParam {
    pub check_res: Vec<RbacAccessParam>,
}

#[derive(Debug, Serialize)]
pub struct RbacMenuStatus {
    pub status: bool, //是否授权成功
    pub name: String, //key,参见perm_check 定义
}

pub async fn perm_menu_check(
    param: &RbacAccessMenuParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let mut out = Vec::with_capacity(param.check_res.len());
    for e in param.check_res.iter() {
        out.push(RbacMenuStatus {
            status: perm_check(e, req_dao).await.map(|_| true).unwrap_or(false),
            name: e.name.to_owned(),
        })
    }
    Ok(JsonResponse::data(JsonData::body(json!({"result":out}))))
}
