use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use lsys_app_barcode::model::BarcodeCreateStatus;
use lsys_app_barcode::model::BarcodeParseStatus;
use serde_json::json;

pub async fn mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
        "create_status":vec![
            status_json_format!(req_dao, BarcodeCreateStatus::EnablePrivate),
            status_json_format!(req_dao, BarcodeCreateStatus::EnablePublic),

        ],
         "parse_status":vec![
            status_json_format!(req_dao, BarcodeParseStatus::Succ),
            status_json_format!(req_dao, BarcodeParseStatus::Fail),

        ],
    }))))
}
