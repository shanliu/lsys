use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::{
    api::user::{user_search, UserSearchParam},
    oauth::user::UserIdSearchParam,
};

#[post("list/{method}")]
pub(crate) async fn user_list<'t>(
    jwt: JwtQuery,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<(String,)>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.0.to_string().as_str() {
        "search" => user_search(json_param.param::<UserSearchParam>()?, &auth_dao).await,
        "id_search" => {
            lsys_web::handler::oauth::user::user_id_search(
                json_param.param::<UserIdSearchParam>()?,
                &auth_dao,
            )
            .await
        }

        name => handler_not_found!(name),
    }?
    .into())
}
