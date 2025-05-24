use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use lsys_docs::model::DocGitCloneStatus;
use lsys_docs::model::DocGitTagStatus;
use serde_json::json;
pub async fn mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
        "tag_status":vec![
            status_format!(json req_dao, DocGitTagStatus::Publish),
            status_format!(json req_dao, DocGitTagStatus::Build),
        ],
           "clone_status":vec![
            status_format!(json req_dao, DocGitCloneStatus::Init),
            status_format!(json req_dao, DocGitCloneStatus::Cloned),
            status_format!(json req_dao, DocGitCloneStatus::Fail),
        ],
    }))))
}
