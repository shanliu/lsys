use std::collections::HashMap;

use crate::dao::AppSecretRecord;
use crate::dao::logger::AppViewSecretLog;
use crate::model::{
    AppFeatureModel, AppFeatureStatus, AppModel, AppOAuthClientModel, AppOAuthServerScopeModel,
    AppOAuthServerScopeStatus, AppRequestModel, AppRequestStatus, AppRequestType, AppSecretType,
    AppStatus,
};

use lsys_core::db::TableMeta;
use lsys_core::db::{OffsetPageParam, QueryBuilderExt, WhereClause};
use lsys_core::utils::{RequestEnv, STRING_CLEAR_FORMAT, StringClear, string_clear};
use lsys_core::valid_param::{ValidParam, ValidParamCheck, ValidPattern, ValidStrlen};
use lsys_core::{db::utils::FetchField, valid_key};
use sqlx::{MySql, QueryBuilder};

use super::super::{AppError, AppResult};
use super::App;

impl App {
    /// 根据APP id 找到对应记录
    pub async fn find_by_id(&self, id: u64) -> AppResult<AppModel> {
        sqlx::query_as::<_, AppModel>(&format!(
            "select * from {} where id=?",
            AppModel::table_name(),
        ))
        .bind(id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::AppNotFound(id.to_string()),
            _ => AppError::Sqlx(e),
        })
    }
    pub async fn find_by_ids(&self, ids: &[u64]) -> AppResult<HashMap<u64, AppModel>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AppModel>::map(
            &self.db,
            |qb| {
                qb.field_in_copied("id", ids);
                qb.push_and().field_in_copied(
                    "status",
                    &[
                        AppStatus::Enable as i8,
                        AppStatus::Init as i8,
                        AppStatus::Disable as i8,
                    ],
                );
            },
            |v| v.id,
        )
        .await?)
    }
    async fn find_by_client_id_param_valid(&self, client_id: &str) -> AppResult<()> {
        let client_id_max = FetchField::new(&self.db)
            .string_max::<AppModel>(&AppModel::CLIENT_ID)
            .await
            .len_or(32);

        ValidParam::default()
            .add(
                valid_key!("client_id"),
                &client_id,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(3, client_id_max))
                    .add_rule(ValidPattern::Ident),
            )
            .check()?;
        Ok(())
    }
    /// 根据APP client_id 找到对应记录
    pub async fn find_by_client_id(&self, client_id: &str) -> AppResult<AppModel> {
        self.find_by_client_id_param_valid(client_id).await?;
        sqlx::query_as::<_, AppModel>(&format!(
            "select * from {} where client_id=?",
            AppModel::table_name(),
        ))
        .bind(client_id)
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
    //获取该应用的请求数量
    pub req_pending_count: bool,
    //获取该应用的子应用请求数量
    pub sub_req_pending_count: bool,
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
    pub req_pending_count: Option<i64>, //当前应用请求数量
    pub sub_req_pending_count: Option<i64>, //当前应用的子应用请求汇总
}

