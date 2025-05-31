use lsys_core::db::{ModelTableName, SqlExpr};
use lsys_core::sql_format;
use lsys_core::PageParam;

use super::App;
use crate::{
    dao::AppResult,
    model::{
        AppRequestFeatureModel, AppRequestModel, AppRequestOAuthClientModel,
        AppRequestSetInfoModel, AppRequestStatus, AppRequestType,
    },
};
use lsys_core::db::SqlQuote;

pub enum AppRequestData {
    None,
    Feature(AppRequestFeatureModel),
    OAuthClient(AppRequestOAuthClientModel),
    ChangeInfo(AppRequestSetInfoModel),
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
    //待审核列表
    pub async fn app_request_data(
        &self,
        app_id: Option<u64>,        //查看app的申请列表
        parent_app_id: Option<u64>, //查看子级的待审核列表
        status: Option<AppRequestStatus>,
        page: Option<&PageParam>,
    ) -> AppResult<Vec<(AppRequestModel, AppRequestData)>> {
        let mut sql = vec![];
        if let Some(tmp) = app_id {
            sql.push(sql_format!("app_id={}", tmp));
        }
        if let Some(tmp) = parent_app_id {
            sql.push(sql_format!("parent_app_id={}", tmp));
        }
        if let Some(tmp) = status {
            sql.push(sql_format!("status={}", tmp as i8));
        }
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
            if !sql.is_empty() {
                SqlExpr(format!(" where {}", sql.join(" and ")))
            } else {
                SqlExpr("".to_string())
            },
            SqlExpr(page_sql),
        ))
        .fetch_all(&self.db)
        .await?;

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
                (e, out_attr)
            })
            .collect::<Vec<_>>())
    }
    //待审核总数
    pub async fn app_request_count(
        &self,
        app_id: Option<u64>,
        parent_app_id: Option<u64>,
        status: Option<AppRequestStatus>,
    ) -> AppResult<i64> {
        let mut sql = vec![];
        if let Some(tmp) = app_id {
            sql.push(sql_format!("app_id={}", tmp));
        }
        if let Some(tmp) = parent_app_id {
            sql.push(sql_format!("parent_app_id={}", tmp));
        }
        if let Some(tmp) = status {
            sql.push(sql_format!("status={}", tmp as i8));
        }
        let sql = sql_format!(
            "select count(*) as total from {} {}",
            AppRequestModel::table_name(),
            if !sql.is_empty() {
                SqlExpr(format!(" where {}", sql.join(" and ")))
            } else {
                SqlExpr("".to_string())
            }
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
}
