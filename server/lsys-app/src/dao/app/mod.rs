mod cache;
mod data;
mod feature;
mod request;
mod sub_app;
use lsys_access::dao::AccessDao;
use lsys_core::{
    db::utils::FetchField, fluent_message, valid_key,
};
use lsys_core::app_core::AppCore;
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::timeout_task::{TimeOutTask, TimeOutTaskNotify};
use lsys_core::utils::{
    now_time, rand_str, string_clear, RandType, RequestEnv, StringClear, STRING_CLEAR_FORMAT,
    STRING_CLEAR_XSS,
};
use lsys_core::valid_param::{ValidError, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen};

pub use data::*;
use lsys_core::db::{Insert, QueryBuilderExt, TableMeta, Update};
pub use request::*;
pub use sub_app::*;

use std::sync::Arc;

use crate::model::{
    AppModel, AppRequestModel, AppRequestSetInfoModel,
    AppRequestStatus, AppRequestType, AppSecretType, AppStatus,
};
use lsys_core::cache::{LocalCache, LocalCacheConfig};

use super::AppSecret;
use super::{logger::AppLog, AppError, AppResult};
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

        // 先获取所有字段长度
        let fetch_field = FetchField::new(&self.db);
        let name_max = fetch_field.string_max::<AppModel>(&AppModel::NAME)
            .await
            .len_or(24);
        let client_id_max = fetch_field.string_max::<AppModel>(&AppModel::CLIENT_ID)
            .await
            .len_or(32);

        ValidParam::default()
            .add(
                valid_key!("app_name"),
                &name,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(3, name_max)),
            )
            .add(
                valid_key!("client_id"),
                &client_id,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(3, client_id_max))
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
        if let Some(papp) = parent_app
            && papp.parent_app_id > 0 {
                return Err(ValidError::message(
                    valid_key!("parent_app"),
                    fluent_message!("papp-id-bad",{
                        "name":&papp.name
                    }),
                )
                .into());
            }

        let (name, client_id) = self.check_app_param_valid(param).await?;
        let app_res = sqlx::query_as::<_, AppModel>(&format!(
            "select * from {} where client_id=? and status in (?,?,?)",
            AppModel::table_name(),
        ))
        .bind(&client_id)
        .bind(AppStatus::Disable as i8)
        .bind(AppStatus::Enable as i8)
        .bind(AppStatus::Init as i8)
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
        let req_res = sqlx::query_scalar::<_, u64>(&format!(
            "select req.app_id from {}  as info
                join {} as req on info.app_request_id=req.id
                where info.client_id=? and req.status=? and req.request_type=? limit 1
            ",
            AppRequestSetInfoModel::table_name(),
            AppRequestModel::table_name(),
        ))
        .bind(&client_id)
        .bind(AppRequestStatus::Pending as i8)
        .bind(AppRequestType::AppChange as i8)
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

        let res = Insert::<_,AppModel>::new()
            .set(AppModel::NAME, &name)
            .set(AppModel::PARENT_APP_ID, parent_app_id)
            .set(AppModel::CLIENT_ID, &client_id)
            .set(AppModel::STATUS, status)
            .set(AppModel::USER_ID, user_id)
            .set(AppModel::USER_APP_ID, user_app_id)
            .set(AppModel::CHANGE_USER_ID, add_user_id)
            .set(AppModel::CHANGE_TIME, time)
            .execute(&mut *db)
            .await;

        let app_id = match res {
            Err(e) => {
                db.rollback().await?;
                return Err(e.into());
            }
            Ok(mr) => mr.last_insert_id(),
        };

        let secret_data = rand_str(RandType::LowerHex, 32);
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

        let secret_data = rand_str(RandType::LowerHex, 32);
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
        let req_res = Insert::<_,AppRequestModel>::new()
            .set(AppRequestModel::PARENT_APP_ID, parent_app_id)
            .set(AppRequestModel::APP_ID, app_id)
            .set(AppRequestModel::REQUEST_TYPE, request_type)
            .set(AppRequestModel::STATUS, req_status)
            .set(AppRequestModel::REQUEST_USER_ID, user_id)
            .set(AppRequestModel::REQUEST_TIME, time)
            .execute(&mut *db)
            .await;
        let req_id = match req_res {
            Err(e) => {
                db.rollback().await?;
                return Err(e.into());
            }
            Ok(mr) => mr.last_insert_id(),
        };
        let req_res = Insert::<_,AppRequestSetInfoModel>::new()
            .set(AppRequestSetInfoModel::APP_REQUEST_ID, req_id)
            .set(AppRequestSetInfoModel::NAME, name.clone())
            .set(AppRequestSetInfoModel::CLIENT_ID, client_id.clone())
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
            let app_res = sqlx::query_as::<_, AppModel>(&format!(
                "select * from {} where client_id=? and status in (?,?,?) and id !=?",
                AppModel::table_name(),
            ))
            .bind(&client_id)
            .bind(AppStatus::Disable as i8)
            .bind(AppStatus::Enable as i8)
            .bind(AppStatus::Init as i8)
            .bind(app.id)
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
            let req_res = sqlx::query_scalar::<_, u64>(&format!(
                "select req.app_id from {}  as info
                    join {} as req on info.app_request_id=req.id
                    where info.client_id=? and req.status=? and req.request_type=? limit 1
                ",
                AppRequestSetInfoModel::table_name(),
                AppRequestModel::table_name(),
            ))
            .bind(&client_id)
            .bind(AppRequestStatus::Pending as i8)
            .bind(AppRequestType::AppChange as i8)
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
            let req_res = Update::<_,AppModel>::new()
                .set(AppModel::NAME, &name)
                .set(AppModel::CLIENT_ID, &client_id)
                .execute(&mut *db, |qb| {
                    qb.push_where().field_eq("id", app.id);
                })
                .await;
            if let Err(e) = req_res {
                db.rollback().await?;
                return Err(e.into());
            }
        }

        //废弃以前申请
        let req_status = AppRequestStatus::Invalid as i8;
        let req_res = Update::<_,AppRequestModel>::new()
            .set(AppRequestModel::STATUS, req_status)
            .execute(&mut *db, |qb| {
                qb.push_where()
                    .field_eq("app_id", app.id)
                    .push_and()
                    .field_in_copied("request_type", &[AppRequestType::AppChange as i8, AppRequestType::AppReq as i8]);
            })
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
        let req_res = Insert::<_,AppRequestModel>::new()
            .set(AppRequestModel::PARENT_APP_ID, app.parent_app_id)
            .set(AppRequestModel::APP_ID, app.id)
            .set(AppRequestModel::REQUEST_TYPE, request_type)
            .set(AppRequestModel::STATUS, req_status)
            .set(AppRequestModel::REQUEST_USER_ID, change_user_id)
            .set(AppRequestModel::REQUEST_TIME, time)
            .execute(&mut *db)
            .await;
        let req_id = match req_res {
            Err(e) => {
                db.rollback().await?;
                return Err(e.into());
            }
            Ok(mr) => mr.last_insert_id(),
        };
        let req_res = Insert::<_,AppRequestSetInfoModel>::new()
            .set(AppRequestSetInfoModel::APP_REQUEST_ID, req_id)
            .set(AppRequestSetInfoModel::NAME, name.clone())
            .set(AppRequestSetInfoModel::CLIENT_ID, client_id.clone())
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
        let req_info = match sqlx::query_as::<_, AppRequestSetInfoModel>(&format!(
            "select * from {} where app_request_id=?",
            AppRequestSetInfoModel::table_name(),
        ))
        .bind(req.id)
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
        let app_res = sqlx::query_as::<_, AppModel>(&format!(
            "select * from {} where client_id=? and status in (?,?,?) and id !=?",
            AppModel::table_name(),
        ))
        .bind(&req_info.client_id)
        .bind(AppStatus::Disable as i8)
        .bind(AppStatus::Enable as i8)
        .bind(AppStatus::Init as i8)
        .bind(app.id)
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

        let status = if confirm_status == AppRequestStatus::Approved {
            AppStatus::Enable as i8
        } else {
            AppStatus::Disable as i8
        };
        let req_res = Update::<_,AppModel>::new()
            .set(AppModel::NAME, req_info.name.clone())
            .set(AppModel::CLIENT_ID, req_info.client_id.clone())
            .set(AppModel::STATUS, status)
            .set(AppModel::CHANGE_USER_ID, confirm_user_id)
            .set(AppModel::CHANGE_TIME, time)
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("id", app.id);
            })
            .await;
        if let Err(e) = req_res {
            db.rollback().await?;
            return Err(e.into());
        }

        //废弃以前申请
        let confirm_status = confirm_status as i8;
        let confirm_note = confirm_note.to_string();
        let req_res = Update::<_,AppRequestModel>::new()
            .set(AppRequestModel::STATUS, confirm_status)
            .set(AppRequestModel::CONFIRM_USER_ID, confirm_user_id)
            .set(AppRequestModel::CONFIRM_TIME, time)
            .set(AppRequestModel::CONFIRM_NOTE, confirm_note)
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("app_id", req.id);
            })
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

        let req_res = Update::<_,AppModel>::new()
            .set(AppModel::STATUS, status)
            .set(AppModel::CHANGE_USER_ID, disable_user_id)
            .set(AppModel::CHANGE_TIME, time)
            .execute(&mut *db, |qb| {
                qb.push_where()
                    .field_eq("id", app.id)
                    .push_or()
                    .field_eq("parent_app_id", app.id);
            })
            .await;
        if let Err(e) = req_res {
            db.rollback().await?;
            return Err(e.into());
        }

        //废弃以前申请
        let confirm_status = AppRequestStatus::Invalid as i8;
        let req_res = Update::<_,AppRequestModel>::new()
            .set(AppRequestModel::STATUS, confirm_status)
            .set(AppRequestModel::CONFIRM_USER_ID, disable_user_id)
            .set(AppRequestModel::CONFIRM_TIME, time)
            .execute(&mut *db, |qb| {
                qb.push_where()
                    .push("(")
                    .field_eq("app_id", app.id)
                    .push(format!(" OR app_id IN (SELECT id FROM {}", AppModel::table_name()))
                    .push_where()
                    .field_eq("parent_app_id", app.id)
                    .push("))")
                    .push_and()
                    .field_eq("status", AppRequestStatus::Pending as i8);
            })
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
            let sub_app = sqlx::query_as::<_, (u64,String)>(&format!(
                "select * from {} where  parent_app_id =? and status = ? and id>?  order by id asc limit 100",
                AppModel::table_name(),
            ))
            .bind(app.id)
            .bind(AppStatus::Disable as i8)
            .bind(clear_start_id)
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

        let sub_app_count = sqlx::query_scalar::<_, i64>(&format!(
            "select count(*) as total from {} where  parent_app_id =? and status in (?,?,?)",
            AppModel::table_name(),
        ))
        .bind(app.id)
        .bind(AppStatus::Enable as i8)
        .bind(AppStatus::Init as i8)
        .bind(AppStatus::Disable as i8)
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

        let req_res = Update::<_,AppModel>::new()
            .set(AppModel::STATUS, status)
            .set(AppModel::CHANGE_USER_ID, delete_user_id)
            .set(AppModel::CHANGE_TIME, time)
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("id", app.id);
            })
            .await;
        if let Err(e) = req_res {
            db.rollback().await?;
            return Err(e.into());
        }

        //废弃以前申请
        let confirm_status = AppRequestStatus::Invalid as i8;
        let req_res = Update::<_,AppRequestModel>::new()
            .set(AppRequestModel::STATUS, confirm_status)
            .set(AppRequestModel::CONFIRM_USER_ID, delete_user_id)
            .set(AppRequestModel::CONFIRM_TIME, time)
            .execute(&mut *db, |qb| {
                qb.push_where()
                    .field_eq("app_id", app.id)
                    .push_and()
                    .field_eq("status", AppRequestStatus::Pending as i8);
            })
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
            None => rand_str(RandType::LowerHex, 32),
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
            None => rand_str(RandType::LowerHex, 32),
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
            None => rand_str(RandType::LowerHex, 32),
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
