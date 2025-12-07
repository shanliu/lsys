use lsys_core::db::{ModelTableName, SqlExpr};
use lsys_core::sql_format;
use lsys_core::PageParam;

use super::App;
use crate::model::AppModel;
use crate::{
    dao::AppResult,
    model::{
        AppRequestFeatureModel, AppRequestModel, AppRequestOAuthClientModel,
        AppRequestSetInfoModel, AppRequestStatus, AppRequestType,
    },
};
use lsys_core::db::SqlQuote;

#[derive(Clone, Debug)]
pub struct AppRequestParam {
    pub id: Option<u64>,
    pub request_user_id: Option<u64>, //查看指定用户的申请列表
    pub app_id: Option<u64>,          //查看app的申请列表
    pub parent_app_id: Option<u64>,   //查看子级的待审核列表
    pub status: Option<AppRequestStatus>,
    pub request_type: Option<AppRequestType>,
}

pub enum AppRequestData {
    None,
    Feature(AppRequestFeatureModel),
    OAuthClient(AppRequestOAuthClientModel),
    ChangeInfo(AppRequestSetInfoModel),
}

#[derive(Default)]
pub struct AppInfoData {
    pub parent_app_id: u64,
    pub parent_app_name: String,
    pub parent_app_client_id: String,
    pub parent_app_status: i8,
    pub parent_app_user_id: u64,
    pub name: String,
    pub client_id: String,
    pub status: i8,
}

