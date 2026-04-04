use lsys_core::db::{OffsetPageParam, TableMeta};

use super::App;
use crate::model::AppModel;
use crate::{
    dao::AppResult,
    model::{
        AppRequestFeatureModel, AppRequestModel, AppRequestOAuthClientModel,
        AppRequestSetInfoModel, AppRequestStatus, AppRequestType,
    },
};
use lsys_core::db::{QueryBuilderExt, WhereClause};
use sqlx::{MySql, QueryBuilder};

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
        Ok(sqlx::query_as::<_, AppRequestModel>(&format!(
            "select * from {} where id=?",
            AppRequestModel::table_name(),
        ))
        .bind(id)
        .fetch_one(&self.db)
        .await?)
    }
    fn app_request_push_where(wb: &mut WhereClause<'_, '_, MySql>, req_param: &AppRequestParam) {
        if let Some(tmp) = req_param.id {
            wb.and().field_eq("id", tmp);
        }
        if let Some(tmp) = req_param.app_id {
            wb.and().field_eq("app_id", tmp);
        }
        if let Some(tmp) = req_param.parent_app_id {
            wb.and().field_eq("parent_app_id", tmp);
        }
        if let Some(tmp) = req_param.status {
            wb.and().field_eq("status", tmp as i8);
        }
        if let Some(tmp) = req_param.request_type {
            wb.and().field_eq("request_type", tmp as i8);
        }
        if let Some(tmp) = req_param.request_user_id {
            wb.and().field_eq("request_user_id", tmp);
        }
    }
}
impl App {
    //待审核列表
    pub async fn app_request_info(
        &self,
        req_param: &AppRequestParam,
        page: &OffsetPageParam,
    ) -> AppResult<Vec<(AppRequestModel, AppInfoData, AppRequestData)>> {
        let data = self.app_request_data(req_param, page).await?;

        let mut app_id_tmp = data.iter().map(|t| t.app_id).collect::<Vec<u64>>();

        let parent_id_tmp = data
            .iter()
            .map(|t| t.parent_app_id)
            .filter(|t| *t > 0)
            .collect::<Vec<u64>>();
        app_id_tmp.extend(parent_id_tmp);

        let app_info_data = if !app_id_tmp.is_empty() {
            let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select id,parent_app_id,name,client_id,status,user_id from {}",
                AppModel::table_name()
            ));
            qb.push_where().field_in_copied("id", &app_id_tmp);
            qb.build_query_as::<(u64, u64, String, String, i8, u64)>()
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
            let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select * from {}",
                AppRequestFeatureModel::table_name()
            ));
            qb.push_where().field_in_copied("app_request_id", &fet_id_tmp);
            qb.build_query_as::<AppRequestFeatureModel>()
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
            let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select * from {}",
                AppRequestOAuthClientModel::table_name()
            ));
            qb.push_where().field_in_copied("app_request_id", &client_id_tmp);
            qb.build_query_as::<AppRequestOAuthClientModel>()
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
            let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select * from {}",
                AppRequestSetInfoModel::table_name()
            ));
            qb.push_where().field_in_copied("app_request_id", &change_id_tmp);
            qb.build_query_as::<AppRequestSetInfoModel>()
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
    //待审核列表
    pub async fn app_request_data(
        &self,
        req_param: &AppRequestParam,
        page: &OffsetPageParam,
    ) -> AppResult<Vec<AppRequestModel>> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select * from {}",
            AppRequestModel::table_name(),
        ));
        {
            let mut wb = WhereClause::new(&mut qb);
            Self::app_request_push_where(&mut wb, req_param);
        }
        qb.push(" order by id desc");
        page.push_limit(&mut qb);
        let data = qb.build_query_as::<AppRequestModel>()
            .fetch_all(&self.db)
            .await?;
        Ok(data)
    }
    //待审核总数
    pub async fn app_request_count(&self, req_param: &AppRequestParam) -> AppResult<i64> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select count(*) as total from {}",
            AppRequestModel::table_name(),
        ));
        {
            let mut wb = WhereClause::new(&mut qb);
            Self::app_request_push_where(&mut wb, req_param);
        }
        let res = qb.build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?;
        Ok(res)
    }
}