impl App {
    async fn attr_app_info(
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
            let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select parent_app_id,status,count(*) as total from {}",
                AppModel::table_name()
            ));
            qb.push_where().field_in_copied("parent_app_id", &sub_ids);
            qb.push_and()
                .field_in_copied("status", &[AppStatus::Enable as i8, AppStatus::Init as i8]);
            qb.push(" group by parent_app_id,status");
            qb.build_query_as::<(u64, i8, i64)>()
                .fetch_all(&self.db)
                .await?
        } else {
            vec![]
        };
        let req_pending_data = if !sub_ids.is_empty() && app_attr.req_pending_count {
            let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select app_id,status,count(*) as total from {}",
                AppRequestModel::table_name()
            ));
            qb.push_where().field_in_copied("app_id", &sub_ids);
            qb.push_and()
                .field_eq("status", AppRequestStatus::Pending as i8);
            qb.push(" group by app_id,status");
            qb.build_query_as::<(u64, i64)>()
                .fetch_all(&self.db)
                .await?
        } else {
            vec![]
        };
        let sub_req_pending_data = if !sub_ids.is_empty() && app_attr.sub_req_pending_count {
            let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select parent_app_id,status,count(*) as total from {}",
                AppRequestModel::table_name()
            ));
            qb.push_where().field_in_copied("parent_app_id", &sub_ids);
            qb.push_and()
                .field_eq("status", AppRequestStatus::Pending as i8);
            qb.push(" group by parent_app_id,status");
            qb.build_query_as::<(u64, i64)>()
                .fetch_all(&self.db)
                .await?
        } else {
            vec![]
        };

        let inn_feature_data = if app_attr.inner_feature && !out_data.is_empty() {
            let keys = AppRequestType::get_inner_feature()
                .into_iter()
                .map(|e| e.feature_key().to_string())
                .collect::<Vec<_>>();
            if !keys.is_empty() {
                let app_ids: Vec<u64> = out_data.iter().map(|e| e.id).collect();
                let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                    "select app_id,feature_key from {}",
                    AppFeatureModel::table_name()
                ));
                qb.push_where().field_in_copied("app_id", &app_ids);
                qb.push_and().field_in_string("feature_key", &keys);
                qb.push_and()
                    .field_eq("status", AppFeatureStatus::Enable as i8);
                qb.build_query_as::<(u64, String)>()
                    .fetch_all(&self.db)
                    .await?
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let ext_feature_data = if app_attr.exter_feature && !out_data.is_empty() {
            let key = AppRequestType::ExterFeatuer.feature_key();
            let rlen = key.len() + 1;
            let app_ids: Vec<u64> = out_data.iter().map(|e| e.id).collect();
            let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select app_id,feature_key from {}",
                AppFeatureModel::table_name()
            ));
            qb.push_where().field_in_copied("app_id", &app_ids);
            qb.push_and()
                .field_eq("status", AppFeatureStatus::Enable as i8);
            qb.push_and().field_like("feature_key", format!("{}%", key));
            qb.build_query_as::<(u64, String)>()
                .fetch_all(&self.db)
                .await?
                .into_iter()
                .map(|e| (e.0, e.1[rlen..].to_owned()))
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        let oauth_client_data = if app_attr.oauth_client_data && !out_data.is_empty() {
            let app_ids: Vec<u64> = out_data.iter().map(|e| e.id).collect();
            let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select * from {}",
                AppOAuthClientModel::table_name()
            ));
            qb.push_where().field_in_copied("app_id", &app_ids);
            qb.build_query_as::<AppOAuthClientModel>()
                .fetch_all(&self.db)
                .await?
        } else {
            vec![]
        };
        let oauth_server_scope_data = if app_attr.oauth_server_data && !out_data.is_empty() {
            let app_ids: Vec<u64> = out_data.iter().map(|e| e.id).collect();
            let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select * from {}",
                AppOAuthServerScopeModel::table_name()
            ));
            qb.push_where().field_in_copied("app_id", &app_ids);
            qb.push_and()
                .field_eq("status", AppOAuthServerScopeStatus::Enable as i8);
            qb.build_query_as::<AppOAuthServerScopeModel>()
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
            let mut qb: QueryBuilder<'_, MySql> =
                QueryBuilder::new(format!("select * from {}", AppModel::table_name()));
            qb.push_where().field_in_copied("id", &pid);
            qb.build_query_as::<AppModel>().fetch_all(&self.db).await?
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
                    req_pending_count: if app_attr.req_pending_count {
                        Some(
                            req_pending_data
                                .iter()
                                .find(|t| t.0 == e.id)
                                .map(|t| t.1)
                                .unwrap_or(0),
                        )
                    } else {
                        None
                    },
                    sub_req_pending_count: if app_attr.sub_req_pending_count {
                        Some(
                            sub_req_pending_data
                                .iter()
                                .find(|t| t.0 == e.id)
                                .map(|t| t.1)
                                .unwrap_or(0),
                        )
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
    fn system_app_data_where(
        &self,
        wc: &mut WhereClause<'_, '_, MySql>,
        app_where: &SystemAppParam,
    ) -> Option<bool> {
        wc.and().field_eq("parent_app_id", 0u64);
        if let Some(ref tmp) = app_where.user_id {
            wc.and().field_eq("user_id", *tmp);
        };
        if let Some(tmp) = app_where.app_name {
            let tmp = string_clear(tmp, StringClear::LikeKeyWord, Some(255));
            if tmp.is_empty() {
                return None;
            }
            wc.and().field_like("name", format!("%{}%", tmp));
        }
        if let Some(ref tmp) = app_where.status {
            wc.and().field_eq("status", *tmp as i8);
        }
        if let Some(ref tmp) = app_where.client_id {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(64));
            if tmp.is_empty() {
                return None;
            }
            wc.and().field_eq("client_id", tmp);
        };
        if let Some(ref tmp) = app_where.app_id {
            wc.and().field_eq("id", *tmp);
        }
        Some(true)
    }
    //系统APP的数据
    pub async fn system_app_info(
        &self,
        app_where: &SystemAppParam<'_>,
        app_attr: Option<&AppAttrParam>,
        page: &OffsetPageParam,
    ) -> AppResult<Vec<(AppModel, AppAttrData)>> {
        let out_data = self.system_app_data(app_where, page).await?;
        self.attr_app_info(out_data, app_attr).await
    }
    pub async fn system_app_data(
        &self,
        app_where: &SystemAppParam<'_>,
        page: &OffsetPageParam,
    ) -> AppResult<Vec<AppModel>> {
        let mut qb =
            QueryBuilder::<MySql>::new(format!("select * from {}", AppModel::table_name()));
        let mut wc = WhereClause::new(&mut qb);
        if self.system_app_data_where(&mut wc, app_where).is_none() {
            return Ok(vec![]);
        }
        wc.builder().push(" order by id desc");
        page.push_limit(wc.builder());
        let out_data = wc
            .builder()
            .build_query_as::<AppModel>()
            .fetch_all(&self.db)
            .await?;
        Ok(out_data)
    }
    //系统APP的数量
    pub async fn system_app_count(&self, app_where: &SystemAppParam<'_>) -> AppResult<i64> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select count(*) as total from {}",
            AppModel::table_name()
        ));
        let mut wc = WhereClause::new(&mut qb);
        if self.system_app_data_where(&mut wc, app_where).is_none() {
            return Ok(0);
        }
        let res = wc
            .builder()
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?;
        Ok(res)
    }
}

