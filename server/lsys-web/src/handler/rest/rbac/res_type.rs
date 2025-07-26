use super::{inner_app_rbac_check, inner_app_self_check, inner_user_data_to_user_id};
use crate::common::JsonData;
use crate::common::{JsonResponse, JsonResult, PageParam, RequestDao};
use lsys_app::model::AppModel;
use lsys_rbac::dao::{ResTypeListParam as RbacResTypeListParam, ResTypeParam};
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ResTypeListParam {
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub use_app_user: bool,
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub res_type: Option<String>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn res_type_data(
    param: &ResTypeListParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    let user_id = inner_user_data_to_user_id(
        app,
        param.use_app_user,
        param.user_param.as_deref(),
        req_dao,
    )
    .await?;
    let res_type_param = param.res_type.as_ref().and_then(|e| {
        if e.trim_matches(['\n', ' ', '\t']).is_empty() {
            None
        } else {
            Some(e)
        }
    });
    let res_param = RbacResTypeListParam {
        user_id: Some(user_id),
        app_id: Some(app.id),
        res_type: res_type_param.map(|x| x.as_str()),
    };

    let rows = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_type_data(&res_param, param.page.as_ref().map(|e| e.into()).as_ref())
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .res
                .res_type_count(&res_param)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": bind_vec_user_info_from_req!(req_dao, rows, user_id),"total":count}),
    )))
}

#[derive(Debug, Deserialize)]
pub struct ResTypeAddOpParam {
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub use_app_user: bool,
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub res_type: String,
    #[serde(deserialize_with = "crate::common::deserialize_vec_u64")]
    pub op_ids: Vec<u64>,
}

pub async fn res_type_op_add(
    param: &ResTypeAddOpParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    let user_id = inner_user_data_to_user_id(
        app,
        param.use_app_user,
        param.user_param.as_deref(),
        req_dao,
    )
    .await?;

    let mut op_data = vec![];
    for tmp in req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_ids(&param.op_ids)
        .await?
        .into_iter()
    {
        inner_app_self_check(app, tmp.1.app_id)?;
        op_data.push(tmp.1);
    }

    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_type_add_op(
            &ResTypeParam {
                res_type: &param.res_type,
                user_id,
                app_id: app.id,
            },
            &op_data,
            app.user_id,
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ResDelOpParam {
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub use_app_user: bool,
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub res_type: String,
    #[serde(deserialize_with = "crate::common::deserialize_vec_u64")]
    pub op_ids: Vec<u64>,
}

pub async fn res_type_op_del(
    param: &ResDelOpParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    let user_id = inner_user_data_to_user_id(
        app,
        param.use_app_user,
        param.user_param.as_deref(),
        req_dao,
    )
    .await?;

    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_type_del_op(
            &ResTypeParam {
                res_type: &param.res_type,
                user_id,
                app_id: app.id,
            },
            &param.op_ids,
            app.user_id,
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ResTypeOpListParam {
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub use_app_user: bool,
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub res_type: String,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn res_type_op_data(
    param: &ResTypeOpListParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    let user_id = inner_user_data_to_user_id(
        app,
        param.use_app_user,
        param.user_param.as_deref(),
        req_dao,
    )
    .await?;

    let res_param = ResTypeParam {
        res_type: &param.res_type,
        user_id,
        app_id: app.id,
    };

    let rows = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_type_op_data(
            &res_param,
            None,
            true,
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .res
                .res_type_op_count(&res_param)
                .await?,
        )
    } else {
        None
    };

    let user_id = rows.iter().map(|e| e.op_res.user_id).collect::<Vec<_>>();
    let user_data = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_users_by_ids(&user_id)
        .await?;
    let rows = rows
        .iter()
        .map(|e| {
            let mut val = json!(e);
            if let serde_json::Value::Object(ref mut map) = val {
                map.insert(
                    "user_data".to_string(),
                    json!(user_data.get(e.op_res.user_id)),
                );
            }
            val
        })
        .collect::<Vec<_>>();
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": rows,"total":count}),
    )))
}
