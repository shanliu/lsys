use crate::{
    dao::{logger::AppRequestLog, AppResult},
    model::{
        AppFeatureModel, AppFeatureStatus, AppModel, AppRequestModel,
        AppRequestStatus, AppRequestType,
    },
};
use lsys_core::{db::{Insert, TableMeta, SqlSuffix, Update}};
use lsys_core::db::SqlQuote;
use lsys_core::fluent_message;
use lsys_core::sql_format;
use lsys_core::utils::{
    now_time, string_clear, RequestEnv, StringClear, STRING_CLEAR_FORMAT, STRING_CLEAR_XSS,
};

use super::{App, AppError};

impl App {
    pub(crate) async fn inner_feature_request(
        &self,
        app: &AppModel,
        inner_request_type: AppRequestType,
        req_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        app.app_status_check()?;
        let req_res = sqlx::query_scalar::<_, i8>(&sql_format!(
            "select status from {} where app_id={} and feature_key = {} limit 1",
            AppFeatureModel::table_name(),
            app.id,
            inner_request_type.feature_key(),
        ))
        .fetch_one(&self.db)
        .await;
        match req_res {
            Ok(fstatus) => {
                if AppFeatureStatus::Enable.eq(fstatus) {
                    return Ok(());
                }
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        };

        let req_status = AppRequestStatus::Pending as i8;
        let request_type = inner_request_type as i8;

        let req_res = sqlx::query_scalar::<_, u64>(&sql_format!(
            "select id from {} where  parent_app_id={} and app_id={} and request_type={} and status={} limit 1",
            AppRequestModel::table_name(),
            app.parent_app_id,
            app.id,
            request_type,
            req_status
        ))
        .fetch_one(&self.db)
        .await;
        match req_res {
            Ok(_) => {
                return Ok(());
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        };

        let time = now_time()?;

        let req_id = Insert::<_,AppRequestModel>::new()
            .set(AppRequestModel::PARENT_APP_ID, app.parent_app_id)
            .set(AppRequestModel::APP_ID, app.id)
            .set(AppRequestModel::REQUEST_TYPE, request_type)
            .set(AppRequestModel::STATUS, req_status)
            .set(AppRequestModel::REQUEST_USER_ID, req_user_id)
            .set(AppRequestModel::REQUEST_TIME, time)
            .execute(&self.db)
            .await?
            .last_insert_id();

        self.logger
            .add(
                &AppRequestLog {
                    user_id: app.user_id,
                    action: "inner_feature_request",
                    parent_app_id: app.parent_app_id,
                    app_id: app.id,
                    request_type,
                    status: req_status,
                    req_data: None,
                },
                Some(req_id),
                Some(req_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    //审核 feature
    pub(crate) async fn inner_feature_confirm(
        &self,
        app: &AppModel,
        req: &AppRequestModel,
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
        if !AppRequestStatus::Pending.eq(req.status) {
            return Ok(());
        }

        if req.app_id != app.id {
            return Err(AppError::System(fluent_message!("app-req-bad-app")));
        }
        let req_type = match AppRequestType::try_from(req.request_type) {
            Ok(t) => t,
            Err(e) => {
                return Err(AppError::System(fluent_message!("app-req-bad", e)));
            }
        };
        if ![AppRequestStatus::Approved, AppRequestStatus::Rejected].contains(&req_status) {
            return Err(AppError::System(fluent_message!("app-req-status-invalid")));
        }

        let time = now_time()?;
        if req_status == AppRequestStatus::Rejected {
            //驳回
            let status = req_status as i8;
            let confirm_note = confirm_note.to_owned();
            Update::<_,AppRequestModel>::new()
                .set(AppRequestModel::STATUS, status)
                .set(AppRequestModel::CONFIRM_USER_ID, confirm_user_id)
                .set(AppRequestModel::CONFIRM_TIME, time)
                .set(AppRequestModel::CONFIRM_NOTE, confirm_note)
                .execute(SqlSuffix::Where(&sql_format!("id={}", req.id)), &self.db)
                .await?;
            return Ok(());
        }
        let fkey = req_type.feature_key().to_string();
        let req_res = sqlx::query_as::<_, (u64, i8)>(&sql_format!(
            "select id,status from {} where app_id={} and feature_key = {}",
            AppFeatureModel::table_name(),
            app.id,
            &fkey
        ))
        .fetch_one(&self.db)
        .await;

        let set_status = AppFeatureStatus::Enable as i8;
        let mut db = self.db.begin().await?;

        match req_res {
            Ok((fid, fstatus)) => {
                if !AppFeatureStatus::Enable.eq(fstatus) {
                    let cres = Update::<_,AppFeatureModel>::new()
                        .set(AppFeatureModel::STATUS, set_status)
                        .set(AppFeatureModel::CHANGE_USER_ID, confirm_user_id)
                        .set(AppFeatureModel::CHANGE_TIME, time)
                        .execute(SqlSuffix::Where(&sql_format!("id={}", fid)), &mut *db)
                        .await;
                    if let Err(err) = cres {
                        db.rollback().await?;
                        return Err(err.into());
                    }
                }
            }
            Err(sqlx::Error::RowNotFound) => {
                let cres = Insert::<_,AppFeatureModel>::new()
                    .set(AppFeatureModel::APP_ID, app.id)
                    .set(AppFeatureModel::FEATURE_KEY, fkey)
                    .set(AppFeatureModel::STATUS, set_status)
                    .set(AppFeatureModel::CHANGE_USER_ID, confirm_user_id)
                    .set(AppFeatureModel::CHANGE_TIME, time)
                    .execute(&mut *db)
                    .await;
                if let Err(err) = cres {
                    db.rollback().await?;
                    return Err(err.into());
                }
            }
            Err(err) => {
                return Err(err.into());
            }
        }

        let status = AppRequestStatus::Approved as i8;
        let confirm_note = confirm_note.to_owned();
        let cres = Update::<_,AppRequestModel>::new()
            .set(AppRequestModel::STATUS, status)
            .set(AppRequestModel::CONFIRM_USER_ID, confirm_user_id)
            .set(AppRequestModel::CONFIRM_TIME, time)
            .set(AppRequestModel::CONFIRM_NOTE, confirm_note)
            .execute(SqlSuffix::Where(&sql_format!("id={}", req.id)), &mut *db)
            .await;
        if let Err(err) = cres {
            db.rollback().await?;
            return Err(err.into());
        }

        db.commit().await?;

        self.logger
            .add(
                &AppRequestLog {
                    action: "inner_feature_confirm",
                    parent_app_id: app.parent_app_id,
                    app_id: app.id,
                    user_id: req.request_user_id,
                    request_type: req.request_type,
                    status,
                    req_data: None,
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
