use crate::{
    dao::{AppResult, logger::AppRequestLog},
    model::{
        AppFeatureModel, AppFeatureStatus, AppModel, AppRequestFeatureModel, AppRequestModel,
        AppRequestStatus, AppRequestType,
    },
};
use lsys_core::db::{BatchInsert, Insert, QueryBuilderExt, TableMeta, Update};
use lsys_core::fluent_message;
use lsys_core::utils::{
    RequestEnv, STRING_CLEAR_FORMAT, STRING_CLEAR_XSS, StringClear, now_time, string_clear,
};
use sqlx::{MySql, QueryBuilder};

use super::{App, AppError};

// 发邮件 发短信等独立于APP的功能管理

impl App {
    pub(crate) fn exter_feature_key(&self, key: &str) -> String {
        format!("{}-{}", AppRequestType::ExterFeatuer.feature_key(), key)
    }
    //申请外部功能
    pub async fn exter_feature_request(
        &self,
        app: &AppModel,
        featuer_data: &[&str],
        req_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        self.exter_feature_param_valid(featuer_data).await?;
        let featuer_data = featuer_data
            .iter()
            .map(|e| self.exter_feature_key(e))
            .collect::<Vec<String>>();
        app.app_status_check()?;
        let req_res = {
            let mut qb = QueryBuilder::<MySql>::new(format!(
                "select feature_key from {}",
                AppFeatureModel::table_name()
            ));
            qb.push_where().field_eq("app_id", app.id);
            qb.push_and().field_in_string("feature_key", &featuer_data);
            qb.push_and()
                .field_eq("status", AppFeatureStatus::Enable as i8);
            qb.build_query_scalar::<String>().fetch_all(&self.db).await
        };
        let req_feature = match req_res {
            Ok(dat) => {
                let mut out = vec![];
                for tmp in featuer_data.iter() {
                    if !dat.contains(tmp) && !tmp.is_empty() {
                        out.push(tmp.to_owned());
                    }
                }
                out
            }
            Err(err) => {
                return Err(err.into());
            }
        };
        if req_feature.is_empty() {
            return Ok(());
        }

        let req_status = AppRequestStatus::Pending as i8;
        let request_type = AppRequestType::ExterFeatuer as i8;
        let need_feature_data = req_feature.iter().map(|e| e.trim()).collect::<Vec<&str>>();

        let req_res = sqlx::query_scalar::<_, String>(&format!(
            "select reqf.feature_data from {} as req join {} reqf on req.id=reqf.app_request_id
             where req.parent_app_id=? and req.app_id=? and req.request_type=? and req.status=? limit 1",
            AppRequestModel::table_name(),
            AppRequestFeatureModel::table_name(),
        ))
        .bind(app.parent_app_id)
        .bind(app.id)
        .bind(request_type)
        .bind(req_status)
        .fetch_one(&self.db)
        .await;
        match req_res {
            Ok(req_feature_data) => {
                let mut bad_req = vec![];
                for tmp in req_feature_data.split(',') {
                    if need_feature_data.contains(&tmp) {
                        bad_req.push(tmp);
                    }
                }
                if !bad_req.is_empty() {
                    return Err(AppError::System(
                        fluent_message!("app-req-exist-exter-feature",{
                           "bad_item":bad_req.join(",")
                        }),
                    ));
                }
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        };

        let time = now_time()?;
        let mut db = self.db.begin().await?;

        let req_res = Insert::<_, AppRequestModel>::new()
            .set(AppRequestModel::PARENT_APP_ID, app.parent_app_id)
            .set(AppRequestModel::APP_ID, app.id)
            .set(AppRequestModel::REQUEST_TYPE, request_type)
            .set(AppRequestModel::STATUS, req_status)
            .set(AppRequestModel::REQUEST_USER_ID, req_user_id)
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
        let need_feature_data_str = need_feature_data.join(",");
        let req_res = Insert::<_, AppRequestFeatureModel>::new()
            .set(AppRequestFeatureModel::APP_REQUEST_ID, req_id)
            .set(
                AppRequestFeatureModel::FEATURE_DATA,
                need_feature_data_str.clone(),
            )
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
                    parent_app_id: app.parent_app_id,
                    app_id: app.id,
                    user_id: app.user_id,
                    request_type,
                    status: req_status,
                    req_data: Some(&need_feature_data_str),
                    action: "exter_feature_request",
                },
                Some(req_id),
                Some(req_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    //审核 外部功能
    pub async fn exter_feature_confirm(
        &self,
        app: &AppModel,
        req: &AppRequestModel,
        mut req_status: AppRequestStatus,
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
        if !AppRequestType::ExterFeatuer.eq(req.request_type) || req.app_id != app.id {
            return Err(AppError::System(fluent_message!("app-req-bad")));
        }
        if ![AppRequestStatus::Approved, AppRequestStatus::Rejected].contains(&req_status) {
            return Err(AppError::System(fluent_message!("app-req-status-invalid")));
        }

        let feature_res = sqlx::query_scalar::<_, String>(&format!(
            "select feature_data from {} where app_request_id=? limit 1",
            AppRequestFeatureModel::table_name(),
        ))
        .bind(req.id)
        .fetch_one(&self.db)
        .await;
        let find_data = match feature_res {
            Ok(tmp) => {
                let out = tmp
                    .split(",")
                    .filter(|e| !e.is_empty())
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>();
                if out.is_empty() {
                    req_status = AppRequestStatus::Rejected;
                }
                out
            }
            Err(sqlx::Error::RowNotFound) => {
                req_status = AppRequestStatus::Rejected;
                vec![]
            }
            Err(err) => {
                return Err(err.into());
            }
        };
        let time = now_time()?;
        if req_status == AppRequestStatus::Rejected {
            //驳回
            let status = req_status as i8;
            Update::<_, AppRequestModel>::new()
                .set(AppRequestModel::STATUS, status)
                .set(AppRequestModel::CONFIRM_USER_ID, confirm_user_id)
                .set(AppRequestModel::CONFIRM_TIME, time)
                .set(AppRequestModel::CONFIRM_NOTE, confirm_note.clone())
                .execute(&self.db, |qb| {
                    qb.push_where().field_eq("id", req.id);
                })
                .await?;
            return Ok(());
        }

        let req_res = {
            let mut qb = QueryBuilder::<MySql>::new(format!(
                "select id,feature_key,status from {}",
                AppFeatureModel::table_name()
            ));
            qb.push_where().field_eq("app_id", app.id);
            qb.push_and().field_in_string("feature_key", &find_data);
            qb.build_query_as::<(u64, String, i8)>()
                .fetch_all(&self.db)
                .await?
        };
        let mut set_val = vec![];
        for tmp in find_data.iter() {
            let stmp = tmp.to_owned();
            if !req_res.iter().any(|t| t.1 == stmp) {
                set_val.push(stmp);
            }
        }

        let set_status = AppFeatureStatus::Enable as i8;
        let mut db = self.db.begin().await?;

        let set_status_id = req_res
            .iter()
            .filter(|e| !AppFeatureStatus::Enable.eq(e.2))
            .map(|e| e.0)
            .collect::<Vec<u64>>();
        if !set_status_id.is_empty() {
            let cres = Update::<_, AppFeatureModel>::new()
                .set(AppFeatureModel::STATUS, set_status)
                .set(AppFeatureModel::CHANGE_USER_ID, confirm_user_id)
                .set(AppFeatureModel::CHANGE_TIME, time)
                .execute(&mut *db, |qb| {
                    qb.push_where().field_in_copied("id", &set_status_id);
                })
                .await;
            if let Err(err) = cres {
                db.rollback().await?;
                return Err(err.into());
            }
        }

        let mut batch_insert = BatchInsert::<_, AppFeatureModel>::with_capacity(set_val.len());
        for tmp in set_val.iter() {
            batch_insert = batch_insert.push(
                Insert::<_, AppFeatureModel>::new()
                    .set(AppFeatureModel::APP_ID, app.id)
                    .set(AppFeatureModel::FEATURE_KEY, tmp)
                    .set(AppFeatureModel::STATUS, set_status)
                    .set(AppFeatureModel::CHANGE_USER_ID, confirm_user_id)
                    .set(AppFeatureModel::CHANGE_TIME, time),
            );
        }
        let cres = batch_insert.execute(&mut *db).await;
        if let Err(err) = cres {
            db.rollback().await?;
            return Err(err.into());
        }

        let status = AppRequestStatus::Approved as i8;

        let cres = Update::<_, AppRequestModel>::new()
            .set(AppRequestModel::STATUS, status)
            .set(AppRequestModel::CONFIRM_USER_ID, confirm_user_id)
            .set(AppRequestModel::CONFIRM_TIME, time)
            .set(AppRequestModel::CONFIRM_NOTE, confirm_note.clone())
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("id", req.id);
            })
            .await;
        if let Err(err) = cres {
            db.rollback().await?;
            return Err(err.into());
        }

        db.commit().await?;

        self.feature_cache.del(&app.id).await;

        self.logger
            .add(
                &AppRequestLog {
                    parent_app_id: app.parent_app_id,
                    app_id: app.id,
                    user_id: req.request_user_id,
                    request_type: req.request_type,
                    status,
                    req_data: Some(&find_data.join(",")),
                    action: "exter_feature_confirm",
                },
                Some(req.id),
                Some(confirm_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    //外部功能是否可用检测
    //仅用在后台,不带缓存:外部用cache下的
    pub async fn exter_feature_check(
        &self,
        app: &AppModel,
        featuer_data: &[&str],
    ) -> AppResult<()> {
        let feature_key = featuer_data
            .iter()
            .map(|e| self.exter_feature_key(e))
            .collect::<Vec<String>>();
        let check_key = &feature_key.iter().map(|e| e.as_str()).collect::<Vec<_>>();
        if app.parent_app_id > 0 {
            let papp = self.find_by_id(app.parent_app_id).await?;
            self.feature_check(&papp, check_key).await?;
        }
        self.feature_check(app, check_key).await
    }
}
