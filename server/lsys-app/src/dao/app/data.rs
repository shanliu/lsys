use std::collections::HashMap;

use crate::model::{
    AppFeatureModel, AppFeatureStatus, AppModel, AppOAuthClientModel, AppOAuthServerScopeModel,
    AppOAuthServerScopeStatus, AppRequestType, AppStatus,
};

use lsys_core::db::ModelTableName;
use lsys_core::db::{SqlExpr, SqlQuote};
use lsys_core::sql_format;
use lsys_core::{impl_dao_fetch_map_by_vec, PageParam};
use regex::Regex;

use super::super::{AppError, AppResult};
use super::App;

impl App {
    /// 根据APP id 找到对应记录
    pub async fn find_by_id(&self, id: &u64) -> AppResult<AppModel> {
        sqlx::query_as::<_, AppModel>(&sql_format!(
            "select * from {} where id={}",
            AppModel::table_name(),
            id
        ))
        .fetch_one(&self.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::AppNotFound(id.to_string()),
            _ => AppError::Sqlx(e),
        })
    }
    impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        AppModel,
        AppResult<HashMap<u64, AppModel>>,
        id,
        id,
        "id in ({id}) and status in ({status})",
        status = &[
            AppStatus::Enable as i8,
            AppStatus::Init as i8,
            AppStatus::Disable as i8
        ]
    );
    /// 根据APP client_id 找到对应记录
    pub async fn find_by_client_id(&self, client_id: &str) -> AppResult<AppModel> {
        sqlx::query_as::<_, AppModel>(&sql_format!(
            "select * from {} where client_id={}",
            AppModel::table_name(),
            client_id
        ))
        .fetch_one(&self.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::AppNotFound(client_id.to_owned()),
            _ => AppError::Sqlx(e),
        })
    }
}

#[derive(Default)]
pub struct AppAttrParam {
    //内部功能列表
    pub inner_feature: bool, //ExterLogin OAuthClient SubApp OAuthServer
    //外部功能列表
    pub exter_feature: bool,
    //获取子应用数量
    pub sub_app_count: bool,
    //获取OAUTH登录信息
    pub oauth_client_data: bool,
    //获取OAUTH服务信息
    pub oauth_server_data: bool,
    //上一级APP信息
    pub parent_app: bool,
}

#[derive(Default)]
pub struct AppAttrData {
    pub exter_login: Option<bool>,  //是否启用外部账号登录
    pub oauth_client: Option<bool>, //是否启用OAUTH登录
    pub oauth_client_data: Option<AppOAuthClientModel>, //OAUTH登录信息
    pub sup_app: Option<bool>,      //是否可查看子应用KEY
    pub oauth_server: Option<bool>, //是否启用OAUTH服务
    pub oauth_server_scope_data: Option<Vec<AppOAuthServerScopeModel>>, //OAUTH服务SCOPE设置
    pub exter_feature: Option<Vec<String>>, //外部功能及启用状态
    pub sub_app_count: Option<Vec<(i8, i64)>>, //子APP数量
    pub parent_app: Option<AppModel>, //上一级APP信息
}

