mod access;
mod cache;
mod data;
mod login;
use super::logger::{AppOAuthClientSecretSetLog, AppOAuthClientSetDomainLog, AppRequestLog};
use super::{App, AppResult, AppSecret};
use super::{AppError, AppOAuthServer};
use crate::dao::oauth_client::access::AppOAuthClientAccess;
use crate::model::AppModel;
use crate::model::AppRequestModel;
use crate::model::AppRequestType;
use crate::model::{
    AppFeatureModel, AppFeatureModelRef, AppOAuthClientModel, AppOAuthClientModelRef,
    AppRequestModelRef, AppRequestOAuthClientModel, AppRequestOAuthClientModelRef,
    AppRequestStatus,
};
use crate::model::{AppFeatureStatus, AppSecretType};
pub use login::*;
use lsys_access::dao::AccessDao;
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::db::SqlQuote;
use lsys_core::db::{Insert, Update};
use lsys_core::db::{ModelTableName, WhereOption};
use lsys_core::{
    fluent_message, string_clear, valid_key, RemoteNotify, StringClear, ValidDomain,
    ValidParamCheck, STRING_CLEAR_FORMAT, STRING_CLEAR_XSS,
};
use lsys_core::{model_option_set, sql_format};
use lsys_core::{now_time, ValidParam};
use lsys_core::{rand_str, RequestEnv};
use lsys_logger::dao::ChangeLoggerDao;
// use regex::Regex;
use sqlx::{MySql, Pool};
use std::sync::Arc;
pub struct AppOAuthClient {
    db: Pool<MySql>,
    app: Arc<App>,
    oauth_server: Arc<AppOAuthServer>,
    access: Arc<AccessDao>,
    oauth_access: AppOAuthClientAccess,
    logger: Arc<ChangeLoggerDao>,
    code_time: u64,
    login_time: u64,
    refresh_time: u64,
    app_secret: Arc<AppSecret>,
    pub(crate) oauth_client_cache: Arc<LocalCache<u64, AppOAuthClientModel>>,
}

pub struct AppOAuthClientConfig {
    pub cache_config: LocalCacheConfig,
    pub code_time: u64,
    pub login_time: u64,
    pub refresh_time: u64,
}

