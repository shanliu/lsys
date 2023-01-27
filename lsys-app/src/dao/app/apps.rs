use std::sync::Arc;

use crate::model::{AppStatus, AppsModel, AppsModelRef};
use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    get_message, now_time, FluentMessage, PageParam,
};

use redis::aio::ConnectionManager;
use regex::Regex;
use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{model_option_set, sql_format, Insert, ModelTableName, Select, Update};
use sqlx_model::{SqlExpr, SqlQuote};
use tokio::sync::Mutex;

use super::{range_client_key, AppsError, AppsResult};

pub struct Apps {
    db: Pool<MySql>,
    pub(crate) fluent: Arc<FluentMessage>,
    pub cache: Arc<LocalCache<String, AppsModel>>,
}

#[derive(Clone, Debug)]
pub struct AppDataWhere<'t> {
    pub user_id: Option<u64>,
    pub status: &'t Option<Vec<AppStatus>>,
    pub client_ids: &'t Option<Vec<String>>,
    pub app_ids: &'t Option<Vec<u64>>,
}

impl Apps {
    pub fn new(
        db: Pool<MySql>,
        redis: Arc<Mutex<ConnectionManager>>,
        fluent: Arc<FluentMessage>,
    ) -> Self {
        Self {
            db,
            fluent,
            cache: Arc::from(LocalCache::new(redis, LocalCacheConfig::new("apps"))),
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
    pub async fn app_data<'t>(
        &self,
        app_where: &AppDataWhere<'t>,
        page: &Option<PageParam>,
    ) -> AppsResult<Vec<AppsModel>> {
        let mut sql = match self.app_data_sql(app_where) {
            Some(sql) => sql,
            None => return Ok(vec![]),
        };
        if sql.is_empty() {
            sql = "1=1".to_string();
        }
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }
        let data = Select::type_new::<AppsModel>()
            .fetch_all_by_where::<AppsModel, _>(Some(sql), &self.db)
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
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> AppsResult<String> {
        let client_secret = range_client_key();
        let change = sqlx_model::model_option_set!(AppsModelRef,{
            client_secret:client_secret,
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
        Ok(client_secret)
    }
    //确认APP
    pub async fn confirm_app<'t>(
        &self,
        app: &AppsModel,
        user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
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
        Ok(res.rows_affected())
    }
    //添加内部APP
    #[allow(clippy::too_many_arguments)]
    pub async fn innernal_app_edit<'t>(
        &self,
        app: &AppsModel,
        name: String,
        client_id: String,
        domain: String,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> AppsResult<u64> {
        let (name, client_id, domain) = self.check_app_param(name, client_id, domain)?;
        let app_res = Select::type_new::<AppsModel>()
            .fetch_one_by_where_call::<AppsModel, _, _>(
                "id!=? and client_id=?".to_string(),
                |mut res, _| {
                    res = res.bind(app.id);
                    res = res.bind(client_id.clone());
                    res
                },
                &self.db,
            )
            .await;
        match app_res {
            Ok(app) => {
                return Err(AppsError::System(get_message!(&self.fluent,
                    "app-client-id-exits","client id {$client_id} already used",
                    ["client_id"=>app.client_id]
                )));
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }
        let change = sqlx_model::model_option_set!(AppsModelRef,{
            name:name,
            client_id:client_id,
            callback_domain:domain,
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
    ) -> AppsResult<u64> {
        self.add_app(
            uesr_id,
            name,
            client_id,
            domain,
            status,
            add_user_id,
            transaction,
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
            let re = Regex::new(r"^http(s)?://[0-9a-z]+\.[0-9a-z]+").map_err(|e| {
                AppsError::System(get_message!(
                    &self.fluent,
                    "auth-alpha-domain-error",
                    e.to_string()
                ))
            })?;
            if !re.is_match(&domain) {
                return Err(AppsError::System(get_message!(
                    &self.fluent,
                    "auth-alpha-domain-error",
                    "submit callback url is wrong"
                )));
            }
        }
        let name = name.trim().to_string();
        if name.len() < 3 || name.len() > 32 {
            return Err(AppsError::System(get_message!(
                &self.fluent,
                "app-name-wrong",
                "name length need 3-32 char"
            )));
        }
        let client_id = client_id.trim().to_string();
        if client_id.len() < 3 || client_id.len() > 32 {
            return Err(AppsError::System(get_message!(
                &self.fluent,
                "app-client-id-wrong",
                "client id length need 3-32 char"
            )));
        }

        let re = Regex::new(r"^[a-z0-9]+$").map_err(|e| {
            AppsError::System(get_message!(
                &self.fluent,
                "auth-alpha-num-error",
                e.to_string()
            ))
        })?;
        if !re.is_match(&client_id) {
            return Err(AppsError::System(get_message!(
                &self.fluent,
                "auth-alpha-num-error",
                "submit client not a alpha or num char"
            )));
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
    ) -> AppsResult<u64> {
        let (name, client_id, domain) = self.check_app_param(name, client_id, domain)?;
        let app_res = Select::type_new::<AppsModel>()
            .fetch_one_by_where_call::<AppsModel, _, _>(
                " client_id=?".to_string(),
                |mut res, _| {
                    res = res.bind(client_id.clone());
                    res
                },
                &self.db,
            )
            .await;
        match app_res {
            Ok(app) => {
                if app.user_id == user_id {
                    return Ok(app.id);
                } else {
                    return Err(AppsError::System(get_message!(&self.fluent,
                        "app-client-id-exits","client id {$client_id} already used",
                        ["client_id"=>app.client_id]
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

        let idata = model_option_set!(AppsModelRef,{
            name:name,
            client_id:client_id,
            client_secret:client_secret,
            status:status,
            user_id:user_id,
            add_user_id:add_user_id,
            add_time:time,
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
                Ok(mr.last_insert_id())
            }
        }
    }
    /// 根据APP id 找到对应记录
    pub async fn find_by_client_id(&self, client_id: &String) -> AppsResult<AppsModel> {
        let useremal = Select::type_new::<AppsModel>()
            .fetch_one_by_where_call::<AppsModel, _, _>(
                "client_id=? ".to_string(),
                |mut res, _| {
                    res = res.bind(client_id.to_owned());
                    res
                },
                &self.db,
            )
            .await?;
        Ok(useremal)
    }
    //内部APP secret 获取
    pub async fn innernal_client_id_get(&self, client_id: &String) -> Result<String, String> {
        let apps = self.cache().find_by_client_id(client_id).await;
        match apps {
            Ok(app) => {
                if !AppStatus::Ok.eq(app.status) {
                    return Err(
                        get_message!(&self.fluent,"app-status","your app id [{$client_id}] not confrim ",[
                            "client_id"=>client_id.clone()
                        ]),
                    );
                }
                Ok(app.client_secret)
            }
            Err(err) => Err(err.to_string()),
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
    let re = Regex::new(r"^http(s)?://[0-9a-z]+\.[0-9a-z]+").unwrap();
    assert!(re.is_match("https://local.host:8080/user/app"));
}
