use std::{collections::HashMap, sync::Arc};

use crate::model::{AppStatus, AppSubAppsModel, AppSubAppsStatus, AppsModel, AppsModelRef};
use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    fluent_message, impl_dao_fetch_map_by_vec, now_time, PageParam, RemoteNotify, RequestEnv,
};

use lsys_logger::dao::ChangeLogger;
use regex::Regex;
use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{
    model_option_set, sql_format, Insert, ModelTableName, Select, Update, WhereOption,
};
use sqlx_model::{SqlExpr, SqlQuote};

use super::{
    super::{AppsError, AppsResult},
    SubApps,
};
use super::{range_client_key, AppLog};
pub struct Apps {
    db: Pool<MySql>,
    pub(crate) cache: Arc<LocalCache<String, AppsModel>>,
    logger: Arc<ChangeLogger>,
    sub_app: Arc<SubApps>,
}

#[derive(Clone, Debug)]
pub struct AppDataWhere<'t> {
    pub user_id: &'t Option<u64>,
    pub status: &'t Option<Vec<AppStatus>>,
    pub client_ids: &'t Option<Vec<String>>,
    pub app_ids: &'t Option<Vec<u64>>,
}

impl Apps {
    pub fn new(
        db: Pool<MySql>,
        sub_app: Arc<SubApps>,
        remote_notify: Arc<RemoteNotify>,
        config:LocalCacheConfig,
        logger: Arc<ChangeLogger>,
    ) -> Self {
        Self {
            db,
            cache:Arc::new(LocalCache::new(remote_notify, config)),
            logger,
            sub_app,
        }
    }
    fn app_data_sql(&self, app_where: &AppDataWhere) -> Option<String> {
        let mut sql_vec = vec![];
        if let Some(ref rid) = app_where.user_id {
            sql_vec.push(sql_format!("user_id = {}", rid));
        };
        if let Some(ref rid) = app_where.status {
            if rid.is_empty() {
                return None;
            } else {
                sql_vec.push(sql_format!(
                    " status in ({})",
                    rid.iter().map(|e| *e as i8).collect::<Vec<i8>>()
                ));
            }
        }
        if let Some(ref rid) = app_where.client_ids {
            if rid.is_empty() {
                return None;
            } else {
                sql_vec.push(sql_format!(" client_id in ({})", rid));
            }
        }
        if let Some(ref rid) = app_where.app_ids {
            if rid.is_empty() {
                return None;
            } else {
                sql_vec.push(sql_format!(" id in ({})", rid));
            }
        }
        Some(sql_vec.join(" and "))
    }

