use lsys_app::model::AppModel;
use lsys_rbac::dao::{AccessCheckRes, AccessSessionRole};
use serde::Deserialize;

use crate::{
    common::{JsonData, JsonResult, RequestDao},
    dao::{access::rest::CheckRestApp, APP_FEATURE_RBAC},
};

#[derive(Debug, Deserialize)]
pub struct CheckResParam {
    pub res_type: String, //资源KEY
    pub res_data: String, //资源KEY
    pub user_id: u64,     //资源用户ID
    pub ops: Vec<String>, //授权列表
}

#[derive(Debug, Deserialize)]
pub struct CheckRoleParam {
    pub role_key: String,
    pub user_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct CheckAccessParam {
    pub role_key: Vec<CheckRoleParam>,
    pub check_res: Vec<Vec<CheckResParam>>,
}

#[derive(Debug, Deserialize)]
pub struct CheckParam {
    pub user_id: u64,
    pub access: CheckAccessParam,
}

pub async fn app_rbac_check(
    param: &CheckParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env(),
            &CheckRestApp { app_id: app.id },
            None,
        )
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(app, &[APP_FEATURE_RBAC])
        .await?;

    let rkey = param
        .access
        .role_key
        .iter()
        .map(|e| AccessSessionRole {
            role_key: e.role_key.as_str(),
            user_id: e.user_id,
        })
        .collect::<Vec<_>>();

    let access_checks = param
        .access
        .check_res
        .iter()
        .map(|check_res_group| {
            check_res_group
                .iter()
                .map(|check_res| AccessCheckRes {
                    user_id: check_res.user_id,
                    res_type: &check_res.res_type,
                    res_data: &check_res.res_data,
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
            &req_dao.access_env(),
            &rkey,
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

// pub async fn app_rbac_menu_check(
//     req_dao: &RequestDao,
//     app: &AppModel,
//     param: &CheckMenuParam,
// ) -> JsonResult<JsonData> {
//     req_dao
//         .web_dao
//         .web_app
//         .app_dao
//         .app
//         .cache()
//         .exter_feature_check(app, &[APP_FEATURE_RBAC])
//         .await?;
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

// pub async fn app_rbac_access_check(
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
