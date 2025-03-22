use super::inner_app_rbac_check;
use crate::common::{JsonData, JsonResult, RequestDao};
use lsys_app::model::AppModel;
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, AccessSessionRole};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct ResCheckParam {
    pub res_type: String, //资源KEY
    pub res_data: String, //资源KEY
    pub user_id: u64,     //资源用户ID,0为APP用户
    pub ops: Vec<String>, //授权列表
}

#[derive(Debug, Deserialize)]
pub struct RoleCheckParam {
    pub role_key: String,
    pub user_id: u64, //角色用户ID,0为APP用户
}

#[derive(Debug, Deserialize)]
pub struct AccessCheckParam {
    pub role_key: Vec<RoleCheckParam>,
    pub check_res: Vec<Vec<ResCheckParam>>,
}

#[derive(Debug, Deserialize)]
pub struct CheckParam {
    pub user_data: Option<String>,
    pub token_data: Option<String>,
    pub request_ip: Option<String>,
    pub access: AccessCheckParam,
}

pub async fn access_check(
    param: &CheckParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    inner_app_rbac_check(app, req_dao).await?;
    let user_data = match param.user_data.as_ref() {
        Some(user_data) => Some(
            req_dao
                .web_dao
                .web_access
                .access_dao
                .user
                .cache()
                .sync_user(app.id, user_data, None, None)
                .await?,
        ),
        None => None,
    };
    let session_role = param
        .access
        .role_key
        .iter()
        .map(|e| AccessSessionRole {
            role_key: &e.role_key,
            user_id: if e.user_id == 0 {
                app.user_id
            } else {
                e.user_id
            },
            app_id: app.id,
        })
        .collect::<Vec<_>>();
    let mut req_env = req_dao.req_env.clone();
    req_env.request_ip = param.request_ip.clone();
    let check_env = AccessCheckEnv {
        req_env: Some(&req_env),
        user_id: user_data.map(|e| e.id).unwrap_or_default(),
        login_token_data: param.token_data.as_deref(),
        session_role,
    };
    let access_checks = param
        .access
        .check_res
        .iter()
        .map(|check_res_group| {
            check_res_group
                .iter()
                .map(|check_res| AccessCheckRes {
                    user_id: if check_res.user_id == 0 {
                        app.user_id
                    } else {
                        check_res.user_id
                    },
                    res_type: &check_res.res_type,
                    res_data: &check_res.res_data,
                    app_id: app.id,
                    op_key_data: check_res.ops.iter().map(|e| e.as_str()).collect::<Vec<_>>(),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .list_check(
            &check_env,
            &access_checks
                .iter()
                .map(|e| e.as_slice())
                .collect::<Vec<_>>(),
        )
        .await?;
    Ok(JsonData::default())
}

// #[derive(Debug, Deserialize)]
// pub struct CheckMenuParam {
//     pub user_id: u64,
//     pub check_res: Vec<CheckAccessParam>,
// }

// pub async fn menu_check(
//     req_dao: &RequestDao,
//     app: &AppModel,
//     param: &CheckMenuParam,
// ) -> JsonResult<JsonData> {
//     req_dao.web_dao.app_sender.mailer.feature_check(app).await?;
//     rbac_menu_check(
//         &req_dao.access_env(),
//         RbacMenuParam {
//             check_res: param.check_res,
//         },
//         &req_dao.web_dao.rbac,
//     )
//     .await
// }

// #[derive(Debug, Deserialize)]
// pub struct AccessParam {
//     pub user_id: u64,
//     pub access: RbacAccessParam,
// }

// pub async fn access_check(
//     app_dao: &WebDao,
//     app: &AppModel,
//     param: &AccessParam,
// ) -> JsonResult<JsonData> {
//     app_dao
//         .user
//         .rbac_dao
//         .rbac
//         .check(
//             &CheckSubAppRbacCheck {
//                 app: app.to_owned(),
//             },
//             None,
//         )
//         .await?;
//     rbac_access_check(param.user_id, param.access, &app_dao.user.rbac_dao).await
// }
