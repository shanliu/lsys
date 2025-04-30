mod cache;
mod data;
use super::logger::AppOAuthServerSetLog;
use super::{App, AppError, AppResult};
use crate::model::{AppModel, AppRequestModel};
use crate::model::{
    AppOAuthClientModel, AppOAuthServerScopeModel, AppOAuthServerScopeModelRef,
    AppOAuthServerScopeStatus, AppRequestStatus, AppRequestType, AppStatus,
};
pub use data::AppOAuthServerScopeData;
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::db::{Insert, ModelTableName, Update};
use lsys_core::db::{SqlQuote, WhereOption};
use lsys_core::{
    fluent_message, now_time, RemoteNotify, RequestEnv, ValidParam, ValidParamCheck, ValidPattern,
    ValidPatternRule, ValidStrlen,
};
use lsys_core::{model_option_set, sql_format};
use lsys_logger::dao::ChangeLoggerDao;
// use regex::Regex;
use serde::Serialize;
use serde_json::json;
use sqlx::{MySql, Pool};
use std::sync::Arc;
pub struct AppOAuthServer {
    app: Arc<App>,
    db: Pool<MySql>,
    logger: Arc<ChangeLoggerDao>,
    pub(crate) oauth_server_scope_cache: Arc<LocalCache<u64, Vec<AppOAuthServerScopeData>>>,
}