impl App {
    /// 根据请求 id 找到对应记录
    pub async fn request_find_by_id(&self, id: u64) -> AppResult<AppRequestModel> {
        Ok(sqlx::query_as::<_, AppRequestModel>(&sql_format!(
            "select * from {} where id={}",
            AppRequestModel::table_name(),
            id
        ))
        .fetch_one(&self.db)
        .await?)
    }
    fn app_request_where_sql(&self, req_param: &AppRequestParam) -> Option<String> {
        let mut sql = vec![];
        if let Some(tmp) = req_param.id {
            sql.push(sql_format!("id={}", tmp));
        }
        if let Some(tmp) = req_param.app_id {
            sql.push(sql_format!("app_id={}", tmp));
        }
        if let Some(tmp) = req_param.parent_app_id {
            sql.push(sql_format!("parent_app_id={}", tmp));
        }
        if let Some(tmp) = req_param.status {
            sql.push(sql_format!("status={}", tmp as i8));
        }
        if let Some(tmp) = req_param.request_type {
            sql.push(sql_format!("request_type={}", tmp as i8));
        }
        if let Some(tmp) = req_param.request_user_id {
            sql.push(sql_format!("request_user_id={}", tmp));
        }
        Some(sql.join(" and "))
    }
    //待审核列表
    pub async fn app_request_data(
        &self,
        req_param: &AppRequestParam,
        page: Option<&PageParam>,
    ) -> AppResult<Vec<(AppRequestModel, AppInfoData, AppRequestData)>> {
        let where_sql = match self.app_request_where_sql(req_param) {
            Some(s) => s,
            None => return Ok(vec![]),
        };
        let page_sql = if let Some(pdat) = page {
            format!(
                " order by id desc limit {} offset {} ",
                pdat.limit, pdat.offset
            )
        } else {
            " order by id desc".to_string()
        };
        let data = sqlx::query_as::<_, AppRequestModel>(&sql_format!(
            "select * from {} {} {}",
            AppRequestModel::table_name(),
            if !where_sql.is_empty() {
                SqlExpr(format!(" where {}", where_sql))
            } else {
                SqlExpr("".to_string())
            },
            SqlExpr(page_sql),
        ))
        .fetch_all(&self.db)
        .await?;

        let mut app_id_tmp = data.iter().map(|t| t.app_id).collect::<Vec<u64>>();

        let parent_id_tmp = data
            .iter()
            .map(|t| t.parent_app_id)
            .filter(|t| *t > 0)
            .collect::<Vec<u64>>();
        app_id_tmp.extend(parent_id_tmp);

        let app_info_data = if !app_id_tmp.is_empty() {
            sqlx::query_as::<_, (u64, u64, String, String, i8, u64)>(&sql_format!(
                "select id,parent_app_id,name,client_id,status,user_id from {} where id in ({})",
                AppModel::table_name(),
                app_id_tmp,
            ))
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let fet_id_tmp = data
            .iter()
            .filter(|t| AppRequestType::ExterFeatuer.eq(t.request_type))
            .map(|t| t.id)
            .collect::<Vec<u64>>();
        let fet_id_data = if !fet_id_tmp.is_empty() {
            sqlx::query_as::<_, AppRequestFeatureModel>(&sql_format!(
                "select * from {} where app_request_id in ({})",
                AppRequestFeatureModel::table_name(),
                fet_id_tmp,
            ))
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let client_id_tmp = data
            .iter()
            .filter(|t| {
                AppRequestType::OAuthClientScope.eq(t.request_type)
                    || AppRequestType::OAuthClient.eq(t.request_type)
            })
            .map(|t| t.id)
            .collect::<Vec<u64>>();
        let client_id_data = if !client_id_tmp.is_empty() {
            sqlx::query_as::<_, AppRequestOAuthClientModel>(&sql_format!(
                "select * from {} where app_request_id in ({})",
                AppRequestOAuthClientModel::table_name(),
                client_id_tmp,
            ))
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let change_id_tmp = data
            .iter()
            .filter(|t| {
                AppRequestType::AppChange.eq(t.request_type)
                    || AppRequestType::AppReq.eq(t.request_type)
            })
            .map(|t| t.id)
            .collect::<Vec<u64>>();
        let change_id_data = if !change_id_tmp.is_empty() {
            sqlx::query_as::<_, AppRequestSetInfoModel>(&sql_format!(
                "select * from {} where app_request_id in ({})",
                AppRequestSetInfoModel::table_name(),
                change_id_tmp,
            ))
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        Ok(data
            .into_iter()
            .map(|e| {
                let out_attr = if AppRequestType::OAuthClientScope.eq(e.request_type)
                    || AppRequestType::OAuthClient.eq(e.request_type)
                {
                    client_id_data
                        .iter()
                        .find(|t| t.app_request_id == e.id)
                        .map(|s| AppRequestData::OAuthClient(s.to_owned()))
                        .unwrap_or(AppRequestData::None)
                } else if AppRequestType::ExterFeatuer.eq(e.request_type) {
                    fet_id_data
                        .iter()
                        .find(|t| t.app_request_id == e.id)
                        .map(|s| AppRequestData::Feature(s.to_owned()))
                        .unwrap_or(AppRequestData::None)
                } else if AppRequestType::AppReq.eq(e.request_type)
                    || AppRequestType::AppChange.eq(e.request_type)
                {
                    change_id_data
                        .iter()
                        .find(|t| t.app_request_id == e.id)
                        .map(|s| AppRequestData::ChangeInfo(s.to_owned()))
                        .unwrap_or(AppRequestData::None)
                } else {
                    AppRequestData::None
                };
                let par_info = app_info_data
                    .iter()
                    .find(|t| t.0 == e.parent_app_id)
                    .map(|t| (t.2.to_owned(), t.3.to_owned(), t.4, t.5))
                    .unwrap_or_default();
                let app_info = app_info_data
                    .iter()
                    .find(|t| t.0 == e.app_id)
                    .map(|t| AppInfoData {
                        parent_app_id: t.1,
                        parent_app_name: par_info.0.to_owned(),
                        parent_app_client_id: par_info.1.to_owned(),
                        parent_app_status: par_info.2,
                        parent_app_user_id: par_info.3,
                        name: t.2.clone(),
                        client_id: t.3.clone(),
                        status: t.4,
                    })
                    .unwrap_or_default();
                (e, app_info, out_attr)
            })
            .collect::<Vec<_>>())
    }
    //待审核总数
    pub async fn app_request_count(&self, req_param: &AppRequestParam) -> AppResult<i64> {
        let where_sql = match self.app_request_where_sql(req_param) {
            Some(s) => s,
            None => return Ok(0),
        };
        let sql = sql_format!(
            "select count(*) as total from {} {}",
            AppRequestModel::table_name(),
            if !where_sql.is_empty() {
                SqlExpr(format!(" where {}", where_sql))
            } else {
                SqlExpr("".to_string())
            }
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
}
