use std::sync::Arc;

use super::super::AppsResult;
use super::{AppSubAppLog, AppSubUserLog};
use crate::dao::AppsError;
use crate::model::{
    AppStatus, AppSubAppsModel, AppSubAppsModelRef, AppSubAppsStatus, AppSubUserModel,
    AppSubUserModelRef, AppSubUserStatus, AppsModel,
};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{fluent_message, now_time, PageParam, RemoteNotify, RequestEnv};
use lsys_logger::dao::ChangeLogger;
use sqlx::{prelude::FromRow, MySql, Pool};
use sqlx::{Row, ValueRef};
use sqlx_model::{model_option_set, sql_format, Insert, ModelTableName, Select, Update};
use sqlx_model::{SqlQuote, WhereOption};
pub struct SubApps {
    db: Pool<MySql>,
    pub(crate) cache: Arc<LocalCache<String, SubAppModel>>,
    logger: Arc<ChangeLogger>,
}

impl SubApps {
    pub fn new(
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        config:LocalCacheConfig,
        logger: Arc<ChangeLogger>,
    ) -> Self {
        Self {
            db,
            cache:Arc::new(LocalCache::new(remote_notify, config)),
            logger,
        }
    }
    pub fn cache(&'_ self) -> SubAppsCache<'_> {
        SubAppsCache { dao: self }
    }
    //列出指定app已添加子用户列表
    pub async fn list_sub_user_data(
        &self,
        app: &AppsModel,
        user_id: &Option<u64>,
        page: &Option<PageParam>,
    ) -> AppsResult<Vec<AppSubUserModel>> {
        let mut where_sql = sql_format!("app_id={}", app.id);
        if let Some(uid) = user_id {
            where_sql += sql_format!(" and user_id={}", uid).as_str();
        }
        let page_sql = if let Some(pdat) = page {
            format!(
                " order by id desc limit {} offset {} ",
                pdat.limit, pdat.offset
            )
        } else {
            " order by id desc".to_string()
        };
        let data = Select::type_new::<AppSubUserModel>()
            .fetch_all_by_where::<AppSubUserModel, _>(
                &WhereOption::Where(where_sql + page_sql.as_str()),
                &self.db,
            )
            .await?;
        Ok(data)
    }
    pub async fn list_sub_user_count(&self, app: &AppsModel) -> AppsResult<i64> {
        let sql = sql_format!(
            "select count(*) as  total from {} where app_id={}",
            AppSubUserModel::table_name(),
            app.id
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    } //更改指定app的指定用户可用状态
    pub async fn set_sub_user(
        &self,
        app: &AppsModel,
        user_id: &u64,
        used: &bool,
        env_data: Option<&RequestEnv>,
    ) -> AppsResult<()> {
        if *user_id == 0 {
            return Err(AppsError::System(fluent_message!("app-add-empty-user",
                {"app":&app.name}
            )));
        }
        match Select::type_new::<AppSubUserModel>()
            .fetch_one_by_where::<AppSubUserModel, _>(
                &WhereOption::Where(sql_format!("app_id={} and user_id={}", app.id, user_id)),
                &self.db,
            )
            .await
        {
            Ok(app_user) => {
                let db = self.db.begin().await?;

                let status = if *used {
                    AppSubUserStatus::Enable
                } else {
                    AppSubUserStatus::Disable
                } as i8;
                let time = now_time().unwrap_or_default();
                let change = sqlx_model::model_option_set!(AppSubUserModelRef,{
                    status:status,
                    change_time:time
                });
                let tmp = Update::<sqlx::MySql, AppSubUserModel, _>::new(change)
                    .execute_by_pk(&app_user, &self.db)
                    .await;
                if let Err(e) = tmp {
                    db.rollback().await?;
                    return Err(e.into());
                }
                let status = if *used {
                    AppSubAppsStatus::Enable
                } else {
                    AppSubAppsStatus::Disable
                } as i8;
                let change = sqlx_model::model_option_set!(AppSubAppsModelRef,{
                    status:status,
                    change_time:time
                });
                let tmp = Update::<sqlx::MySql, AppSubAppsModel, _>::new(change)
                    .execute_by_where(
                        &WhereOption::Where(sql_format!(
                            "app_id={} and user_id={} and status!={}",
                            app_user.app_id,
                            app_user.user_id,
                            AppSubAppsStatus::Delete
                        )),
                        &self.db,
                    )
                    .await;
                if let Err(e) = tmp {
                    db.rollback().await?;
                    return Err(e.into());
                }
                db.commit().await?;
            }
            Err(sqlx::Error::RowNotFound) => {
                let status = if *used {
                    AppSubUserStatus::Enable
                } else {
                    AppSubUserStatus::Disable
                } as i8;
                let time = now_time().unwrap_or_default();
                let idata = model_option_set!(AppSubUserModelRef,{
                    app_id:app.id,
                    user_id:user_id,
                    status:status,
                    change_time:time
                });
                Insert::<sqlx::MySql, AppSubUserModel, _>::new(idata)
                    .execute(&self.db)
                    .await?;
            }
            Err(err) => return Err(err.into()),
        };

        if !used {
            //清除查询缓存
            let sql = sql_format!(
                "select 
                    app.client_id
                    from {} as app_sub join {} as app
                    on app_sub.sub_app_id=app.id 
                    where app_sub.app_id={} and app_sub.user_id = {} and app_sub.status in ({})",
                AppSubAppsModel::table_name(),
                AppsModel::table_name(),
                app.id,
                user_id,
                &[
                    AppSubAppsStatus::Enable as i8,
                    AppSubAppsStatus::Disable as i8,
                ]
            );
            let query = sqlx::query_scalar::<_, String>(&sql);
            if let Ok(tmps) = query.fetch_all(&self.db).await {
                for tmp in tmps {
                    self.cache().clear_sub_secret_cache(app.id, &tmp).await;
                }
            }
        }

        self.logger
            .add(
                &AppSubUserLog {
                    used: *used,
                    sub_app_user_id: user_id.to_owned(),
                },
                &Some(app.id),
                &Some(app.user_id),
                &Some(app.user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
}
#[derive(FromRow)]
pub struct AppSubAppDataModel {
    pub sub_app: AppSubAppsModel,
    pub sub_app_name: String,
    pub sub_app_client_id: String,
}
impl SubApps {
    //列出指定app跟指定用户的已关联app及对应设置
    pub async fn list_sub_app_data(
        &self,
        app: &AppsModel,
        user_id: &Option<u64>,
        page: &Option<PageParam>,
    ) -> AppsResult<Vec<AppSubAppDataModel>> {
        let mut sql = sql_format!(
            "select 
                app.name as sub_app_name,
                app.client_id as sub_app_client_id,
                app_sub.*
                from {} as app_sub join {} as app
                on app_sub.sub_app_id=app.id 
                where app_sub.app_id={} and app_sub.status in ({})",
            AppSubAppsModel::table_name(),
            AppsModel::table_name(),
            app.id,
            &[
                AppSubAppsStatus::Disable as i8,
                AppSubAppsStatus::Enable as i8
            ],
        );
        if let Some(uid) = user_id {
            sql += sql_format!(" and app_sub.user_id={}", uid).as_str();
        }
        let sql = if let Some(pdat) = page {
            format!(
                "{} order by id desc limit {} offset {} ",
                sql, pdat.limit, pdat.offset
            )
        } else {
            format!("{} order by id desc ", sql)
        };
        Ok(sqlx::query(&sql)
            .try_map(
                |row: sqlx::mysql::MySqlRow| match AppSubAppsModel::from_row(&row) {
                    Ok(sub_app) => {
                        let sub_app_name = row
                            .try_get::<String, &str>("sub_app_name")
                            .unwrap_or_default();
                        let sub_app_client_id = row
                            .try_get::<String, &str>("sub_app_client_id")
                            .unwrap_or_default();
                        Ok(AppSubAppDataModel {
                            sub_app,

                            sub_app_name,
                            sub_app_client_id,
                        })
                    }
                    Err(err) => Err(err),
                },
            )
            .fetch_all(&self.db)
            .await?)
    }
    pub async fn list_sub_app_count(
        &self,
        app: &AppsModel,
        user_id: &Option<u64>,
    ) -> AppsResult<i64> {
        let mut sql = sql_format!(
            "select count(*)
                from {} as app_sub 
                where app_sub.app_id={} and app_sub.status in ({})",
            AppSubAppsModel::table_name(),
            app.id,
            &[
                AppSubAppsStatus::Disable as i8,
                AppSubAppsStatus::Enable as i8
            ],
        );
        if let Some(uid) = user_id {
            sql += sql_format!(" and app_sub.user_id={}", uid).as_str();
        }
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    } //更改指定a
      //进行上级应用关联
    pub async fn app_parent_set(
        &self,
        parent_app: &AppsModel,
        app: &AppsModel,
        sub_secret: &str,
        env_data: Option<&RequestEnv>,
    ) -> AppsResult<u64> {
        if parent_app.id == app.id {
            return Err(AppsError::System(fluent_message!("app-parent-add-self",
                {"name":&app.name}
            )));
        }
        if sub_secret.is_empty() {
            return Err(AppsError::System(
                fluent_message!("app-parent-secret-empty",
                {"name":&app.name}
                ),
            ));
        }
        let sql = sql_format!(
            "select id from {} where app_id={} and user_id={} and status={}",
            AppSubUserModel::table_name(),
            parent_app.id,
            app.user_id,
            AppSubUserStatus::Enable
        );
        if let Err(err) = sqlx::query_scalar::<_, u64>(&sql).fetch_one(&self.db).await {
            return Err(match err {
                sqlx::Error::RowNotFound => {
                    AppsError::System(fluent_message!("app-parent-add-bad-user",
                        {
                            "name":&app.name,
                            "parent_name":&parent_app.name,
                            "user_id":app.user_id
                        }
                         // "your can't add sub app to {} ,user id [{}] is not allowed ",
                           // parent_app.name,
                           // app.user_id
                    ))
                }
                err => err.into(),
            });
        }

        let res = match Select::type_new::<AppSubAppsModel>()
            .fetch_one_by_where::<AppSubAppsModel, _>(
                &WhereOption::Where(sql_format!(
                    "app_id={} and sub_app_id={}",
                    parent_app.id,
                    app.id
                )),
                &self.db,
            )
            .await
        {
            Ok(sapp) => {
                let time = now_time().unwrap_or_default();
                let change = sqlx_model::model_option_set!(AppSubAppsModelRef,{
                    status:AppSubAppsStatus::Enable as i8,
                    change_time:time
                });
                Update::<sqlx::MySql, AppSubAppsModel, _>::new(change)
                    .execute_by_pk(&sapp, &self.db)
                    .await?;
                return Ok(sapp.id);
            }
            Err(sqlx::Error::RowNotFound) => {
                let change_time = now_time()?;
                let sub_client_secret = sub_secret.to_owned();
                let status = AppSubAppsStatus::Enable.to();
                let idata = model_option_set!(AppSubAppsModelRef,{
                    app_id:parent_app.id,
                    user_id:app.user_id,
                    sub_app_id:app.id,
                    sub_client_secret:sub_client_secret,
                    status:status,
                    change_time:change_time
                });
                let res = Insert::<sqlx::MySql, AppSubAppsModel, _>::new(idata)
                    .execute(&self.db)
                    .await?;
                Ok(res.last_insert_id())
            }
            Err(err) => return Err(err.into()),
        };

        self.logger
            .add(
                &AppSubAppLog {
                    parent_app_id: parent_app.id,
                    sub_client_secret: Some(sub_secret.to_owned()),
                    status: AppSubAppsStatus::Enable.to(),
                },
                &Some(app.id),
                &Some(app.user_id),
                &Some(app.user_id),
                None,
                env_data,
            )
            .await;
        res
    }
    //删除上级应用关联
    pub async fn app_parent_del(
        &self,
        parent_app: &AppsModel,
        app: &AppsModel,
        env_data: Option<&RequestEnv>,
    ) -> AppsResult<()> {
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(AppSubAppsModelRef,{
            status:AppSubAppsStatus::Delete as i8,
            change_time:time
        });
        Update::<sqlx::MySql, AppSubAppsModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!(
                    "app_id={} and sub_app_id={}",
                    parent_app.id,
                    app.id
                )),
                &self.db,
            )
            .await?;

        self.cache()
            .clear_sub_secret_cache(parent_app.id, &app.client_id)
            .await;

        self.logger
            .add(
                &AppSubAppLog {
                    parent_app_id: parent_app.id,
                    sub_client_secret: None,
                    status: AppSubAppsStatus::Delete.to(),
                },
                &Some(app.id),
                &Some(app.user_id),
                &Some(app.user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
}
#[derive(FromRow)]
pub struct ParentAppModel {
    pub app_id: u64,
    pub app_name: String,
    pub app_client_id: String,
    pub user_id: u64,
    pub user_status: i8,
    pub sub_app: Option<ParentSubAppModel>,
}
#[derive(FromRow)]
pub struct ParentSubAppModel {
    pub sub_app_id: u64,
    pub sub_app_name: String,
    pub sub_app_client_id: String,
    pub sub_app_status: i8,
    pub sub_app_client_secret: String,
    pub change_time: u64,
}
impl SubApps {
    //列出当前用户可以可用的上级APP
    pub async fn parent_app_data(
        &self,
        app: &AppsModel,
        isset: &Option<bool>,
        page: &Option<PageParam>,
    ) -> AppsResult<Vec<ParentAppModel>> {
        let where_sql = match isset {
            Some(iss) => {
                if *iss {
                    sql_format!(
                        "select papp.id as papp_id,
                                papp.name as papp_name,
                                papp.client_id as papp_client_id,
                                papp.user_id as papp_user_id,
                                p_app_user.status as papp_user_status,
                                sapp.id as sapp_id,
                                sapp.name as sapp_name,
                                sapp.client_id as sapp_client_id,
                                sub_app.status as sub_app_status,
                                sub_app.sub_client_secret as sub_app_secret,
                                sub_app.change_time as change_time
                                from {} as p_app_user 
                                join {} as papp on p_app_user.app_id=papp.id 
                                join {} as sub_app on p_app_user.app_id=sub_app.app_id 
                                    and sub_app.status in ({})
                                    and sub_app.user_id={}
                                    and sub_app.sub_app_id={}
                                join {} as sapp on sub_app.sub_app_id=sapp.id
                                where p_app_user.user_id={}  and p_app_user.status = {} and  p_app_user.app_id!={}",
                        AppSubUserModel::table_name(),
                        AppsModel::table_name(),
                        AppSubAppsModel::table_name(),
                        &[
                            AppSubAppsStatus::Enable as i8,
                            AppSubAppsStatus::Disable as i8,
                        ],
                        app.user_id,
                        app.id,
                        AppsModel::table_name(),
                        app.user_id,
                        AppSubUserStatus::Enable as i8,
                        app.id,
                    )
                } else {
                    sql_format!(
                        "select papp.id as papp_id,
                                papp.name as papp_name,
                                papp.client_id as papp_client_id,
                                papp.user_id as papp_user_id,
                                p_app_user.status as papp_user_status,
                                NULL as sapp_id,
                                NULL as sapp_name,
                                NULL as sapp_client_id,
                                NULL as sub_app_status,
                                NULL as sub_app_secret,
                                NULL as change_time
                                from {} as p_app_user 
                                join {} as papp on p_app_user.app_id=papp.id 
                                where p_app_user.user_id={} 
                                and p_app_user.status = {}
                                and  p_app_user.app_id!={}
                                and p_app_user.app_id not in (
                                    select app_id from {} where sub_app_id={} and user_id={} and status in ({})
                                )",
                        AppSubUserModel::table_name(),
                        AppsModel::table_name(),
                        app.user_id,
                        AppSubUserStatus::Enable as i8,
                        app.id,
                        AppSubAppsModel::table_name(),
                        app.id,
                        app.user_id,
                        &[
                            AppSubAppsStatus::Enable as i8,
                            AppSubAppsStatus::Disable as i8,
                        ],
                    )
                }
            }
            None => {
                sql_format!(
                    "select papp.id as papp_id,
                            papp.name as papp_name,
                            papp.client_id as papp_client_id,
                            papp.user_id as papp_user_id,
                            p_app_user.status as papp_user_status,
                            sapp.id as sapp_id,
                            sapp.name as sapp_name,
                            sapp.client_id as sapp_client_id,
                            sub_app.status as sub_app_status,
                            sub_app.sub_client_secret as sub_app_secret,
                            sub_app.change_time as change_time
                            from {} as p_app_user 
                            join {} as papp on p_app_user.app_id=papp.id 
                            left join {} as sub_app on p_app_user.app_id=sub_app.app_id 
                                and sub_app.status in ({})
                                and sub_app.user_id={}
                                and sub_app.sub_app_id={}
                            left join {} as sapp on sub_app.sub_app_id=sapp.id
                            where p_app_user.user_id={} and p_app_user.status = {}  and   p_app_user.app_id!={}",
                    AppSubUserModel::table_name(),
                    AppsModel::table_name(),
                    AppSubAppsModel::table_name(),
                    &[
                        AppSubAppsStatus::Enable as i8,
                        AppSubAppsStatus::Disable as i8,
                    ],
                    app.user_id,
                    app.id,
                    AppsModel::table_name(),
                    app.user_id,
                    AppSubUserStatus::Enable as i8,
                    app.id,
                )
            }
        };
        let sql = if let Some(pdat) = page {
            format!(
                "{} order by papp_id desc limit {} offset {} ",
                where_sql, pdat.limit, pdat.offset
            )
        } else {
            format!("{} order by papp_id desc ", where_sql)
        };
        Ok(sqlx::query(&sql)
            .try_map(|row: sqlx::mysql::MySqlRow| {
                let app_id = row.try_get::<u64, &str>("papp_id").unwrap_or_default();
                let app_name = row.try_get::<String, &str>("papp_name").unwrap_or_default();
                let app_client_id = row
                    .try_get::<String, &str>("papp_client_id")
                    .unwrap_or_default();
                let user_id = row.try_get::<u64, &str>("papp_user_id").unwrap_or_default();
                let user_status = row
                    .try_get::<i8, &str>("papp_user_status")
                    .unwrap_or_default();
                let mut sub_app = None;
                if !row.try_get_raw("sapp_id")?.is_null() {
                    let sub_app_id = row.try_get::<u64, &str>("sapp_id").unwrap_or_default();
                    let sub_app_name = row.try_get::<String, &str>("sapp_name").unwrap_or_default();
                    let sub_app_client_id = row
                        .try_get::<String, &str>("sapp_client_id")
                        .unwrap_or_default();
                    let sub_app_status = row
                        .try_get::<i8, &str>("sub_app_status")
                        .unwrap_or_default();
                    let sub_app_client_secret = row
                        .try_get::<String, &str>("sub_app_secret")
                        .unwrap_or_default();
                    let change_time = row.try_get::<u64, &str>("change_time").unwrap_or_default();
                    sub_app = Some(ParentSubAppModel {
                        sub_app_id,
                        sub_app_name,
                        sub_app_client_id,
                        sub_app_status,
                        sub_app_client_secret,
                        change_time,
                    });
                }
                Ok(ParentAppModel {
                    app_id,
                    app_name,
                    app_client_id,
                    user_id,
                    user_status,
                    sub_app,
                })
            })
            .fetch_all(&self.db)
            .await?)
    }
    //列出当前用户可以可用的上级APP
    pub async fn parent_app_count(&self, app: &AppsModel, isset: &Option<bool>) -> AppsResult<i64> {
        let sql = match isset {
            Some(iss) => {
                if *iss {
                    sql_format!(
                        "select count(*) as total
                                from {} as p_app_user 
                                where p_app_user.user_id={}  and p_app_user.status = {}
                                and  p_app_user.app_id!={}
                                and p_app_user.app_id in (
                                    select app_id from {} where sub_app_id={} and user_id={} and status in ({})
                                ) ",
                        AppSubUserModel::table_name(),
                        app.user_id,
                        &[
                            AppSubAppsStatus::Enable as i8,
                            AppSubAppsStatus::Disable as i8,
                        ],
                        app.id,
                        AppSubAppsModel::table_name(),
                        app.id,
                        app.user_id,
                        AppSubUserStatus::Enable as i8,
                    )
                } else {
                    sql_format!(
                        "select count(*) as total
                                from {} as p_app_user 
                                where p_app_user.user_id={} 
                                and p_app_user.status = {}
                                and  p_app_user.app_id!={}
                                and p_app_user.app_id not in (
                                    select app_id from {} where sub_app_id={} and user_id={} and status in ({})
                                )",
                        AppSubUserModel::table_name(),
                        app.user_id,
                        AppSubUserStatus::Enable as i8,
                        app.id,
                        AppSubAppsModel::table_name(),
                        app.id,
                        app.user_id,
                        &[
                            AppSubAppsStatus::Enable as i8,
                            AppSubAppsStatus::Disable as i8,
                        ],
                    )
                }
            }
            None => {
                sql_format!(
                    "select count(*) as total
                            from {} as p_app_user 
                            where p_app_user.user_id={} and p_app_user.status = {}  and  p_app_user.app_id!={}",
                    AppSubUserModel::table_name(),
                    app.user_id,
                    AppSubUserStatus::Enable as i8,
                    app.id,
                )
            }
        };
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
}

#[derive(FromRow, Clone)]
pub struct SubAppModel {
    pub app_id: u64,
    pub user_id: u64,
    pub app_name: String,
    pub app_client_id: String,
    pub sub_client_secret: String,
}
impl SubApps {
    //内部APP secret 获取
    pub async fn find_sub_secret_by_client_id(
        &self,
        app_id: &u64,
        client_id: &str,
    ) -> AppsResult<SubAppModel> {
        let sql = sql_format!(
            "select sub_app.app_id,sub_app.user_id,
                app.name as app_name,
                app.client_id as app_client_id,
                sub_app.sub_client_secret
                from {} as sub_app join {} as app
                on sub_app.sub_app_id=app.id 
                where sub_app.app_id={} and app.client_id={}
                and app.status={} and sub_app.status={}",
            AppSubAppsModel::table_name(),
            AppsModel::table_name(),
            app_id,
            client_id,
            AppStatus::Ok,
            AppSubAppsStatus::Enable,
        );
        let res = sqlx::query_as::<_, SubAppModel>(sql.as_str());
        Ok(res.fetch_one(&self.db).await?)
    }
}

pub struct SubAppsCache<'t> {
    pub dao: &'t SubApps,
}
impl<'t> SubAppsCache<'t> {
    pub(crate) async fn clear_sub_secret_cache(&self, app_id: u64, client_id: &str) {
        let cache_key = format!("sub-app-{}{}", app_id, client_id);
        self.dao.cache.del(&cache_key).await;
    }
    //内部APP secret 获取
    pub async fn find_sub_secret_by_client_id(
        &self,
        app_id: &u64,
        client_id: &str,
    ) -> AppsResult<SubAppModel> {
        let cache_key = format!("sub-app-{}{}", app_id, client_id);
        let cache_data = self.dao.cache.get(&cache_key).await;
        match cache_data {
            Some(data) => Ok(data),
            None => match self
                .dao
                .find_sub_secret_by_client_id(app_id, client_id)
                .await
            {
                Ok(data) => {
                    self.dao.cache.set(cache_key, data.clone(), 0).await;
                    Ok(data)
                }
                Err(e) => Err(e),
            },
        }
    }
}