impl AppOAuthServer {
    pub fn new(
        db: Pool<MySql>,
        app: Arc<App>,
        logger: Arc<ChangeLoggerDao>,
        remote_notify: Arc<RemoteNotify>,
        cache_config: LocalCacheConfig,
    ) -> Self {
        Self {
            db,
            app,
            logger,
            oauth_server_scope_cache: Arc::new(LocalCache::new(
                remote_notify.clone(),
                cache_config,
            )),
        }
    }
    //OAUTH 服务申请
    pub async fn oauth_request(
        &self,
        app: &AppModel,
        req_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        app.app_status_check()?;
        app.is_system_app_check()?;
        self.app
            .inner_feature_request(app, AppRequestType::OAuthServer, req_user_id, env_data)
            .await
    }
    //OAUTH 服务申请确认
    pub async fn oauth_confirm(
        &self,
        app: &AppModel,
        req_status: AppRequestStatus,
        confirm_note: &str,
        confirm_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        app.app_status_check()?;
        let req = sqlx::query_as::<_, AppRequestModel>(&sql_format!(
            "select id,status from {} where app_id={} and feature_key = {}",
            AppRequestModel::table_name(),
            app.id,
            AppRequestType::OAuthServer.feature_key()
        ))
        .fetch_one(&self.db)
        .await?;
        self.app
            .inner_feature_confirm(
                app,
                &req,
                req_status,
                confirm_note,
                confirm_user_id,
                env_data,
            )
            .await
    }
}
#[derive(Serialize)]
pub struct AppOAuthServerScopeParam<'t> {
    pub key: &'t str,
    pub name: &'t str,
    pub desc: &'t str,
}
impl AppOAuthServer {
    //OAUTH 服务设置 SOPCE 数据
    pub async fn oauth_setting<'t>(
        &self,
        app: &AppModel,
        req: &'t [AppOAuthServerScopeParam<'t>],
        set_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        // let re = Regex::new(r"^[a-z0-9_]+$")
        //     .map_err(|e| AppError::System(fluent_message!("rule-error", e)))?;

        for tmp in req {
            ValidParam::default()
                .add(
                    "key",
                    &tmp.key,
                    &ValidParamCheck::default()
                        .add_rule(ValidStrlen::range(2, 32))
                        .add_rule(ValidPattern::new(ValidPatternRule::Ident)),
                )
                .add(
                    "name",
                    &tmp.name,
                    &ValidParamCheck::default().add_rule(ValidStrlen::range(2, 64)),
                )
                .check()?;

            // if !re.is_match(tmp.key) || tmp.key.len() < 2 {
            //     return Err(AppError::System(fluent_message!(
            //         "app-oauth-server-skey-wrong"
            //     )));
            // }
            // if tmp.name.trim().is_empty() {
            //     return Err(AppError::System(fluent_message!(
            //         "app-oauth-server-name-wrong"
            //     )));
            // }
        }

        self.oauth_check(app).await?;
        let find_res = sqlx::query_as::<_, (u64, String)>(&sql_format!(
            "select id,scope_key from {} where app_id={}",
            AppOAuthServerScopeModel::table_name(),
            app.id,
        ))
        .fetch_all(&self.db)
        .await?;
        let time = now_time()?;
        let mut add_data = vec![];
        let mut change_data = vec![];
        for tmp in req {
            match find_res.iter().find(|e| e.1.as_str() == tmp.key) {
                Some(stmp) => {
                    change_data.push((
                        stmp.0,
                        tmp.key.to_string(),
                        tmp.name.to_string(),
                        tmp.desc.to_string(),
                    ));
                }
                None => {
                    add_data.push((
                        tmp.key.to_string(),
                        tmp.name.to_string(),
                        tmp.desc.to_string(),
                    ));
                }
            }
        }

        let mut del_data = vec![];
        for (_, skey) in find_res {
            if !req.iter().any(|m| m.key == skey) {
                del_data.push(skey);
            }
        }

        for del_key in del_data.iter() {
            let req = sqlx::query_scalar::<_, String>(&sql_format!(
                "select app.name from {} as app 
                join  {} as oc on app.id=oc.app_id
                where app.parent_app_id={} AND status IN ({}) and FIND_IN_SET({}, oc.scope_data)",
                AppModel::table_name(),
                AppOAuthClientModel::table_name(),
                app.id,
                &[
                    AppStatus::Enable as i8,
                    AppStatus::Init as i8,
                    AppStatus::Disable as i8
                ],
                del_key,
            ))
            .fetch_all(&self.db)
            .await?;
            if !req.is_empty() {
                return Err(AppError::System(
                    fluent_message!("app-oauth-server-use-scope",{
                        "scope_data":req.join(",")
                    }),
                ));
            }
        }

        let mut db = self.db.begin().await?;

        if !del_data.is_empty() {
            let del_status = AppOAuthServerScopeStatus::Delete as i8;
            let change = model_option_set!(AppOAuthServerScopeModelRef,{
                status:del_status,
                change_user_id:set_user_id,
                change_time:time
            });
            let cres = Update::<AppOAuthServerScopeModel, _>::new(change)
                .execute_by_where(
                    &lsys_core::db::WhereOption::Where(sql_format!(
                        "app_id={} and scope_key in ({})",
                        app.id,
                        del_data
                    )),
                    &mut *db,
                )
                .await;
            if let Err(err) = cres {
                db.rollback().await?;
                return Err(err.into());
            }
        }

        let set_status = AppOAuthServerScopeStatus::Enable as i8;

        for tmp in change_data {
            let change = model_option_set!(AppOAuthServerScopeModelRef,{
                app_id:app.id,
                scope_key:tmp.1,
                status:set_status,
                scope_name:tmp.2,
                scope_desc:tmp.3,
                change_user_id:set_user_id,
                change_time:time
            });
            let cres = Update::<AppOAuthServerScopeModel, _>::new(change)
                .execute_by_where(&WhereOption::Where(sql_format!("id={}", tmp.0)), &mut *db)
                .await;
            if let Err(err) = cres {
                db.rollback().await?;
                return Err(err.into());
            }
        }
        let mut add_vec = vec![];
        for tmp in add_data.iter() {
            add_vec.push(model_option_set!(AppOAuthServerScopeModelRef,{
                app_id:app.id,
                scope_key:tmp.0,
                scope_name:tmp.1,
                scope_desc:tmp.2,
                status:set_status,
                change_user_id:set_user_id,
                change_time:time
            }));
        }
        let cres = Insert::<AppOAuthServerScopeModel, _>::new_vec(add_vec)
            .execute(&mut *db)
            .await;
        if let Err(err) = cres {
            db.rollback().await?;
            return Err(err.into());
        }

        db.commit().await?;

        self.oauth_server_scope_cache.del(&app.id).await;

        self.logger
            .add(
                &AppOAuthServerSetLog {
                    parent_app_id: app.parent_app_id,
                    app_id: app.id,
                    scope_data: &json!(req).to_string(),
                    user_id: app.user_id,
                },
                Some(app.id),
                Some(set_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
}
