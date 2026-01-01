use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use lsys_access::model::SessionStatus;
use lsys_app::dao::AppDao;
use lsys_app_barcode::dao::BarCodeDao;
use lsys_app_sender::dao;
use lsys_logger::dao::ChangeLogData;
use lsys_rbac::dao::RbacDao;
use lsys_setting::dao::SettingDao;
use lsys_user::dao::UserDao;
use lsys_user::model::AccountEmailStatus;
use lsys_user::model::AccountMobileStatus;
use lsys_user::model::AccountStatus;
use serde_json::json;

pub async fn mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    // 汇总所有 log_types

    let mut change_types: Vec<serde_json::Value> = Vec::new();

    for log_type in AppDao::log_types() {
        change_types.push(var_json_format!(req_dao, log_type));
    }
    for log_type in UserDao::log_types() {
        change_types.push(var_json_format!(req_dao, log_type));
    }
    for log_type in RbacDao::log_types() {
        change_types.push(var_json_format!(req_dao, log_type));
    }
    for log_type in SettingDao::log_types() {
        change_types.push(var_json_format!(req_dao, log_type));
    }
    for log_type in dao::log_types() {
        change_types.push(var_json_format!(req_dao, log_type));
    }
    for log_type in BarCodeDao::log_types() {
        change_types.push(var_json_format!(req_dao, log_type));
    }
    let msg_type = <crate::dao::logger::MessageView as ChangeLogData>::log_type();
    change_types.push(var_json_format!(req_dao, msg_type));

    Ok(JsonResponse::data(JsonData::body(json!({
        "session_status":vec![
            status_json_format!(req_dao, SessionStatus::Enable),
            status_json_format!(req_dao, SessionStatus::Delete),
        ],
        "mobile_status":vec![
             status_json_format!(req_dao, AccountMobileStatus::Init),
            status_json_format!(req_dao, AccountMobileStatus::Valid),
        ],
        "email_status":vec![
            status_json_format!(req_dao, AccountEmailStatus::Init),
            status_json_format!(req_dao, AccountEmailStatus::Valid),
        ],
        "account_status":vec![
            status_json_format!(req_dao, AccountStatus::Init),
            status_json_format!(req_dao, AccountStatus::Enable),
        ],
        "change_type": change_types,
    }))))
}
