use crate::{
    dao::{logger::AppRequestLog, AppResult},
    model::{
        AppFeatureModel, AppFeatureModelRef, AppFeatureStatus, AppModel, AppRequestFeatureModel,
        AppRequestFeatureModelRef, AppRequestModel, AppRequestModelRef, AppRequestStatus,
        AppRequestType,
    },
};
use lsys_core::db::SqlQuote;
use lsys_core::db::{Insert, ModelTableName, Update};
use lsys_core::{fluent_message, now_time, RequestEnv};
use lsys_core::{model_option_set, sql_format};

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
        let featuer_data = featuer_data
            .iter()
            .map(|e| self.exter_feature_key(e))
            .collect::<Vec<String>>();
        app.app_status_check()?;
        let req_res = sqlx::query_scalar::<_, String>(&sql_format!(
            "select feature_key from {} where app_id={} and feature_key in ({}) and status={}",
            AppFeatureModel::table_name(),
            app.id,
            featuer_data,
            AppFeatureStatus::Enable as i8
        ))
        .fetch_all(&self.db)
        .await;
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
        let time = now_time()?;
        let mut db = self.db.begin().await?;

        let req_status = AppRequestStatus::Pending as i8;
        let request_type = AppRequestType::ExterFeatuer as i8;
        let idata = model_option_set!(AppRequestModelRef,{
            parent_app_id:app.parent_app_id,
            app_id:app.id,
            request_type:request_type,
            status:req_status,
            request_user_id:req_user_id,
            request_time:time,
        });
        let req_res = Insert::<sqlx::MySql, AppRequestModel, _>::new(idata)
            .execute(&mut *db)
            .await;
        let req_id = match req_res {
            Err(e) => {
                db.rollback().await?;
                return Err(e.into());
            }
            Ok(mr) => mr.last_insert_id(),
        };
        let feature_data = req_feature
            .iter()
            .map(|e| e.trim())
            .collect::<Vec<&str>>()
            .join(",");
        let idata = model_option_set!(AppRequestFeatureModelRef,{
            app_request_id:req_id,
            feature_data:feature_data,
        });
        let req_res = Insert::<sqlx::MySql, AppRequestFeatureModel, _>::new(idata)
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
                    req_data: Some(&feature_data),
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

        let feature_res = sqlx::query_scalar::<_, String>(&sql_format!(
            "select feature_data from {} where app_request_id={}",
            AppRequestFeatureModel::table_name(),
            req.id,
        ))
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
            let confirm_note = confirm_note.to_owned();
            let change = model_option_set!(AppRequestModelRef,{
                status:status,
                confirm_user_id:confirm_user_id,
                confirm_time:time,
                confirm_note:confirm_note,
            });
            Update::<sqlx::MySql, AppRequestModel, _>::new(change)
                .execute_by_pk(req, &self.db)
                .await?;
            return Ok(());
        }

        let req_res = sqlx::query_as::<_, (u64, String, i8)>(&sql_format!(
            "select id,feature_key,status from {} where app_id={} and feature_key in ({}) ",
            AppFeatureModel::table_name(),
            app.id,
            find_data
        ))
        .fetch_all(&self.db)
        .await?;
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
            let change = model_option_set!(AppFeatureModelRef,{
                status:set_status,
                change_user_id:confirm_user_id,
                change_time:time
            });
            let cres = Update::<sqlx::MySql, AppFeatureModel, _>::new(change)
                .execute_by_where(
                    &lsys_core::db::WhereOption::Where(sql_format!("id in ({})", set_status_id)),
                    &mut *db,
                )
                .await;
            if let Err(err) = cres {
                db.rollback().await?;
                return Err(err.into());
            }
        }

        let mut iarr = vec![];
        for tmp in set_val.iter() {
            iarr.push(model_option_set!(AppFeatureModelRef,{
                app_id:app.id,
                feature_key:tmp,
                status:set_status,
                change_user_id:confirm_user_id,
                change_time:time
            }));
        }
        let cres = Insert::<sqlx::MySql, AppFeatureModel, _>::new_vec(iarr)
            .execute(&mut *db)
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
        let cres = Update::<sqlx::MySql, AppRequestModel, _>::new(change)
            .execute_by_pk(req, &mut *db)
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
    //列出支持外部功能
    pub async fn exter_feature_list(&self, app: &AppModel) -> AppResult<Vec<String>> {
        let key = AppRequestType::ExterFeatuer.feature_key();
        let len = key.len() + 1;
        let req_res = sqlx::query_scalar::<_, String>(&sql_format!(
            "select feature_key from {} where app_id={} and status ={} and feature_key like {}",
            AppFeatureModel::table_name(),
            app.id,
            AppFeatureStatus::Enable as i8,
            format!("{}%", key)
        ))
        .fetch_all(&self.db)
        .await?;
        let mut out = vec![];
        for tmp in req_res {
            out.push(tmp[len..].to_string());
        }
        Ok(out)
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
            let papp = self.find_by_id(&app.parent_app_id).await?;
            self.inner_feature_sub_app_check(&papp).await?;
            self.feature_check(&papp, check_key).await?;
        }
        self.feature_check(app, check_key).await
    }
}