    //app 列表数据
    pub async fn app_data<'t>(
        &self,
        app_where: &AppDataWhere<'t>,
        page: &Option<PageParam>,
    ) -> AppsResult<Vec<AppsModel>> {
        let where_sql = match self.app_data_sql(app_where) {
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
        let sql = if !where_sql.is_empty() {
            WhereOption::Where(where_sql + page_sql.as_str())
        } else if page.is_some() {
            WhereOption::NoWhere(page_sql)
        } else {
            WhereOption::None
        };
        let data = Select::type_new::<AppsModel>()
            .fetch_all_by_where::<AppsModel, _>(&sql, &self.db)
            .await?;
        Ok(data)
    }
    pub async fn app_count<'t>(&self, app_where: &AppDataWhere<'t>) -> AppsResult<i64> {
        let where_sql = match self.app_data_sql(app_where) {
            Some(sql) => {
                if sql.is_empty() {
                    "".to_string()
                } else {
                    format!("where {}", sql)
                }
            }
            None => return Ok(0),
        };
        let sql = sql_format!(
            "select count(*) as total from {} {}",
            AppsModel::table_name(),
            SqlExpr(where_sql)
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        AppsModel,
        AppsResult<HashMap<u64, AppsModel>>,
        id,
        id,
        "id in ({id}) "
    );
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        AppsModel,
        AppsResult<AppsModel>,
        id,
        "id={id}"
    );
    //重设secret
    pub async fn reset_secret<'t>(
        &self,
        app: &AppsModel,
        change_user_id: &u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AppsResult<(String, String)> {
        let time = now_time().unwrap_or_default();
        let client_secret = range_client_key();
        let change_user_id = change_user_id.to_owned();
        let oauth_secret = range_client_key();
        let change = sqlx_model::model_option_set!(AppsModelRef,{
            client_secret:client_secret,
            oauth_secret:oauth_secret,
            change_user_id:change_user_id,
            change_time:time,
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::<sqlx::MySql, AppsModel, _>::new(change)
            .execute_by_pk(app, &mut db)
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e.into());
        }
        db.commit().await?;
        self.cache.clear(&app.client_id).await;

        self.logger
            .add(
                &AppLog {
                    action: "reset_secret",
                    status: app.status,
                    name: app.name.to_owned(),
                    client_id: app.client_id.to_owned(),
                    client_secret: app.client_secret.to_owned(),
                    callback_domain: app.callback_domain.to_owned(),
                },
                &Some(app.id),
                &Some(app.user_id),
                &Some(change_user_id),
                None,
                env_data,
            )
            .await;

        Ok((client_secret, oauth_secret))
    }
    //确认APP
    pub async fn confirm_app<'t>(
        &self,
        app: &AppsModel,
        user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AppsResult<u64> {
        if AppStatus::Ok.eq(app.status) {
            return Ok(0);
        }
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(AppsModelRef,{
            status:AppStatus::Ok as i8,
            confirm_time:time,
            confirm_user_id:user_id
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::<sqlx::MySql, AppsModel, _>::new(change)
            .execute_by_pk(app, &mut db)
            .await;
        let res = match tmp {
            Ok(e) => e,
            Err(ie) => {
                db.rollback().await?;
                return Err(ie.into());
            }
        };
        db.commit().await?;
        self.cache.clear(&app.client_id).await;

        self.logger
            .add(
                &AppLog {
                    action: "confirm",
                    status: AppStatus::Ok as i8,
                    name: app.name.to_owned(),
                    client_id: app.client_id.to_owned(),
                    client_secret: app.client_secret.to_owned(),
                    callback_domain: app.callback_domain.to_owned(),
                },
                &Some(app.id),
                &Some(app.user_id),
                &Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok(res.rows_affected())
    }
    pub async fn app_status(
        &self,
        app: &AppsModel,
        status: bool,
        change_user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> AppsResult<()> {
        if !AppStatus::Delete.eq(app.status) && !AppStatus::Ok.eq(app.status) {
            return Err(AppsError::System(fluent_message!("app-not-confirm",
                {"name":&app.name}
            )));
        }
        if (status && AppStatus::Ok.eq(app.status)) || (!status && AppStatus::Delete.eq(app.status))
        {
            return Ok(());
        }

        let db_status = if status {
            AppStatus::Ok as i8
        } else {
            AppStatus::Delete as i8
        };
        let change_time = now_time().unwrap_or_default();
        let change_user_id = change_user_id.to_owned();
        let change = sqlx_model::model_option_set!(AppsModelRef,{
            status:db_status,
            change_user_id:change_user_id,
            change_time:change_time,
        });
        Update::<sqlx::MySql, AppsModel, _>::new(change)
            .execute_by_pk(app, &self.db)
            .await?;
        if !status {
            self.clear_app_cache(app).await;
        }

        self.logger
            .add(
                &AppLog {
                    action: "status",
                    status: db_status,
                    name: app.name.to_owned(),
                    client_id: app.client_id.to_owned(),
                    client_secret: app.client_secret.to_owned(),
                    callback_domain: app.callback_domain.to_owned(),
                },
                &Some(app.id),
                &Some(app.user_id),
                &Some(change_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
    async fn clear_app_cache(&self, app: &AppsModel) {
        //清除查询缓存
        self.cache.clear(&app.client_id).await;
        let sql = sql_format!(
            "select 
                app_id
                    from {} as app_sub 
                    where sub_app_id={} and status in ({})",
            AppSubAppsModel::table_name(),
            app.id,
            &[
                AppSubAppsStatus::Enable as i8,
                AppSubAppsStatus::Disable as i8,
            ]
        );
        let query = sqlx::query_scalar::<_, u64>(&sql);
        if let Ok(tmps) = query.fetch_all(&self.db).await {
            for tmp in tmps {
                self.sub_app
                    .cache()
                    .clear_sub_secret_cache(tmp, &app.client_id)
                    .await;
            }
        }
    }
    //添加内部APP
    #[allow(clippy::too_many_arguments)]
    pub async fn innernal_app_edit<'t>(
        &self,
        app: &AppsModel,
        name: String,
        client_id: String,
        domain: String,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AppsResult<u64> {
        let (name, client_id, domain) = self.check_app_param(name, client_id, domain)?;
        let app_res = Select::type_new::<AppsModel>()
            .fetch_one_by_where::<AppsModel, _>(
                &WhereOption::Where(sql_format!("id!={} and client_id={}", app.id, client_id)),
                &self.db,
            )
            .await;
        match app_res {
            Ok(app) => {
                return Err(AppsError::System(fluent_message!("app-client-id-exits",
                    //"client id {$client_id} already used",
                    {
                        "client_id":app.client_id,
                        "other_name":app.name
                    }
                )));
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }
        let change_time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(AppsModelRef,{
            name:name,
            client_id:client_id,
            callback_domain:domain,
            change_user_id:change_user_id,
            change_time:change_time,
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::<sqlx::MySql, AppsModel, _>::new(change)
            .execute_by_pk(app, &mut db)
            .await;
        let res = match tmp {
            Ok(e) => e,
            Err(ie) => {
                db.rollback().await?;
                return Err(ie.into());
            }
        };
        db.commit().await?;

        if client_id != app.client_id {
            //清除查询缓存
            self.clear_app_cache(app).await;
        }

        self.logger
            .add(
                &AppLog {
                    action: "edit",
                    name,
                    status: app.status,
                    client_id,
                    client_secret: app.client_secret.to_owned(),
                    callback_domain: domain,
                },
                &Some(app.id),
                &Some(app.user_id),
                &Some(change_user_id),
                None,
                env_data,
            )
            .await;

        Ok(res.rows_affected())
    }
    //添加内部APP
    #[allow(clippy::too_many_arguments)]
    pub async fn innernal_app_add<'t>(
        &self,
        uesr_id: u64,
        add_user_id: u64,
        name: String,
        client_id: String,
        domain: String,
        status: AppStatus,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AppsResult<u64> {
        self.add_app(
            uesr_id,
            name,
            client_id,
            domain,
            status,
            add_user_id,
            transaction,
            env_data,
        )
        .await
    }
    fn check_app_param(
        &self,
        name: String,
        client_id: String,
        domain: String,
    ) -> AppsResult<(String, String, String)> {
        let domain = domain.trim().to_string();
        if !domain.is_empty() {
            let ipre = Regex::new(r"^[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}(:[\d]{1,5})?$")
                .map_err(|e| AppsError::System(fluent_message!("auth-alpha-ip-error", e)))?;
            let dre = Regex::new(
                r"^[0-9a-zA-Z]{0,1}[0-9a-zA-Z-]*(\.[0-9a-zA-Z-]*)*(\.[0-9a-zA-Z]*)+(:[\d]{1,5})?$",
            )
            .map_err(|e| AppsError::System(fluent_message!("auth-alpha-domain-error", e)))?;
            if !ipre.is_match(&domain) && !dre.is_match(&domain) {
                return Err(AppsError::System(fluent_message!(
                    "auth-alpha-domain-error"
                )));
            }
        }
        let name = name.trim().to_string();
        if name.len() < 3 || name.len() > 32 {
            return Err(AppsError::System(fluent_message!("app-name-wrong",
                {"len": name.len(),
                "min":2,
            "max":31}
                )));
        }
        let client_id = client_id.trim().to_string();
        if client_id.len() < 3 || client_id.len() > 32 {
            return Err(AppsError::System(fluent_message!("app-client-id-wrong",
                {"len": client_id.len(),
                "min":2,
            "max":31}
            )));
        }

        let re = Regex::new(r"^[a-z0-9]+$")
            .map_err(|e| AppsError::System(fluent_message!("auth-alpha-rule-error", e)))?;
        if !re.is_match(&client_id) {
            return Err(AppsError::System(fluent_message!("auth-alpha-check-error")));
        }
        Ok((name, client_id, domain))
    }
    /// 添加APP
    #[allow(clippy::too_many_arguments)]
    async fn add_app<'t>(
        &self,
        user_id: u64,
        name: String,
        client_id: String,
        domain: String,
        status: AppStatus,
        add_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AppsResult<u64> {
        let (name, client_id, domain) = self.check_app_param(name, client_id, domain)?;
        let app_res = Select::type_new::<AppsModel>()
            .fetch_one_by_where::<AppsModel, _>(
                &WhereOption::Where(sql_format!(" client_id={}", client_id)),
                &self.db,
            )
            .await;
        match app_res {
            Ok(app) => {
                if app.user_id == user_id {
                    return Ok(app.id);
                } else {
                    return Err(AppsError::System(fluent_message!("app-client-id-exits",
                        //"client id {$client_id} already used",
                        {"client_id":app.client_id,
                        "other_name":app.name
                    }
                    )));
                }
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }

        let time = now_time()?;
        let status = status as i8;

        let client_secret = range_client_key();
        let oauth_secret = range_client_key();
        let idata = model_option_set!(AppsModelRef,{
            name:name,
            client_id:client_id,
            oauth_secret:oauth_secret,
            client_secret:client_secret,
            status:status,
            user_id:user_id,
            change_user_id:add_user_id,
            change_time:time,
            callback_domain:domain
        });

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let res = Insert::<sqlx::MySql, AppsModel, _>::new(idata)
            .execute(&mut db)
            .await;
        match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                db.commit().await?;

                self.logger
                    .add(
                        &AppLog {
                            action: "add",
                            name,
                            status,
                            client_id,
                            client_secret,
                            callback_domain: domain,
                        },
                        &Some(mr.last_insert_id()),
                        &Some(user_id),
                        &Some(add_user_id),
                        None,
                        env_data,
                    )
                    .await;

                Ok(mr.last_insert_id())
            }
        }
    }
    /// 根据APP client_id 找到对应记录
    pub async fn find_by_client_id(&self, client_id: &String) -> AppsResult<AppsModel> {
        let useremal = Select::type_new::<AppsModel>()
            .fetch_one_by_where::<AppsModel, _>(
                &WhereOption::Where(sql_format!(" client_id={}", client_id)),
                &self.db,
            )
            .await?;
        Ok(useremal)
    }
    //内部APP secret 获取
    pub async fn find_secret_by_client_id(&self, client_id: &String) -> Result<String, AppsError> {
        let apps = self.cache().find_by_client_id(client_id).await;
        match apps {
            Ok(app) => {
                if !AppStatus::Ok.eq(app.status) {
                    return Err(
                        AppsError::System(fluent_message!("app-find-bad-status",{
                            "client_id":client_id
                        })), //,"your app id [{$client_id}] not confrim "
                    );
                }
                Ok(app.client_secret)
            }
            Err(err) => Err(err),
        }
    }
    pub fn cache(&'_ self) -> AppsCache<'_> {
        AppsCache { dao: self }
    }
}

pub struct AppsCache<'t> {
    pub dao: &'t Apps,
}
impl<'t> AppsCache<'t> {
    lsys_core::impl_cache_fetch_one!(find_by_client_id, dao, cache, String, AppsResult<AppsModel>);
}

#[test]
fn test_url() {
    let re = Regex::new(r"^[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}(:[\d]{1,5})?$").unwrap();
    assert!(re.is_match("127.0.0.1:80"));
    let re = Regex::new(
        r"^[0-9a-zA-Z]{0,1}[0-9a-zA-Z-]*(\.[0-9a-zA-Z-]*)*(\.[0-9a-zA-Z]*)+(:[\d]{1,5})?$",
    )
    .unwrap();
    assert!(re.is_match("aaa.com"));
}