impl AppOAuthClient {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        app: Arc<App>,
        oauth_server: Arc<AppOAuthServer>,
        access: Arc<AccessDao>,
        logger: Arc<ChangeLoggerDao>,
        remote_notify: Arc<RemoteNotify>,
        app_secret: Arc<AppSecret>,
        config: AppOAuthClientConfig,
    ) -> Self {
        let oauth_access = AppOAuthClientAccess::new(access.auth.clone(), redis);
        Self {
            db,
            app,
            oauth_server,
            access,
            code_time: config.code_time,
            login_time: config.login_time,
            refresh_time: if config.refresh_time < config.login_time {
                config.login_time
            } else {
                config.refresh_time
            },
            logger,
            app_secret,
            oauth_access,
            oauth_client_cache: Arc::new(LocalCache::new(
                remote_notify.clone(),
                config.cache_config,
            )),
        }
    }
    //检测当前请求的SOPCE是否在父应用的SCOPE中
    async fn server_scope_check(&self, app: &AppModel, scope_data: &[&str]) -> AppResult<()> {
        if app.parent_app_id > 0 {
            //上级app需要开通 OAuthServer 权限
            let papp = self.app.find_by_id(app.parent_app_id).await?;
            self.oauth_server.check_scope(&papp, scope_data).await?;
        }
        Ok(())
    }
    //获取指定APP的已授权的SCOPE DATA 数据
    //不要在外部使用,外部用 cache find_by_app 得到AppOAuthClientModel后获取 scope_data
    async fn get_oauth_client_scope_data(&self, app: &AppModel) -> AppResult<String> {
        let scope_res = sqlx::query_scalar::<_, String>(&sql_format!(
            "select scope_data from {} where app_id={}",
            AppOAuthClientModel::table_name(),
            app.id,
        ))
        .fetch_one(&self.db)
        .await;
        let scope_data = match scope_res {
            Ok(scope) => scope,
            Err(sqlx::Error::RowNotFound) => "".to_string(),
            Err(err) => {
                return Err(err.into());
            }
        };
        Ok(scope_data)
    }
    //发起OAUTH接入申请
    pub async fn oauth_request(
        &self,
        app: &AppModel,
        scope_data: &[&str],
        req_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        app.app_status_check()?;

        self.server_scope_check(app, scope_data).await?;

        let req_res = sqlx::query_scalar::<_, i8>(&sql_format!(
            "select status from {} where app_id={} and feature_key={}",
            AppFeatureModel::table_name(),
            app.id,
            AppRequestType::OAuthClient.feature_key(),
        ))
        .fetch_one(&self.db)
        .await;
        match req_res {
            Ok(status) => {
                if AppFeatureStatus::Enable.eq(status) {
                    let scope_inner = self.get_oauth_client_scope_data(app).await?;
                    let tmp_scope = scope_inner.split(",").collect::<Vec<&str>>();
                    let mut add_scope = vec![];
                    for sd in scope_data {
                        let ssd = sd.trim();
                        if !ssd.is_empty() && !tmp_scope.contains(&ssd) {
                            add_scope.push(ssd);
                        }
                    }
                    if !add_scope.is_empty() {
                        self.scope_request_create(app, &add_scope, req_user_id, env_data)
                            .await?;
                    }
                    return Ok(());
                }
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }
        let time = now_time()?;
        let mut db = self.db.begin().await?;

        let req_status = AppRequestStatus::Pending as i8;
        let request_type = AppRequestType::OAuthClient as i8;
        let idata = model_option_set!(AppRequestModelRef,{
            parent_app_id:app.parent_app_id,
            app_id:app.id,
            request_type:request_type,
            status:req_status,
            request_user_id:req_user_id,
            request_time:time,
        });
        let req_res = Insert::<AppRequestModel, _>::new(idata)
            .execute(&mut *db)
            .await;
        let req_id = match req_res {
            Err(e) => {
                db.rollback().await?;
                return Err(e.into());
            }
            Ok(mr) => mr.last_insert_id(),
        };
        let scope_data = scope_data
            .iter()
            .map(|e| e.trim())
            .collect::<Vec<&str>>()
            .join(",");
        let idata = model_option_set!(AppRequestOAuthClientModelRef,{
            app_request_id:req_id,
            scope_data:scope_data,
        });
        let req_res = Insert::<AppRequestOAuthClientModel, _>::new(idata)
            .execute(&mut *db)
            .await;
        if let Err(err) = req_res {
            db.rollback().await?;
            return Err(err.into());
        }
        db.commit().await?;

        self.logger
            .add(
                &AppRequestLog {
                    action: "oauth_request",
                    parent_app_id: app.parent_app_id,
                    app_id: app.id,
                    user_id: app.user_id,
                    request_type,
                    status: req_status,
                    req_data: Some(&scope_data),
                },
                Some(req_id),
                Some(req_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    async fn scope_request_create(
        &self,
        app: &AppModel,
        scope_data: &[&str],
        req_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        let time = now_time()?;
        let mut db = self.db.begin().await?;

        let req_status = AppRequestStatus::Pending as i8;
        let request_type = AppRequestType::OAuthClientScope as i8;
        let idata = model_option_set!(AppRequestModelRef,{
            parent_app_id:app.parent_app_id,
            app_id:app.id,
            request_type:request_type,
            status:req_status,
            request_user_id:req_user_id,
            request_time:time,
        });
        let req_res = Insert::<AppRequestModel, _>::new(idata)
            .execute(&mut *db)
            .await;
        let req_id = match req_res {
            Err(e) => {
                db.rollback().await?;
                return Err(e.into());
            }
            Ok(mr) => mr.last_insert_id(),
        };
        let scope_data = scope_data
            .iter()
            .map(|e| e.trim())
            .collect::<Vec<&str>>()
            .join(",");
        let idata = model_option_set!(AppRequestOAuthClientModelRef,{
            app_request_id:req_id,
            scope_data:scope_data,
        });
        let req_res = Insert::<AppRequestOAuthClientModel, _>::new(idata)
            .execute(&mut *db)
            .await;
        if let Err(err) = req_res {
            db.rollback().await?;
            return Err(err.into());
        }
        db.commit().await?;

        self.logger
            .add(
                &AppRequestLog {
                    action: "scope_request",
                    parent_app_id: app.parent_app_id,
                    app_id: app.id,
                    user_id: app.user_id,
                    request_type,
                    status: req_status,
                    req_data: Some(&scope_data),
                },
                Some(req_id),
                Some(req_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    //OAUTH登录申请SCOPE
    pub async fn scope_request(
        &self,
        app: &AppModel,
        scope_data: &[&str],
        req_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        self.oauth_check(app).await?;
        self.server_scope_check(app, scope_data).await?;

        let mut add_scope = vec![];
        let oc_res = sqlx::query_scalar::<_, String>(&sql_format!(
            "select scope_data from {} where app_id={}",
            AppOAuthClientModel::table_name(),
            app.id,
        ))
        .fetch_one(&self.db)
        .await;
        let scope_inner = match oc_res {
            Ok(sc) => sc,
            Err(sqlx::Error::RowNotFound) => "".to_string(),
            Err(err) => {
                return Err(err.into());
            }
        };
        let tmp_scope = scope_inner.split(",").collect::<Vec<&str>>();
        for sd in scope_data {
            let ssd = sd.trim();
            if !ssd.is_empty() && !tmp_scope.contains(&ssd) {
                add_scope.push(ssd);
            }
        }
        if add_scope.is_empty() {
            return Ok(());
        }
        self.scope_request_create(app, &add_scope, req_user_id, env_data)
            .await
    }
    //OAUTH登录申请确认
    pub async fn oauth_confirm(
        &self,
        app: &AppModel,
        req_status: AppRequestStatus,
        confirm_note: &str,
        confirm_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        let confirm_note = string_clear(
            confirm_note,
            StringClear::Option(STRING_CLEAR_FORMAT | STRING_CLEAR_XSS),
            Some(255),
        );
        app.app_status_check()?;
        if ![AppRequestStatus::Approved, AppRequestStatus::Rejected].contains(&req_status) {
            return Err(AppError::System(fluent_message!("app-req-status-invalid")));
        }
        let req_res = sqlx::query_as::<_, AppRequestModel>(&sql_format!(
            "select * from {} where app_id={} and request_type={}
            ",
            AppRequestModel::table_name(),
            app.id,
            AppRequestType::OAuthClient as i8,
        ))
        .fetch_one(&self.db)
        .await;
        let req = match req_res {
            Ok(req) => {
                if !AppRequestStatus::Pending.eq(req.status) {
                    return Ok(());
                }
                req
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::System(fluent_message!("app-req-bad")));
            }
            Err(err) => {
                return Err(err.into());
            }
        };

        let time = now_time()?;
        if req_status == AppRequestStatus::Rejected {
            //驳回
            let status = req_status as i8;
            // let confirm_note = confirm_note.to_owned();
            let change = model_option_set!(AppRequestModelRef,{
                status:status,
                confirm_user_id:confirm_user_id,
                confirm_time:time,
                confirm_note:confirm_note,
            });
            Update::<AppRequestModel, _>::new(change)
                .execute_by_pk(&req, &self.db)
                .await?;
            return Ok(());
        }

        let scope_res = sqlx::query_scalar::<_, String>(&sql_format!(
            "select scope_data from {} where app_request_id={}",
            AppRequestOAuthClientModel::table_name(),
            req.id,
        ))
        .fetch_one(&self.db)
        .await;
        let mut set_req_status = req_status;
        let scope_data = match scope_res {
            Ok(scope) => scope,
            Err(sqlx::Error::RowNotFound) => {
                set_req_status = AppRequestStatus::Rejected;
                "".to_string()
            }
            Err(err) => {
                return Err(err.into());
            }
        };
        let mut scope_set = scope_data.split(",").collect::<Vec<&str>>();
        self.server_scope_check(app, &scope_set).await?;

        //通过
        let oa_res = sqlx::query_as::<_, (u64, String)>(&sql_format!(
            "select id,scope_data from {} where app_id={}
            ",
            AppOAuthClientModel::table_name(),
            app.id,
        ))
        .fetch_one(&self.db)
        .await;

        let fe_res = sqlx::query_scalar::<_, u64>(&sql_format!(
            "select id from {} where app_id={} and feature_key={}
            ",
            AppFeatureModel::table_name(),
            app.id,
            AppRequestType::OAuthClient.feature_key(),
        ))
        .fetch_one(&self.db)
        .await;

        let mut db = self.db.begin().await?;

        match fe_res {
            Ok(oid) => {
                let status = AppFeatureStatus::Enable as i8;
                let change = model_option_set!(AppFeatureModelRef,{
                    status:status,
                    change_user_id:confirm_user_id,
                    change_time:time
                });
                let cres = Update::<AppFeatureModel, _>::new(change)
                    .execute_by_where(&WhereOption::Where(sql_format!("id={}", oid)), &mut *db)
                    .await;
                if let Err(err) = cres {
                    db.rollback().await?;
                    return Err(err.into());
                }
            }
            Err(sqlx::Error::RowNotFound) => {
                let status = AppFeatureStatus::Enable as i8;
                let feature_key = AppRequestType::OAuthClient.feature_key().to_string();
                let idata = model_option_set!(AppFeatureModelRef,{
                    app_id:app.id,
                    status:status,
                    feature_key:feature_key,
                    change_user_id:confirm_user_id,
                    change_time:time
                });
                let cres = Insert::<AppFeatureModel, _>::new(idata)
                    .execute(&mut *db)
                    .await;
                if let Err(err) = cres {
                    db.rollback().await?;
                    return Err(err.into());
                }
            }
            Err(err) => {
                db.rollback().await?;
                return Err(err.into());
            }
        }

        match oa_res {
            Ok((oid, in_scope_data)) => {
                for ts in in_scope_data.split(",").collect::<Vec<&str>>() {
                    if !scope_set.contains(&ts) {
                        scope_set.push(ts);
                    }
                }
                let set_scope = scope_set.join(",");
                let change = model_option_set!(AppOAuthClientModelRef,{
                    scope_data:set_scope,
                    change_user_id:confirm_user_id,
                    change_time:time
                });
                let cres = Update::<AppOAuthClientModel, _>::new(change)
                    .execute_by_where(&WhereOption::Where(sql_format!("id={}", oid)), &mut *db)
                    .await;
                if let Err(err) = cres {
                    db.rollback().await?;
                    return Err(err.into());
                }
            }
            Err(sqlx::Error::RowNotFound) => {
                let idata = model_option_set!(AppOAuthClientModelRef,{
                    app_id:app.id,
                    scope_data:scope_data,
                    change_user_id:confirm_user_id,
                    change_time:time
                });
                let cres = Insert::<AppOAuthClientModel, _>::new(idata)
                    .execute(&mut *db)
                    .await;
                if let Err(err) = cres {
                    db.rollback().await?;
                    return Err(err.into());
                }

                let secret_data = rand_str(lsys_core::RandType::LowerHex, 32);
                if let Err(e) = self
                    .app_secret
                    .multiple_add(
                        app.id,
                        AppSecretType::OAuth,
                        &secret_data,
                        0,
                        confirm_user_id,
                        &mut *db,
                    )
                    .await
                {
                    db.rollback().await?;
                    return Err(e);
                };
            }
            Err(err) => {
                db.rollback().await?;
                return Err(err.into());
            }
        };
        let status = set_req_status as i8;
        let confirm_note = confirm_note.to_owned();
        let change = model_option_set!(AppRequestModelRef,{
            status:status,
            confirm_user_id:confirm_user_id,
            confirm_time:time,
            confirm_note:confirm_note,
        });
        let cres = Update::<AppRequestModel, _>::new(change)
            .execute_by_pk(&req, &mut *db)
            .await;
        if let Err(err) = cres {
            db.rollback().await?;
            return Err(err.into());
        }
        db.commit().await?;

        self.app.feature_cache.del(&app.id).await;
        self.oauth_client_cache.del(&app.id).await;

        self.logger
            .add(
                &AppRequestLog {
                    action: "oauth_confirm",
                    parent_app_id: app.parent_app_id,
                    app_id: app.id,
                    user_id: req.request_user_id,
                    request_type: req.request_type,
                    status,
                    req_data: Some(&scope_data),
                },
                Some(req.id),
                Some(confirm_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    //OAUTH登录的SOPCE申请确认
    pub async fn scope_confirm(
        &self,
        app: &AppModel,
        req: &AppRequestModel,
        req_status: AppRequestStatus,
        confirm_note: &str,
        confirm_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        self.oauth_check(app).await?;
        if !AppRequestStatus::Pending.eq(req.status) {
            return Ok(());
        }
        if ![AppRequestStatus::Approved, AppRequestStatus::Rejected].contains(&req_status) {
            return Err(AppError::System(fluent_message!("app-req-status-invalid")));
        }
        if !AppRequestType::OAuthClientScope.eq(req.request_type) || req.app_id != app.id {
            return Err(AppError::System(fluent_message!("app-req-bad")));
        }
        let scope_res = sqlx::query_scalar::<_, String>(&sql_format!(
            "select scope_data from {} where app_request_id={}",
            AppRequestOAuthClientModel::table_name(),
            req.id,
        ))
        .fetch_one(&self.db)
        .await;
        let mut set_req_status = req_status;
        let scope_data = match scope_res {
            Ok(scope) => scope,
            Err(sqlx::Error::RowNotFound) => {
                set_req_status = AppRequestStatus::Rejected;
                "".to_string()
            }
            Err(err) => {
                return Err(err.into());
            }
        };
        let time = now_time()?;
        if set_req_status == AppRequestStatus::Rejected {
            //驳回
            let status = set_req_status as i8;
            let confirm_note = confirm_note.to_owned();
            let change = model_option_set!(AppRequestModelRef,{
                status:status,
                confirm_user_id:confirm_user_id,
                confirm_time:time,
                confirm_note:confirm_note,
            });
            Update::<AppRequestModel, _>::new(change)
                .execute_by_pk(req, &self.db)
                .await?;
            return Ok(());
        }
        let in_scope_data = self.get_oauth_client_scope_data(app).await?;
        let time = now_time()?;
        let mut db = self.db.begin().await?;
        let mut tmp_scope = scope_data.split(",").collect::<Vec<&str>>();
        for ts in in_scope_data.split(",").collect::<Vec<&str>>() {
            if !tmp_scope.contains(&ts) {
                tmp_scope.push(ts);
            }
        }

        if let Err(err) = self.server_scope_check(app, &tmp_scope).await {
            db.rollback().await?;
            return Err(err);
        }

        let set_scope = tmp_scope.join(",");
        let change = model_option_set!(AppOAuthClientModelRef,{
            scope_data:set_scope,
            change_user_id:confirm_user_id,
            change_time:time
        });
        let cres = Update::<AppOAuthClientModel, _>::new(change)
            .execute_by_where(
                &lsys_core::db::WhereOption::Where(sql_format!("app_id={}", app.id)),
                &mut *db,
            )
            .await;
        if let Err(err) = cres {
            db.rollback().await?;
            return Err(err.into());
        }
        let status = AppRequestStatus::Approved as i8;
        let confirm_note = confirm_note.to_owned();
        let change = model_option_set!(AppRequestModelRef,{
            status:status,
            confirm_user_id:confirm_user_id,
            confirm_time:time,
            confirm_note:confirm_note,
        });
        let cres = Update::<AppRequestModel, _>::new(change)
            .execute_by_pk(req, &mut *db)
            .await;
        if let Err(err) = cres {
            db.rollback().await?;
            return Err(err.into());
        }

        db.commit().await?;

        self.oauth_client_cache.del(&app.id).await;

        self.logger
            .add(
                &AppRequestLog {
                    action: "scope_confirm",
                    parent_app_id: app.parent_app_id,
                    app_id: app.id,
                    user_id: req.request_user_id,
                    request_type: req.request_type,
                    status,
                    req_data: Some(&scope_data),
                },
                Some(req.id),
                Some(confirm_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
}
pub struct AppOAuthClientParam<'t> {
    pub oauth_secret: Option<&'t str>,
    pub reset_secret: bool,
    pub callback_domain: Option<&'t str>,
}
impl AppOAuthClient {
    async fn oauth_set_domain_param_valid(&self, callback_domain: &str) -> AppResult<()> {
        ValidParam::default()
            .add(
                valid_key!("callback_domain"),
                &callback_domain,
                &ValidParamCheck::default().add_rule(ValidDomain::default()),
            )
            .check()?;
        Ok(())
    }
    //OAUTH登录参数设置
    pub async fn oauth_set_domain(
        &self,
        app: &AppModel,
        callback_domain: &str,
        set_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        self.oauth_set_domain_param_valid(callback_domain).await?;
        self.oauth_check(app).await?;
        let oa_res = sqlx::query_scalar::<_, u64>(&sql_format!(
            "select id from {} where app_id={}
            ",
            AppOAuthClientModel::table_name(),
            app.id,
        ))
        .fetch_one(&self.db)
        .await;
        let time = now_time()?;

        let callback_domain = callback_domain.to_owned();
        match oa_res {
            Ok(oid) => {
                let change = model_option_set!(AppOAuthClientModelRef,{
                    change_user_id:set_user_id,
                    change_time:time,
                    callback_domain:callback_domain
                });

                Update::<AppOAuthClientModel, _>::new(change)
                    .execute_by_where(&WhereOption::Where(sql_format!("id={}", oid)), &self.db)
                    .await?;
            }
            Err(sqlx::Error::RowNotFound) => {
                let scope_data = "".to_string();
                let idata = model_option_set!(AppOAuthClientModelRef,{
                    app_id:app.id,
                    scope_data:scope_data,
                    change_user_id:set_user_id,
                    change_time:time,
                    callback_domain:callback_domain
                });
                Insert::<AppOAuthClientModel, _>::new(idata)
                    .execute(&self.db)
                    .await?;
            }
            Err(err) => {
                return Err(err.into());
            }
        };
        self.oauth_client_cache.del(&app.id).await;
        self.logger
            .add(
                &AppOAuthClientSetDomainLog {
                    parent_app_id: app.parent_app_id,
                    app_id: app.id,
                    user_id: app.user_id,
                    callback_domain: &callback_domain,
                },
                Some(app.id),
                Some(set_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    //添加secret
    pub async fn secret_add(
        &self,
        app: &AppModel,
        secret: Option<&str>,
        time_out: u64,
        change_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<String> {
        let client_secret = match secret {
            Some(sstr) => sstr.to_string(),
            None => rand_str(lsys_core::RandType::LowerHex, 32),
        };
        self.app_secret
            .multiple_add(
                app.id,
                AppSecretType::OAuth,
                &client_secret,
                time_out,
                change_user_id,
                &self.db,
            )
            .await?;
        self.logger
            .add(
                &AppOAuthClientSecretSetLog {
                    action: "add",
                    parent_app_id: app.parent_app_id,
                    app_id: app.id,
                    user_id: app.user_id,
                    oauth_secret: &client_secret,
                },
                Some(app.id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        Ok(client_secret)
    }
    //重设secret
    pub async fn secret_change(
        &self,
        app: &AppModel,
        old_secret: &str,
        secret: Option<&str>,
        time_out: u64,
        change_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<String> {
        let client_secret = match secret {
            Some(sstr) => sstr.to_string(),
            None => rand_str(lsys_core::RandType::LowerHex, 32),
        };
        self.app_secret
            .multiple_change(
                app.id,
                AppSecretType::OAuth,
                &client_secret,
                old_secret,
                time_out,
                change_user_id,
                &self.db,
            )
            .await?;
        self.logger
            .add(
                &AppOAuthClientSecretSetLog {
                    action: "change",
                    parent_app_id: app.parent_app_id,
                    app_id: app.id,
                    user_id: app.user_id,
                    oauth_secret: &client_secret,
                },
                Some(app.id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        Ok(client_secret)
    }
    //删除secret
    pub async fn secret_del(
        &self,
        app: &AppModel,
        old_secret: &str,
        change_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        self.app_secret
            .multiple_del(
                app.id,
                AppSecretType::OAuth,
                old_secret,
                change_user_id,
                &self.db,
            )
            .await?;
        self.logger
            .add(
                &AppOAuthClientSecretSetLog {
                    action: "delete",
                    parent_app_id: app.parent_app_id,
                    app_id: app.id,
                    user_id: app.user_id,
                    oauth_secret: old_secret,
                },
                Some(app.id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
}
