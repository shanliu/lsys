use crate::{
    dao::{logger::AppRequestLog, AppResult},
    model::{
        AppFeatureModel, AppFeatureModelRef, AppFeatureStatus, AppModel, AppRequestModel,
        AppRequestModelRef, AppRequestStatus, AppRequestType,
    },
};
use lsys_core::{
    db::{Insert, ModelTableName, Update},
    string_clear, StringClear, STRING_CLEAR_XSS,
};
use lsys_core::{
    db::{SqlQuote, WhereOption},
    STRING_CLEAR_FORMAT,
};
use lsys_core::{fluent_message, now_time, RequestEnv};
use lsys_core::{model_option_set, sql_format};

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
            "select id,status from {} where app_id={} and feature_key = {}",
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
            "select id from {} where  parent_app_id={} and app_id={} and request_type={} and status={} ",
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

        let idata = model_option_set!(AppRequestModelRef,{
            parent_app_id:app.parent_app_id,
            app_id:app.id,
            request_type:request_type,
            status:req_status,
            request_user_id:req_user_id,
            request_time:time,
        });
        let req_id = Insert::<AppRequestModel, _>::new(idata)
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
                    let change = model_option_set!(AppFeatureModelRef,{
                        status:set_status,
                        change_user_id:confirm_user_id,
                        change_time:time
                    });
                    let cres = Update::<AppFeatureModel, _>::new(change)
                        .execute_by_where(&WhereOption::Where(sql_format!("id={}", fid)), &mut *db)
                        .await;
                    if let Err(err) = cres {
                        db.rollback().await?;
                        return Err(err.into());
                    }
                }
            }
            Err(sqlx::Error::RowNotFound) => {
                let iarr = model_option_set!(AppFeatureModelRef,{
                    app_id:app.id,
                    feature_key:fkey,
                    status:set_status,
                    change_user_id:confirm_user_id,
                    change_time:time
                });
                let cres = Insert::<AppFeatureModel, _>::new(iarr)
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