impl App {
    async fn attr_app_data(
        &self,
        out_data: Vec<AppModel>,
        app_attr: Option<&AppAttrParam>,
    ) -> AppResult<Vec<(AppModel, AppAttrData)>> {
        let app_attr = match app_attr {
            Some(tmp) => tmp,
            None => {
                return Ok(out_data
                    .into_iter()
                    .map(|e| (e, AppAttrData::default()))
                    .collect::<Vec<_>>());
            }
        };
        let sub_ids = out_data
            .iter()
            .flat_map(|e| {
                if e.parent_app_id == 0 {
                    Some(e.id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let sub_count_data = if !sub_ids.is_empty() && app_attr.sub_app_count {
            sqlx::query_as::<_, (u64, i8, i64)>(&sql_format!(
                "select parent_app_id,status,count(*) as total from {} where 
                parent_app_id in ({}) and status in ({})
                group by parent_app_id,status",
                AppModel::table_name(),
                sub_ids,
                [
                    AppStatus::Enable as i8,
                    AppStatus::Init as i8,
                    AppStatus::Init as i8,
                ]
            ))
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let inn_feature_data = if app_attr.inner_feature {
            let keys = AppRequestType::get_inner_feature()
                .into_iter()
                .map(|e| e.feature_key().to_string())
                .collect::<Vec<_>>();
            if !out_data.is_empty() && !keys.is_empty() {
                sqlx::query_as::<_,(u64,String)>(&sql_format!(
                    "select app_id,feature_key from {} where app_id in ({}) and feature_key  in ({}) and status={}",
                    AppFeatureModel::table_name(),
                    out_data.iter().map(|e|e.id).collect::<Vec<_>>(),
                    keys,
                    AppFeatureStatus::Enable as i8
                )).fetch_all(&self.db).await?
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let ext_feature_data = if app_attr.exter_feature {
            let key = AppRequestType::ExterFeatuer.feature_key();
            let rlen = key.len() + 1;
            sqlx::query_as::<_,(u64,String)>(&sql_format!(
                "select app_id,feature_key from {} where app_id in ({})  and status ={} and feature_key like {}",
                AppFeatureModel::table_name(),
                out_data.iter().map(|e|e.id).collect::<Vec<_>>(),
                AppFeatureStatus::Enable as i8,
                format!("{}%", key)
            ))
            .fetch_all(&self.db).await?
            .into_iter()
            .map(|e|{
                (e.0,e.1[rlen..].to_owned())
            })
            .collect::<Vec<_>>()
        } else {
            vec![]
        };
        let oauth_client_data = if app_attr.oauth_client_data {
            sqlx::query_as::<_, AppOAuthClientModel>(&sql_format!(
                "select * from {} where app_id in ({})",
                AppOAuthClientModel::table_name(),
                out_data.iter().map(|e| e.id).collect::<Vec<_>>(),
            ))
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };
        let oauth_server_scope_data = if app_attr.oauth_server_data {
            sqlx::query_as::<_, AppOAuthServerScopeModel>(&sql_format!(
                "select * from {} where app_id in ({}) and status={}",
                AppOAuthClientModel::table_name(),
                out_data.iter().map(|e| e.id).collect::<Vec<_>>(),
                AppOAuthServerScopeStatus::Enable as i8
            ))
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };
        let pid = out_data
            .iter()
            .flat_map(|e| {
                if e.parent_app_id > 0 {
                    Some(e.parent_app_id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let parent_app_data = if app_attr.parent_app && !pid.is_empty() {
            sqlx::query_as::<_, AppModel>(&sql_format!(
                "select * from {} where id in ({})",
                AppModel::table_name(),
                pid,
            ))
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        Ok(out_data
            .into_iter()
            .map(|e| {
                //初始化值
                let attr = AppAttrData {
                    exter_login: if app_attr.inner_feature {
                        Some(inn_feature_data.iter().any(|t| {
                            t.0 == e.id && AppRequestType::ExterLogin.feature_key() == t.1.as_str()
                        }))
                    } else {
                        None
                    },
                    oauth_client: if app_attr.inner_feature {
                        Some(inn_feature_data.iter().any(|t| {
                            t.0 == e.id && AppRequestType::OAuthClient.feature_key() == t.1.as_str()
                        }))
                    } else {
                        None
                    },
                    sup_app: if app_attr.inner_feature {
                        Some(inn_feature_data.iter().any(|t| {
                            t.0 == e.id && AppRequestType::SubApp.feature_key() == t.1.as_str()
                        }))
                    } else {
                        None
                    },
                    oauth_server: if app_attr.inner_feature {
                        Some(inn_feature_data.iter().any(|t| {
                            t.0 == e.id && AppRequestType::OAuthServer.feature_key() == t.1.as_str()
                        }))
                    } else {
                        None
                    },
                    exter_feature: if app_attr.exter_feature {
                        Some(
                            ext_feature_data
                                .iter()
                                .filter(|mt| mt.0 == e.id)
                                .map(|e| e.1.to_owned())
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        None
                    },
                    sub_app_count: if app_attr.sub_app_count {
                        Some(
                            sub_count_data
                                .iter()
                                .filter(|t| t.0 == e.id)
                                .map(|t| (t.1, t.2))
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        None
                    },
                    oauth_client_data: if app_attr.oauth_client_data {
                        oauth_client_data
                            .iter()
                            .find(|t| t.app_id == e.id)
                            .map(|t| t.to_owned())
                    } else {
                        None
                    },
                    oauth_server_scope_data: if app_attr.oauth_server_data {
                        Some(
                            oauth_server_scope_data
                                .iter()
                                .filter(|t| t.app_id == e.id)
                                .map(|t| t.to_owned())
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        None
                    },
                    parent_app: if app_attr.parent_app {
                        parent_app_data
                            .iter()
                            .find(|t| t.id == e.parent_app_id)
                            .map(|t| t.to_owned())
                    } else {
                        None
                    },
                };
                (e, attr)
            })
            .collect::<Vec<_>>())
    }
}

#[derive(Clone, Debug)]
pub struct SystemAppParam<'t> {
    pub user_id: Option<u64>,
    pub status: Option<AppStatus>,
    pub client_id: Option<&'t str>,
    pub app_name: Option<&'t str>,
    pub app_id: Option<u64>,
}

impl App {
    fn system_app_data_sql(&self, app_where: &SystemAppParam) -> Option<Vec<String>> {
        let mut sql_vec = vec!["parent_app_id=0".to_string()];
        if let Some(ref tmp) = app_where.user_id {
            sql_vec.push(sql_format!("user_id = {}", tmp));
        };
        if let Some(tmp) = app_where.app_name {
            sql_vec.push(sql_format!("name like {}", format!("%{}%", tmp)));
        }
        if let Some(ref tmp) = app_where.status {
            sql_vec.push(sql_format!("status = {}", *tmp as i8));
        }
        if let Some(ref tmp) = app_where.client_id {
            sql_vec.push(sql_format!("client_id = {}", tmp));
        };
        if let Some(ref tmp) = app_where.app_id {
            sql_vec.push(sql_format!("id = {}", tmp));
        }
        Some(sql_vec)
    }
    //系统APP的数据
    pub async fn system_app_data(
        &self,
        app_where: &SystemAppParam<'_>,
        app_attr: Option<&AppAttrParam>,
        page: Option<&PageParam>,
    ) -> AppResult<Vec<(AppModel, AppAttrData)>> {
        let where_sql = match self.system_app_data_sql(app_where) {
            Some(sql) => sql,
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
        let out_data = sqlx::query_as::<_, AppModel>(&sql_format!(
            "select * from {} where {} {}",
            AppModel::table_name(),
            where_sql.join(","),
            if page.is_some() {
                SqlExpr(page_sql)
            } else {
                SqlExpr("".to_string())
            }
        ))
        .fetch_all(&self.db)
        .await?;
        self.attr_app_data(out_data, app_attr).await
    }
    //系统APP的数量
    pub async fn system_app_count(&self, app_where: &SystemAppParam<'_>) -> AppResult<i64> {
        let where_sql = match self.system_app_data_sql(app_where) {
            Some(sql) => sql,
            None => return Ok(0),
        };
        let sql = sql_format!(
            "select  count(*) as total from {} where {}",
            AppModel::table_name(),
            where_sql.join(",")
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
}

#[derive(Clone, Debug)]
pub struct UserAppDataParam<'t> {
    pub parent_app_id: Option<u64>,
    pub status: Option<AppStatus>,
    pub client_id: Option<&'t str>,
    pub app_id: Option<u64>,
}

impl App {
    fn user_app_data_sql(&self, user_id: u64, app_where: &UserAppDataParam) -> Option<Vec<String>> {
        let mut sql_vec = vec![sql_format!("user_id={}", user_id)];
        if let Some(ref rid) = app_where.parent_app_id {
            sql_vec.push(sql_format!("parent_app_id = {}", rid));
        };
        if let Some(ref tmp) = app_where.status {
            sql_vec.push(sql_format!("status = {}", *tmp as i8));
        }
        if let Some(ref tmp) = app_where.client_id {
            sql_vec.push(sql_format!("client_id = {}", tmp));
        };
        if let Some(ref tmp) = app_where.app_id {
            sql_vec.push(sql_format!("id = {}", tmp));
        }
        Some(sql_vec)
    }
    //指定用户APP的数据
    pub async fn user_app_data(
        &self,
        user_id: u64,
        app_where: &UserAppDataParam<'_>,
        app_attr: Option<&AppAttrParam>,
        page: Option<&PageParam>,
    ) -> AppResult<Vec<(AppModel, AppAttrData)>> {
        let where_sql = match self.user_app_data_sql(user_id, app_where) {
            Some(sql) => sql,
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
        let out_data = sqlx::query_as::<_, AppModel>(&sql_format!(
            "select * from {} where {} {}",
            AppModel::table_name(),
            where_sql.join(","),
            if page.is_some() {
                SqlExpr(page_sql)
            } else {
                SqlExpr("".to_string())
            }
        ))
        .fetch_all(&self.db)
        .await?;
        self.attr_app_data(out_data, app_attr).await
    }
    //指定用户APP的数量
    pub async fn user_app_count(
        &self,
        user_id: u64,
        app_where: &UserAppDataParam<'_>,
    ) -> AppResult<i64> {
        let where_sql = match self.user_app_data_sql(user_id, app_where) {
            Some(sql) => sql,
            None => return Ok(0),
        };
        let sql = sql_format!(
            "select count(*) as total from {} where {}",
            AppModel::table_name(),
            where_sql.join(",")
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
}

#[derive(Clone, Debug)]
pub struct UserParentAppDataParam<'t> {
    pub key_word: Option<&'t str>,
}

impl App {
    fn user_parent_app_data_sql(&self, app_where: &UserParentAppDataParam) -> Option<Vec<String>> {
        let mut sql_vec = vec![sql_format!(
            "status={} AND parent_app_id=0 and user_app_id=0 and id in (
                select app_id from {} where status={} and feature_key ={}
            )",
            AppStatus::Enable as i8,
            AppFeatureModel::table_name(),
            AppFeatureStatus::Enable as i8,
            AppRequestType::SubApp.feature_key()
        )];
        if let Some(tmp) = app_where.key_word {
            let key_word = if let Ok(re) = Regex::new(r"[\s%_&\*\.\^\!@#\$'\t\n\r]+") {
                re.replace_all(tmp, " ").trim().to_string()
            } else {
                tmp.trim().to_owned()
            };
            sql_vec.push(sql_format!(
                "( client_id = {} or name={} )",
                key_word,
                format!("%{}%", key_word)
            ));
        };
        Some(sql_vec)
    }
    //指定用户APP的数据
    pub async fn user_parent_app_data(
        &self,
        app_where: &UserParentAppDataParam<'_>,
        page: Option<&PageParam>,
    ) -> AppResult<Vec<AppModel>> {
        let where_sql = match self.user_parent_app_data_sql(app_where) {
            Some(sql) => sql,
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
        let out_data = sqlx::query_as::<_, AppModel>(&sql_format!(
            "select * from {} where {} {}",
            AppModel::table_name(),
            where_sql.join(","),
            if page.is_some() {
                SqlExpr(page_sql)
            } else {
                SqlExpr("".to_string())
            }
        ))
        .fetch_all(&self.db)
        .await?;
        Ok(out_data)
    }
    //指定用户APP的数量
    pub async fn user_parent_app_count(
        &self,
        app_where: &UserParentAppDataParam<'_>,
    ) -> AppResult<i64> {
        let where_sql = match self.user_parent_app_data_sql(app_where) {
            Some(sql) => sql,
            None => return Ok(0),
        };
        let sql = sql_format!(
            "select count (*) as total from {} where {}",
            AppModel::table_name(),
            where_sql.join(",")
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
}