#[derive(Clone, Debug)]
pub struct SystemSubAppParam<'t> {
    pub status: Option<AppStatus>,
    pub client_id: Option<&'t str>,
    pub app_id: u64,
}

impl App {
    fn system_sub_app_data_where(
        &self,
        wc: &mut WhereClause<'_, '_, MySql>,
        app_where: &SystemSubAppParam,
    ) -> Option<bool> {
        wc.and().field_eq("parent_app_id", app_where.app_id);
        if let Some(ref tmp) = app_where.status {
            wc.and().field_eq("status", *tmp as i8);
        }
        if let Some(ref tmp) = app_where.client_id {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(64));
            if tmp.is_empty() {
                return None;
            }
            wc.and().field_eq("client_id", tmp);
        };
        Some(true)
    }

    //系统APP的数据

    pub async fn system_sub_app_info(
        &self,
        app_where: &SystemSubAppParam<'_>,
        app_attr: Option<&AppAttrParam>,
        page: &OffsetPageParam,
    ) -> AppResult<Vec<(AppModel, AppAttrData)>> {
        let out_data = self.system_sub_app_data(app_where, page).await?;
        self.attr_app_info(out_data, app_attr).await
    }
    pub async fn system_sub_app_data(
        &self,
        app_where: &SystemSubAppParam<'_>,
        page: &OffsetPageParam,
    ) -> AppResult<Vec<AppModel>> {
        let mut qb =
            QueryBuilder::<MySql>::new(format!("select * from {}", AppModel::table_name()));
        let mut wc = WhereClause::new(&mut qb);
        if self.system_sub_app_data_where(&mut wc, app_where).is_none() {
            return Ok(vec![]);
        }
        wc.builder().push(" order by id desc");
        page.push_limit(wc.builder());
        let out_data = wc
            .builder()
            .build_query_as::<AppModel>()
            .fetch_all(&self.db)
            .await?;
        Ok(out_data)
    }
    //系统APP的数量
    pub async fn system_sub_app_count(&self, app_where: &SystemSubAppParam<'_>) -> AppResult<i64> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select count(*) as total from {}",
            AppModel::table_name()
        ));
        let mut wc = WhereClause::new(&mut qb);
        if self.system_sub_app_data_where(&mut wc, app_where).is_none() {
            return Ok(0);
        }
        let res = wc
            .builder()
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?;
        Ok(res)
    }
}

#[derive(Clone, Debug)]
pub struct UserAppDataParam<'t> {
    pub parent_app_id: Option<u64>,
    pub status: Option<AppStatus>,
    pub client_id: Option<&'t str>,
    pub like_client_id: Option<&'t str>,
    pub app_id: Option<u64>,
}

