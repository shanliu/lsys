use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use lsys_web::handler::rest::rbac::{
    res_add, res_data, res_del, res_edit, res_type_data, res_type_op_add, res_type_op_data,
    res_type_op_del, ResAddParam, ResDelOpParam, ResDelParam, ResEditParam, ResParam,
    ResTypeAddOpParam, ResTypeListParam, ResTypeOpListParam,
};

#[post("/res")]
pub async fn res(rest: RestQuery) -> ResponseJsonResult<ResponseJson> {
    let data = match rest.rfc.method.as_deref().unwrap_or_default() {
        "add" => res_add(&rest.param::<ResAddParam>()?, &rest.get_app().await?, &rest).await,
        "edit" => {
            res_edit(
                &rest.param::<ResEditParam>()?,
                &rest.get_app().await?,
                &rest,
            )
            .await
        }
        "delete" => res_del(&rest.param::<ResDelParam>()?, &rest.get_app().await?, &rest).await,
        "list" => res_data(&rest.param::<ResParam>()?, &rest.get_app().await?, &rest).await,
        "type_data" => {
            res_type_data(
                &rest.param::<ResTypeListParam>()?,
                &rest.get_app().await?,
                &rest,
            )
            .await
        }
        "type_op_add" => {
            res_type_op_add(
                &rest.param::<ResTypeAddOpParam>()?,
                &rest.get_app().await?,
                &rest,
            )
            .await
        }
        "type_op_del" => {
            res_type_op_del(
                &rest.param::<ResDelOpParam>()?,
                &rest.get_app().await?,
                &rest,
            )
            .await
        }
        "type_op_data" => {
            res_type_op_data(
                &rest.param::<ResTypeOpListParam>()?,
                &rest.get_app().await?,
                &rest,
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| rest.fluent_error_json_response(&e))?
        .into())
}
