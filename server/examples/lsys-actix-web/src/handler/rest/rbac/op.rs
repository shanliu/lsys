use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use lsys_web::handler::rest::rbac::{
    op_add, op_data, op_del, op_edit, OpAddParam, OpDataParam, OpDelParam, OpEditParam,
};

#[post("/op")]
pub async fn op(rest: RestQuery) -> ResponseJsonResult<ResponseJson> {
    let data = match rest.rfc.method.as_deref().unwrap_or_default() {
        "add" => op_add(&rest.param::<OpAddParam>()?, &rest.get_app().await?, &rest).await,
        "edit" => op_edit(&rest.param::<OpEditParam>()?, &rest.get_app().await?, &rest).await,
        "delete" => op_del(&rest.param::<OpDelParam>()?, &rest.get_app().await?, &rest).await,
        "list" => op_data(&rest.param::<OpDataParam>()?, &rest.get_app().await?, &rest).await,
        name => handler_not_found!(name),
    };
    Ok(data.map_err(|e| rest.fluent_error_json_response(&e))?.into())
}