impl App {
    fn user_app_data_where(
        &self,
        wc: &mut WhereClause<'_, '_, MySql>,
        user_id: u64,
        app_where: &UserAppDataParam,
    ) -> Option<bool> {
        wc.and().field_eq("user_id", user_id);
        if let Some(ref rid) = app_where.parent_app_id {
            wc.and().field_eq("parent_app_id", *rid);
        };
        if let Some(ref tmp) = app_where.status {
            wc.and().field_eq("status", *tmp as i8);
        }
        if let Some(ref tmp) = app_where.client_id {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(64));
            if tmp.is_empty() {
                return None;
            }
            wc.and().field_eq("client_id", tmp);
        };
        if let Some(ref tmp) = app_where.like_client_id {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(64));
            if tmp.is_empty() {
                return None;
            }
            wc.and().field_like("client_id", format!("{}%", tmp));
        };
        if let Some(ref tmp) = app_where.app_id {
            wc.and().field_eq("id", *tmp);
        }
        Some(true)
    }
    //指定用户APP的数据
    pub async fn user_app_info(
        &self,
        user_id: u64,
        app_where: &UserAppDataParam<'_>,
        app_attr: Option<&AppAttrParam>,
        page: &OffsetPageParam,
    ) -> AppResult<Vec<(AppModel, AppAttrData)>> {
        let out_data = self.user_app_data(user_id, app_where, page).await?;
        self.attr_app_info(out_data, app_attr).await
    }
    pub async fn user_app_data(
        &self,
        user_id: u64,
        app_where: &UserAppDataParam<'_>,
        page: &OffsetPageParam,
    ) -> AppResult<Vec<AppModel>> {
        let mut qb =
            QueryBuilder::<MySql>::new(format!("select * from {}", AppModel::table_name()));
        let mut wc = WhereClause::new(&mut qb);
        if self
            .user_app_data_where(&mut wc, user_id, app_where)
            .is_none()
        {
            return Ok(vec![]);
        }
        wc.builder().push(" order by id desc");
        page.push_limit(wc.builder());
        let out_data = wc
            .builder()
            .build_query_as::<AppModel>()
            .fetch_all(&self.db)
            .await?;
        Ok(out_data)
    }
    //指定用户APP的数量
    pub async fn user_app_count(
        &self,
        user_id: u64,
        app_where: &UserAppDataParam<'_>,
    ) -> AppResult<i64> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select count(*) as total from {}",
            AppModel::table_name()
        ));
        let mut wc = WhereClause::new(&mut qb);
        if self
            .user_app_data_where(&mut wc, user_id, app_where)
            .is_none()
        {
            return Ok(0);
        }
        let res = wc
            .builder()
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?;
        Ok(res)
    }
}

#[derive(Clone, Debug)]
pub struct UserSubAppParam {
    pub status: Option<AppStatus>,
    pub app_id: u64,
    pub sub_app_id: Option<u64>,
}

impl App {
    fn user_sub_app_data_where(
        &self,
        wc: &mut WhereClause<'_, '_, MySql>,
        app_where: &UserSubAppParam,
    ) -> Option<bool> {
        wc.and().field_eq("parent_app_id", app_where.app_id);
        if let Some(ref tmp) = app_where.status {
            wc.and().field_eq("status", *tmp as i8);
        }
        if let Some(ref tmp) = app_where.sub_app_id {
            wc.and().field_eq("id", *tmp);
        }
        Some(true)
    }
    //用户指定APP的子应用数据
    pub async fn user_sub_app_info(
        &self,
        app_where: &UserSubAppParam,
        app_attr: Option<&AppAttrParam>,
        page: &OffsetPageParam,
    ) -> AppResult<Vec<(AppModel, AppAttrData)>> {
        let out_data = self.user_sub_app_data(app_where, page).await?;
        self.attr_app_info(out_data, app_attr).await
    }
    pub async fn user_sub_app_data(
        &self,
        app_where: &UserSubAppParam,
        page: &OffsetPageParam,
    ) -> AppResult<Vec<AppModel>> {
        let mut qb =
            QueryBuilder::<MySql>::new(format!("select * from {}", AppModel::table_name()));
        let mut wc = WhereClause::new(&mut qb);
        if self.user_sub_app_data_where(&mut wc, app_where).is_none() {
            return Ok(vec![]);
        }
        wc.builder().push(" order by id desc");
        page.push_limit(wc.builder());
        let out_data = wc
            .builder()
            .build_query_as::<AppModel>()
            .fetch_all(&self.db)
            .await?;
        Ok(out_data)
    }
    //用户指定APP的子应用数量
    pub async fn user_sub_app_count(&self, app_where: &UserSubAppParam) -> AppResult<i64> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select count(*) as total from {}",
            AppModel::table_name()
        ));
        let mut wc = WhereClause::new(&mut qb);
        if self.user_sub_app_data_where(&mut wc, app_where).is_none() {
            return Ok(0);
        }
        let res = wc
            .builder()
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?;
        Ok(res)
    }
}

