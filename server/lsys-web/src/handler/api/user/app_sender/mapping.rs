use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use lsys_app_sender::model::SenderLogStatus;
use lsys_app_sender::model::SenderLogType;
use lsys_app_sender::model::SenderMailBodyStatus;
use lsys_app_sender::model::SenderMailConfigType;
use lsys_app_sender::model::SenderMailMessageStatus;
use lsys_app_sender::model::SenderSmsBodyStatus;
use lsys_app_sender::model::SenderSmsConfigType;
use lsys_app_sender::model::SenderSmsMessageStatus;
use serde_json::json;
pub async fn mailer_mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
        "log_type":vec![
            status_format!(json req_dao, SenderLogType::Init),
            status_format!(json req_dao, SenderLogType::Send),
            status_format!(json req_dao, SenderLogType::Cancel),
        ],
        "log_status":vec![
            status_format!(json req_dao, SenderLogStatus::Succ),
            status_format!(json req_dao, SenderLogStatus::Fail),
            status_format!(json req_dao, SenderLogStatus::MessageCancel),
            status_format!(json req_dao, SenderLogStatus::NotifySucc),
            status_format!(json req_dao, SenderLogStatus::NotifyFail),
        ],
         "sms_config_type":vec![
            status_format!(json req_dao, SenderSmsConfigType::Close),
            status_format!(json req_dao, SenderSmsConfigType::Limit),
            status_format!(json req_dao, SenderSmsConfigType::MaxOfSend),
            status_format!(json req_dao, SenderSmsConfigType::PassTpl),
            status_format!(json req_dao, SenderSmsConfigType::Block),
        ],
         "mail_config_type":vec![
            status_format!(json req_dao, SenderMailConfigType::Close),
            status_format!(json req_dao, SenderMailConfigType::Limit),
            status_format!(json req_dao, SenderMailConfigType::MaxOfSend),
            status_format!(json req_dao, SenderMailConfigType::PassTpl),
            status_format!(json req_dao, SenderMailConfigType::Block),
            status_format!(json req_dao, SenderMailConfigType::BlockDomain),
        ],
        "mail_branch_status":vec![
            status_format!(json req_dao, SenderMailBodyStatus::Init),
            status_format!(json req_dao, SenderMailBodyStatus::Finish),
        ],
         "mail_send_status":vec![
            status_format!(json req_dao, SenderMailMessageStatus::Init),
            status_format!(json req_dao, SenderMailMessageStatus::IsSend),
            status_format!(json req_dao, SenderMailMessageStatus::IsReceived),
            status_format!(json req_dao, SenderMailMessageStatus::SendFail),
            status_format!(json req_dao, SenderMailMessageStatus::IsCancel),
        ],
    }))))
}

pub async fn smser_mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
        "log_type":vec![
            status_format!(json req_dao, SenderLogType::Init),
            status_format!(json req_dao, SenderLogType::Send),
            status_format!(json req_dao, SenderLogType::Cancel),
        ],
        "log_status":vec![
            status_format!(json req_dao, SenderLogStatus::Succ),
            status_format!(json req_dao, SenderLogStatus::Fail),
            status_format!(json req_dao, SenderLogStatus::MessageCancel),
            status_format!(json req_dao, SenderLogStatus::NotifySucc),
            status_format!(json req_dao, SenderLogStatus::NotifyFail),
        ],
         "sms_config_type":vec![
            status_format!(json req_dao, SenderSmsConfigType::Close),
            status_format!(json req_dao, SenderSmsConfigType::Limit),
            status_format!(json req_dao, SenderSmsConfigType::MaxOfSend),
            status_format!(json req_dao, SenderSmsConfigType::PassTpl),
            status_format!(json req_dao, SenderSmsConfigType::Block),
        ],
         "sms_branch_status":vec![
            status_format!(json req_dao, SenderSmsBodyStatus::Init),
            status_format!(json req_dao, SenderSmsBodyStatus::Finish),
        ],
         "sms_send_status":vec![
            status_format!(json req_dao, SenderSmsMessageStatus::Init),
            status_format!(json req_dao, SenderSmsMessageStatus::IsSend),
            status_format!(json req_dao, SenderSmsMessageStatus::IsReceived),
            status_format!(json req_dao, SenderSmsMessageStatus::SendFail),
            status_format!(json req_dao, SenderSmsMessageStatus::IsCancel),
        ],
    }))))
}
