use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use lsys_web::handler::rest::rbac::{
    access_check, access_list_check, CheckParam, RbacMenuListParam,
};

#[post("/check/{method}")]
pub async fn check(rest: RestQuery) -> ResponseJsonResult<ResponseJson> {
    let data = match rest.rfc.method.as_deref().unwrap_or_default() {
        "access" => access_check(&rest.param::<CheckParam>()?, &rest.get_app().await?, &rest).await,
        "access_list" => {
            access_list_check(
                &rest.param::<RbacMenuListParam>()?,
                &rest.get_app().await?,
                &rest,
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(data.map_err(|e| rest.fluent_error_json_response(&e))?.into())
}