#[derive(Clone, Debug)]
pub struct UserParentAppDataParam<'t> {
    pub key_word: Option<&'t str>,
}

impl App {
    fn user_parent_app_data_where(
        &self,
        wc: &mut WhereClause<'_, '_, MySql>,
        app_where: &UserParentAppDataParam,
    ) -> Option<bool> {
        wc.and().field_eq("status", AppStatus::Enable as i8);
        wc.and().push(format!(
            "parent_app_id=0 and user_app_id=0 and id in (select app_id from {}",
            AppFeatureModel::table_name()
        ));
        wc.builder()
            .push_where()
            .field_eq("status", AppFeatureStatus::Enable as i8);
        wc.builder().push_and().field_eq(
            "feature_key",
            AppRequestType::SubApp.feature_key().to_string(),
        );
        wc.builder().push(")");
        if let Some(tmp) = app_where.key_word {
            let key_word = string_clear(tmp, StringClear::LikeKeyWord, Some(255));
            if key_word.is_empty() {
                return None;
            }
            wc.and().push("(");
            wc.builder().field_eq("client_id", key_word.clone());
            wc.builder()
                .push_or()
                .field_like("name", format!("%{}%", key_word));
            wc.builder().push(")");
        };
        Some(true)
    }
    //用户可用父APP列表
    pub async fn user_parent_app_data(
        &self,
        app_where: &UserParentAppDataParam<'_>,
        page: &OffsetPageParam,
    ) -> AppResult<Vec<AppModel>> {
        let mut qb =
            QueryBuilder::<MySql>::new(format!("select * from {}", AppModel::table_name()));
        let mut wc = WhereClause::new(&mut qb);
        if self
            .user_parent_app_data_where(&mut wc, app_where)
            .is_none()
        {
            return Ok(vec![]);
        }
        wc.builder().push(" order by id desc");
        page.push_limit(wc.builder());
        let out_data = wc
            .builder()
            .build_query_as::<AppModel>()
            .fetch_all(&self.db)
            .await?;
        Ok(out_data)
    }
    //用户可用父APP列表
    pub async fn user_parent_app_count(
        &self,
        app_where: &UserParentAppDataParam<'_>,
    ) -> AppResult<i64> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select count(*) as total from {}",
            AppModel::table_name()
        ));
        let mut wc = WhereClause::new(&mut qb);
        if self
            .user_parent_app_data_where(&mut wc, app_where)
            .is_none()
        {
            return Ok(0);
        }
        let res = wc
            .builder()
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await?;
        Ok(res)
    }
}

impl App {
    //查看secret
    pub async fn view_app_secret(
        &self,
        app: &AppModel,
        view_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<Vec<AppSecretRecord>> {
        let app_secret = self
            .app_secret
            .multiple_find_secret_by_app_id(app.id, AppSecretType::App)
            .await?;
        self.logger
            .add(
                &AppViewSecretLog {
                    action: "view_secret",
                    app_id: app.id,
                    user_id: app.user_id,
                    app_name: &app.name,
                    secret_data: &app_secret
                        .iter()
                        .map(|e| e.secret_data.as_str())
                        .collect::<Vec<_>>()
                        .join(",")
                        .to_string(),
                },
                Some(app.id),
                Some(view_user_id),
                None,
                env_data,
            )
            .await;
        Ok(app_secret)
    }
    //查看secret
    pub async fn view_notify_secret(
        &self,
        app: &AppModel,
        view_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<AppSecretRecord> {
        let notify_secret = self
            .app_secret
            .single_find_secret_app_id(app.id, AppSecretType::Notify)
            .await?;
        self.logger
            .add(
                &AppViewSecretLog {
                    action: "view_secret",
                    app_id: app.id,
                    user_id: app.user_id,
                    app_name: &app.name,
                    secret_data: &notify_secret.secret_data.to_string(),
                },
                Some(app.id),
                Some(view_user_id),
                None,
                env_data,
            )
            .await;
        Ok(notify_secret)
    }
}
