use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery, ReqQuery};
use actix_web::{post, web};
use lsys_web::dao::WebDao;
use lsys_web::handler::rest::rbac::{
    CheckParam, RbacMenuListParam, access_check, access_list_check, mapping_data,
};

#[post("/base")]
pub async fn base(
    rest: RestQuery,
    req_dao: ReqQuery,
    web_dao: web::Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let data = match rest.rfc.method.as_deref().unwrap_or_default() {
        "mapping" => mapping_data(&req_dao).await,
        "access" => access_check(&rest.param::<CheckParam>()?, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await,
        "access_list" => {
            access_list_check(
                &rest.param::<RbacMenuListParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| req_dao.fluent_error_json_response(&e))?
        .into())
}
