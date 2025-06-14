mod cache;
mod data;
mod feature;
mod request;
mod sub_app;
use lsys_access::dao::AccessDao;
use lsys_core::{
    fluent_message, now_time, rand_str, string_clear, valid_key, AppCore, RequestEnv, StringClear,
    TimeOutTask, TimeOutTaskNotify, ValidError, ValidParam, ValidParamCheck, ValidPattern,
    ValidStrlen, STRING_CLEAR_FORMAT, STRING_CLEAR_XSS,
};

pub use data::*;
use lsys_core::db::{Insert, ModelTableName, Update, WhereOption};
use lsys_core::{model_option_set, sql_format};
pub use request::*;
pub use sub_app::*;

use std::sync::Arc;

use crate::model::AppRequestSetInfoModelRef;
use crate::model::{
    AppModel, AppModelRef, AppRequestModel, AppRequestModelRef, AppRequestSetInfoModel,
    AppRequestStatus, AppRequestType, AppSecretType, AppStatus,
};
use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    RemoteNotify,
};

use super::AppSecret;
use super::{logger::AppLog, AppError, AppResult};
use lsys_core::db::SqlQuote;
use lsys_logger::dao::ChangeLoggerDao;
// use regex::Regex;
use sqlx::{MySql, Pool};
pub struct App {
    app_core: Arc<AppCore>,
    db: Pool<MySql>,
    pub(crate) id_cache: Arc<LocalCache<u64, AppModel>>, //appid,AppModel
    pub(crate) client_id_cache: Arc<LocalCache<String, Option<u64>>>, //client_id,appid
    pub(crate) feature_cache: Arc<LocalCache<u64, Vec<(String, bool)>>>, //appid,vec<(feature_key,exists)>
    logger: Arc<ChangeLoggerDao>,
    sub_app_change_notify: Arc<SubAppChangeNotify>,
    sub_app_timeout_notify: Arc<TimeOutTaskNotify>,
    app_secret: Arc<AppSecret>,
    access: Arc<AccessDao>,
}

impl App {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
        logger: Arc<ChangeLoggerDao>,
        app_secret: Arc<AppSecret>,
        sub_app_change_notify: Arc<SubAppChangeNotify>,
        sub_app_timeout_notify: Arc<TimeOutTaskNotify>,
        access: Arc<AccessDao>,
    ) -> Self {
        Self {
            app_core,
            db,
            id_cache: Arc::new(LocalCache::new(remote_notify.clone(), config)),
            client_id_cache: Arc::new(LocalCache::new(remote_notify.clone(), config)),
            feature_cache: Arc::new(LocalCache::new(remote_notify.clone(), config)),
            logger,
            app_secret,
            sub_app_change_notify,
            sub_app_timeout_notify,
            access,
        }
    }
}
pub struct AppDataParam<'t> {
    pub name: &'t str,
    pub client_id: &'t str,
}
impl App {
    pub async fn listen_sub_app_change_notify(&self, channel_buffer: Option<usize>) {
        TimeOutTask::<SubAppChangeNotify>::new(
            self.app_core.clone(),
            self.sub_app_timeout_notify.clone(),
            self.sub_app_change_notify.clone(),
            self.sub_app_change_notify.clone(),
        )
        .listen(channel_buffer)
        .await;
    }

    async fn check_app_param_valid(&self, param: &AppDataParam<'_>) -> AppResult<(String, String)> {
        let name = string_clear(param.name, StringClear::Option(STRING_CLEAR_FORMAT), None);
        let client_id = string_clear(
            param.client_id,
            StringClear::Option(STRING_CLEAR_FORMAT),
            None,
        );
        ValidParam::default()
            .add(
                valid_key!("app_name"),
                &name,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(3, 24)),
            )
            .add(
                valid_key!("client_id"),
                &client_id,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(3, 32))
                    .add_rule(ValidPattern::Ident),
            )
            .check()?;
        Ok((name, client_id))
    }
    //创建APP
    pub async fn app_new_request(
        &self,
        user_id: u64,
        parent_app: Option<&AppModel>,
        user_app_id: u64,
        param: &AppDataParam<'_>,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<u64> {
        if user_app_id > 0 {
            //某应用登录,只能在申请某应用下应用
            match parent_app {
                Some(papp) => {
                    if papp.id != user_app_id {
                        return Err(ValidError::message(
                            valid_key!("parent_app"),
                            fluent_message!("papp-not-match-parent",{
                                "name":&papp.name
                            }),
                        )
                        .into());
                    }
                }
                None => {
                    return Err(ValidError::message(
                        valid_key!("parent_app"),
                        fluent_message!("papp-bad-parent"),
                    )
                    .into());
                }
            }
        }
        if let Some(papp) = parent_app {
            if papp.parent_app_id > 0 {
                return Err(ValidError::message(
                    valid_key!("parent_app"),
                    fluent_message!("papp-id-bad",{
                        "name":&papp.name
                    }),
                )
                .into());
            }
        }

        let (name, client_id) = self.check_app_param_valid(param).await?;
        let app_res = sqlx::query_as::<_, AppModel>(&sql_format!(
            "select * from {} where client_id={} and status in ({})",
            AppModel::table_name(),
            client_id,
            &[
                AppStatus::Disable as i8,
                AppStatus::Enable as i8,
                AppStatus::Init as i8
            ]
        ))
        .fetch_one(&self.db)
        .await;
        match app_res {
            Ok(app) => {
                if app.user_id == user_id && app.name == name {
                    return Ok(app.id);
                } else {
                    return Err(ValidError::message(
                        valid_key!("client_id"),
                        fluent_message!("app-client-id-exits",{
                            "client_id":app.client_id,
                            "other_name":app.name
                        }),
                    )
                    .into());
                }
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }
        let req_res = sqlx::query_scalar::<_, u64>(&sql_format!(
            "select req.app_id from {}  as info
                join {} as req on info.app_request_id=req.id
                where info.client_id={} and req.status={} and req.request_type={} limit 1
            ",
            AppRequestSetInfoModel::table_name(),
            AppRequestModel::table_name(),
            client_id,
            AppRequestStatus::Pending as i8,
            AppRequestType::AppChange as i8
        ))
        .fetch_one(&self.db)
        .await;
        match req_res {
            Ok(app_id) => {
                //其他应用请求改为 client_id 值
                return Err(ValidError::message(
                    valid_key!("client_id"),
                    fluent_message!("app-client-id-req",{
                        "client_id":client_id,
                        "app_id":app_id,
                    }),
                )
                .into());
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }

        let mut db = self.db.begin().await?;

        let time = now_time()?;
        let status = AppStatus::Init as i8;
        let parent_app_id = parent_app.as_ref().map(|e| e.id).unwrap_or_default();

        let idata = model_option_set!(AppModelRef,{
            name:name,
            parent_app_id:parent_app_id,
            client_id:client_id,
            status:status,
            user_id:user_id,
            user_app_id:user_app_id,
            change_user_id:add_user_id,
            change_time:time,
        });
        let res = Insert::<AppModel, _>::new(idata).execute(&mut *db).await;

        let app_id = match res {
            Err(e) => {
                db.rollback().await?;
                return Err(e.into());
            }
            Ok(mr) => mr.last_insert_id(),
        };

        let secret_data = rand_str(lsys_core::RandType::LowerHex, 32);
        if let Err(e) = self
            .app_secret
            .single_set(
                app_id,
                AppSecretType::Notify,
                &secret_data,
                0,
                user_id,
                Some(&mut db),
            )
            .await
        {
            db.rollback().await?;
            return Err(e);
        };

        let secret_data = rand_str(lsys_core::RandType::LowerHex, 32);
        if let Err(e) = self
            .app_secret
            .multiple_add(
                app_id,
                AppSecretType::App,
                &secret_data,
                0,
                user_id,
                &mut *db,
            )
            .await
        {
            db.rollback().await?;
            return Err(e);
        };

        let req_status = AppRequestStatus::Pending as i8;
        let request_type = AppRequestType::AppReq as i8;
        let idata = model_option_set!(AppRequestModelRef,{
            parent_app_id:parent_app_id,
            app_id:app_id,
            request_type:request_type,
            status:req_status,
            request_user_id:user_id,
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
        let idata = model_option_set!(AppRequestSetInfoModelRef,{
            app_request_id:req_id,
            name:name,
            client_id:client_id,
        });
        let req_res = Insert::<AppRequestSetInfoModel, _>::new(idata)
            .execute(&mut *db)
            .await;
        if let Err(err) = req_res {
            db.rollback().await?;
            return Err(err.into());
        }
        db.commit().await?;

        self.client_id_cache.clear(&client_id).await;

        self.logger
            .add(
                &AppLog {
                    action: "add",
                    name: &name,
                    status,
                    user_id,
                    client_id: &client_id,
                    client_secret: Some(&secret_data),
                    parent_app_id,
                    user_app_id,
                },
                Some(app_id),
                Some(add_user_id),
                None,
                env_data,
            )
            .await;
        Ok(app_id)
    }
    //APP更改请求
    pub async fn app_change_request(
        &self,
        app: &AppModel,
        param: &AppDataParam<'_>,
        change_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        if AppStatus::Delete.eq(app.status) {
            return Err(AppError::AppNotFound(app.client_id.to_owned()));
        }
        let (name, client_id) = self.check_app_param_valid(param).await?;
        if app.name == name && app.client_id == client_id {
            return Ok(());
        }
        if app.client_id != client_id {
            let app_res = sqlx::query_as::<_, AppModel>(&sql_format!(
                "select * from {} where client_id={} and status in ({}) and id !={}",
                AppModel::table_name(),
                client_id,
                &[
                    AppStatus::Disable as i8,
                    AppStatus::Enable as i8,
                    AppStatus::Init as i8
                ],
                app.id
            ))
            .fetch_one(&self.db)
            .await;
            match app_res {
                Ok(app) => {
                    return Err(AppError::System(fluent_message!("app-client-id-exits",{
                        "client_id":app.client_id,
                        "other_name":app.name
                    })));
                }
                Err(sqlx::Error::RowNotFound) => {}
                Err(err) => {
                    return Err(err.into());
                }
            }
            let req_res = sqlx::query_scalar::<_, u64>(&sql_format!(
                "select req.app_id from {}  as info
                    join {} as req on info.app_request_id=req.id
                    where info.client_id={} and req.status={} and req.request_type={} limit 1
                ",
                AppRequestSetInfoModel::table_name(),
                AppRequestModel::table_name(),
                client_id,
                AppRequestStatus::Pending as i8,
                AppRequestType::AppChange as i8
            ))
            .fetch_one(&self.db)
            .await;
            match req_res {
                Ok(app_id) => {
                    return Err(AppError::System(fluent_message!("app-client-id-req",{
                        "client_id":client_id,
                        "app_id":app_id
                    })));
                }
                Err(sqlx::Error::RowNotFound) => {}
                Err(err) => {
                    return Err(err.into());
                }
            }
        }

        let time = now_time()?;
        let mut db = self.db.begin().await?;

        if AppStatus::Init.eq(app.status) {
            let change = model_option_set!(AppModelRef,{
                name:name,
                client_id:client_id
            });
            let req_res = Update::<AppModel, _>::new(change)
                .execute_by_pk(app, &mut *db)
                .await;
            if let Err(e) = req_res {
                db.rollback().await?;
                return Err(e.into());
            }
        }

        //废弃以前申请
        let req_status = AppRequestStatus::Invalid as i8;
        let change = model_option_set!(AppRequestModelRef,{
            status:req_status,
        });
        let req_res = Update::<AppRequestModel, _>::new(change)
            .execute_by_where(
                &lsys_core::db::WhereOption::Where(sql_format!(
                    "app_id={} and request_type in ({})",
                    app.id,
                    &[
                        AppRequestType::AppChange as i8,
                        AppRequestType::AppReq as i8
                    ]
                )),
                &mut *db,
            )
            .await;
        if let Err(e) = req_res {
            db.rollback().await?;
            return Err(e.into());
        }
        //重新申请
        let req_status = AppRequestStatus::Pending as i8;
        let request_type = if AppStatus::Init.eq(app.status) {
            AppRequestType::AppReq
        } else {
            AppRequestType::AppChange
        } as i8;
        let idata = model_option_set!(AppRequestModelRef,{
            parent_app_id:app.parent_app_id,
            app_id:app.id,
            request_type:request_type,
            status:req_status,
            request_user_id:change_user_id,
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
        let idata = model_option_set!(AppRequestSetInfoModelRef,{
            app_request_id:req_id,
            name:name,
            client_id:client_id,
        });
        let req_res = Insert::<AppRequestSetInfoModel, _>::new(idata)
            .execute(&mut *db)
            .await;
        if let Err(err) = req_res {
            db.rollback().await?;
            return Err(err.into());
        }
        db.commit().await?;

        self.client_id_cache.clear(&client_id).await;
        if client_id != app.client_id {
            self.client_id_cache.clear(&app.client_id).await;
        }
        self.id_cache.clear(&app.id).await;

        self.logger
            .add(
                &AppLog {
                    action: "change",
                    name: &name,
                    user_id: app.user_id,
                    status: app.status,
                    client_id: &client_id,
                    client_secret: None,
                    parent_app_id: app.parent_app_id,
                    user_app_id: app.user_app_id,
                },
                Some(app.id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    //审核APP
    pub async fn app_confirm_request(
        &self,
        app: &AppModel,
        req: &AppRequestModel,
        confirm_status: AppRequestStatus,
        confirm_note: &str,
        confirm_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        let confirm_note = string_clear(
            confirm_note,
            StringClear::Option(STRING_CLEAR_FORMAT | STRING_CLEAR_XSS),
            Some(255),
        );
        if AppStatus::Delete.eq(app.status) {
            return Err(AppError::AppNotFound(app.client_id.to_owned()));
        }
        if req.app_id != app.id {
            return Err(AppError::System(fluent_message!("app-req-bad-app")));
        }
        if !AppRequestStatus::Pending.eq(req.status) {
            return Err(AppError::System(fluent_message!("app-req-is-confirm")));
        }
        if ![AppRequestStatus::Approved, AppRequestStatus::Rejected].contains(&confirm_status) {
            return Err(AppError::System(fluent_message!("app-req-status-invalid")));
        }
        if ![
            AppRequestType::AppChange as i8,
            AppRequestType::AppReq as i8,
        ]
        .contains(&req.request_type)
        {
            return Err(AppError::System(fluent_message!("app-req-is-invalid")));
        }
        let req_info = match sqlx::query_as::<_, AppRequestSetInfoModel>(&sql_format!(
            "select * from {} where app_request_id={}",
            AppRequestSetInfoModel::table_name(),
            req.id,
        ))
        .fetch_one(&self.db)
        .await
        {
            Ok(info) => info,
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::System(fluent_message!("app-req-is-miss-info")));
            }
            Err(err) => {
                return Err(err.into());
            }
        };
        let app_res = sqlx::query_as::<_, AppModel>(&sql_format!(
            "select * from {} where client_id={} and status in ({}) and id !={}",
            AppModel::table_name(),
            req_info.client_id,
            &[
                AppStatus::Disable as i8,
                AppStatus::Enable as i8,
                AppStatus::Init as i8
            ],
            app.id
        ))
        .fetch_one(&self.db)
        .await;
        match app_res {
            Ok(app) => {
                return Err(AppError::System(fluent_message!("app-client-id-exits",{
                    "client_id":app.client_id,
                    "other_name":app.name
                })));
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }
        let time = now_time()?;
        let mut db = self.db.begin().await?;

        let status = AppStatus::Enable as i8;

        let change = model_option_set!(AppModelRef,{
            name:req_info.name,
            client_id:req_info.client_id,
            status:status,
            change_user_id:confirm_user_id,
            change_time:time
        });
        let req_res = Update::<AppModel, _>::new(change)
            .execute_by_pk(app, &mut *db)
            .await;
        if let Err(e) = req_res {
            db.rollback().await?;
            return Err(e.into());
        }

        //废弃以前申请
        let confirm_status = confirm_status as i8;
        let confirm_note = confirm_note.to_string();
        let change = model_option_set!(AppRequestModelRef,{
            status:confirm_status,
            confirm_user_id:confirm_user_id,
            confirm_time:time,
            confirm_note:confirm_note,
        });
        let req_res = Update::<AppRequestModel, _>::new(change)
            .execute_by_pk(req, &mut *db)
            .await;
        if let Err(e) = req_res {
            db.rollback().await?;
            return Err(e.into());
        }

        db.commit().await?;

        self.client_id_cache.clear(&req_info.client_id).await;
        if req_info.client_id != app.client_id {
            self.client_id_cache.clear(&app.client_id).await;
        }
        self.id_cache.clear(&app.id).await;

        self.logger
            .add(
                &AppLog {
                    action: "confirm",
                    name: &app.name,
                    status: app.status,
                    user_id: app.user_id,
                    client_id: &app.client_id,
                    client_secret: None,
                    parent_app_id: app.parent_app_id,
                    user_app_id: app.user_app_id,
                },
                Some(app.id),
                Some(confirm_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    //禁用APP
    pub async fn app_disable(
        &self,
        app: &AppModel,
        disable_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        if AppStatus::Disable.eq(app.status) {
            self.app_close_clear(app.id).await;
            return Ok(());
        }
        if AppStatus::Delete.eq(app.status) {
            return Err(AppError::AppNotFound(app.client_id.to_owned()));
        }
        if ![AppStatus::Enable as i8, AppStatus::Init as i8].contains(&app.status) {
            return Err(AppError::System(fluent_message!("app-req-status-invalid")));
        }

        let time = now_time()?;
        let mut db = self.db.begin().await?;

        let status = AppStatus::Disable as i8;

        let change = model_option_set!(AppModelRef,{
            status:status,
            change_user_id:disable_user_id,
            change_time:time
        });
        let req_res = Update::<AppModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("id={} or parent_app_id={}", app.id, app.id)),
                &mut *db,
            )
            .await;
        if let Err(e) = req_res {
            db.rollback().await?;
            return Err(e.into());
        }

        //废弃以前申请
        let confirm_status = AppRequestStatus::Invalid as i8;
        let change = model_option_set!(AppRequestModelRef,{
            status:confirm_status,
            confirm_user_id:disable_user_id,
            confirm_time:time,
        });
        let req_res = Update::<AppRequestModel, _>::new(change)
            .execute_by_where(
                &lsys_core::db::WhereOption::Where(sql_format!(
                    "(app_id={} or app_id in (select id from {} where parent_app_id={})) and status={}",
                    app.id,
                    AppModel::table_name(),
                    app.id,
                    AppRequestStatus::Pending as i8
                )),
                &mut *db,
            )
            .await;
        if let Err(e) = req_res {
            db.rollback().await?;
            return Err(e.into());
        }

        db.commit().await?;

        self.client_id_cache.clear(&app.client_id).await;
        self.id_cache.clear(&app.id).await;
        self.app_close_clear(app.id).await;

        let mut clear_start_id = 0;
        loop {
            let sub_app = sqlx::query_as::<_, (u64,String)>(&sql_format!(
                "select * from {} where  parent_app_id ={} and status = {} and id>{}  order by id asc limit 100",
                AppModel::table_name(),
                app.id,
                 AppStatus::Disable as i8,
                clear_start_id
            ))
            .fetch_all(&self.db)
            .await?;
            if sub_app.is_empty() {
                break;
            }
            for sapp in sub_app {
                clear_start_id = sapp.0;
                self.client_id_cache.clear(&sapp.1).await;
                self.id_cache.clear(&sapp.0).await;
                self.app_close_clear(sapp.0).await;
            }
        }

        self.logger
            .add(
                &AppLog {
                    action: "disable",
                    name: &app.name,
                    status: app.status,
                    user_id: app.user_id,
                    client_id: &app.client_id,
                    client_secret: None,
                    parent_app_id: app.parent_app_id,
                    user_app_id: app.user_app_id,
                },
                Some(app.id),
                Some(disable_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
    async fn app_close_clear(&self, app_id: u64) {
        let _ = self.access.auth.clear_app_login(app_id).await;
    }
    //禁用APP
    pub async fn app_delete(
        &self,
        app: &AppModel,
        delete_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        if AppStatus::Delete.eq(app.status) {
            self.app_close_clear(app.id).await;
            return Ok(());
        }

        let sub_app_count = sqlx::query_scalar::<_, i64>(&sql_format!(
            "select count(*) as total from {} where  parent_app_id ={} and status in ({})",
            AppModel::table_name(),
            app.id,
            &[
                AppStatus::Enable as i8,
                AppStatus::Init as i8,
                AppStatus::Disable as i8
            ],
        ))
        .fetch_one(&self.db)
        .await;
        match sub_app_count {
            Ok(total) => {
                if total > 0 {
                    return Err(AppError::System(fluent_message!("app-exits-sub-app",{
                        "total":total,
                    })));
                }
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }

        let time = now_time()?;
        let mut db = self.db.begin().await?;

        let status = AppStatus::Delete as i8;

        let change_app = model_option_set!(AppModelRef,{
            status:status,
            change_user_id:delete_user_id,
            change_time:time
        });
        let req_res = Update::<AppModel, _>::new(change_app)
            .execute_by_pk(app, &mut *db)
            .await;
        if let Err(e) = req_res {
            db.rollback().await?;
            return Err(e.into());
        }

        //废弃以前申请
        let confirm_status = AppRequestStatus::Invalid as i8;
        let change_req = model_option_set!(AppRequestModelRef,{
            status:confirm_status,
            confirm_user_id:delete_user_id,
            confirm_time:time,
        });
        let req_res = Update::<AppRequestModel, _>::new(change_req)
            .execute_by_where(
                &lsys_core::db::WhereOption::Where(sql_format!(
                    "app_id={} and status={}",
                    app.id,
                    AppRequestStatus::Pending as i8
                )),
                &mut *db,
            )
            .await;
        if let Err(e) = req_res {
            db.rollback().await?;
            return Err(e.into());
        }

        if let Err(e) = self
            .app_secret
            .delete_from_app_id(app.id, delete_user_id, &mut *db)
            .await
        {
            db.rollback().await?;
            return Err(e);
        };

        db.commit().await?;

        self.client_id_cache.clear(&app.client_id).await;
        self.id_cache.clear(&app.id).await;

        self.logger
            .add(
                &AppLog {
                    action: "delete",
                    name: &app.name,
                    user_id: app.user_id,
                    status: app.status,
                    client_id: &app.client_id,
                    client_secret: None,
                    parent_app_id: app.parent_app_id,
                    user_app_id: app.user_app_id,
                },
                Some(app.id),
                Some(delete_user_id),
                None,
                env_data,
            )
            .await;
        self.app_close_clear(app.id).await;
        Ok(())
    }
}
impl App {
    pub async fn notify_secret_change(
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
        let mut db = self.db.begin().await?;
        self.app_secret
            .single_set(
                app.id,
                AppSecretType::Notify,
                &client_secret,
                time_out,
                change_user_id,
                Some(&mut db),
            )
            .await?;
        db.commit().await?;
        self.logger
            .add(
                &AppLog {
                    action: "notify_secret_change",
                    name: &app.name,
                    user_id: app.user_id,
                    status: app.status,
                    client_id: &app.client_id,
                    client_secret: Some(&client_secret),
                    parent_app_id: app.parent_app_id,
                    user_app_id: app.user_app_id,
                },
                Some(app.id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        Ok(client_secret)
    }
    //添加secret
    pub async fn app_secret_add(
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
                AppSecretType::App,
                &client_secret,
                time_out,
                change_user_id,
                &self.db,
            )
            .await?;
        self.logger
            .add(
                &AppLog {
                    action: "app_secret_add",
                    name: &app.name,
                    user_id: app.user_id,
                    status: app.status,
                    client_id: &app.client_id,
                    client_secret: Some(&client_secret),
                    parent_app_id: app.parent_app_id,
                    user_app_id: app.user_app_id,
                },
                Some(app.id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        self.sub_app_change_notify
            .add_app_secret_change_notify(app)
            .await;
        if time_out > 0 {
            self.sub_app_timeout_notify.notify_timeout(time_out).await?;
        }
        Ok(client_secret)
    }
    //重设secret
    pub async fn app_secret_change(
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
                AppSecretType::App,
                &client_secret,
                old_secret,
                time_out,
                change_user_id,
                &self.db,
            )
            .await?;
        self.logger
            .add(
                &AppLog {
                    action: "app_secret_change",
                    name: &app.name,
                    user_id: app.user_id,
                    status: app.status,
                    client_id: &app.client_id,
                    client_secret: Some(&client_secret),
                    parent_app_id: app.parent_app_id,
                    user_app_id: app.user_app_id,
                },
                Some(app.id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        self.sub_app_change_notify
            .add_app_secret_change_notify(app)
            .await;
        if time_out > 0 {
            self.sub_app_timeout_notify.notify_timeout(time_out).await?;
        }
        Ok(client_secret)
    }
    //删除secret
    pub async fn app_secret_del(
        &self,
        app: &AppModel,
        old_secret: &str,
        change_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        self.app_secret
            .multiple_del(
                app.id,
                AppSecretType::App,
                old_secret,
                change_user_id,
                &self.db,
            )
            .await?;
        self.logger
            .add(
                &AppLog {
                    action: "app_secret_del",
                    name: &app.name,
                    user_id: app.user_id,
                    status: app.status,
                    client_id: &app.client_id,
                    client_secret: Some(old_secret),
                    parent_app_id: app.parent_app_id,
                    user_app_id: app.user_app_id,
                },
                Some(app.id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        self.sub_app_change_notify
            .add_app_secret_change_notify(app)
            .await;
        Ok(())
    }
}
