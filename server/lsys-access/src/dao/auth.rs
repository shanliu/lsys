use std::fmt::Display;

use std::str::FromStr;
use std::sync::Arc;

use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{fluent_message, now_time, rand_str, RandType, RemoteNotify};

use crate::dao::AccessUser;
use lsys_core::db::Insert;
use lsys_core::db::{ModelTableName, SqlQuote, Update, WhereOption};
use lsys_core::{model_option_set, sql_format};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};

use crate::model::{
    SessionDataModel, SessionDataModelRef, SessionModel, SessionModelRef, SessionStatus, UserModel,
};

use super::{AccessError, AccessResult, SessionBody};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccessAuthSessionCacheKey {
    app_id: u64,
    oauth_app_id: u64,
    token_data: String,
}
impl Display for AccessAuthSessionCacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::json!(self))
    }
}
impl FromStr for AccessAuthSessionCacheKey {
    type Err = AccessError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str::<AccessAuthSessionCacheKey>(s)?)
    }
}

pub struct AccessAuth {
    db: Pool<MySql>,
    user: Arc<AccessUser>,
    pub(crate) session_cache: Arc<LocalCache<AccessAuthSessionCacheKey, SessionModel>>,
    pub(crate) session_data_cache:
        Arc<LocalCache<AccessAuthSessionCacheKey, Vec<(String, String)>>>,
}

impl AccessAuth {
    pub fn new(
        db: Pool<MySql>,
        user: Arc<AccessUser>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
    ) -> Self {
        Self {
            // user_cache: Arc::new(LocalCache::new(remote_notify.clone(), config)),
            session_cache: Arc::new(LocalCache::new(remote_notify.clone(), config)),
            session_data_cache: Arc::new(LocalCache::new(remote_notify, config)),
            db,
            user,
        }
    }
    //通过ID获取用户
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_user_by_id,
        u64,
        UserModel,
        AccessResult<UserModel>,
        id,
        "id = {id} "
    );
    fn wrap_session_body(
        &self,
        session: SessionModel,
        user: UserModel,
    ) -> AccessResult<SessionBody> {
        let session_body = SessionBody::new(user, session);
        session_body.valid()?;
        Ok(session_body)
    }
    async fn load_session_body(&self, session: SessionModel) -> AccessResult<SessionBody> {
        let user = self.find_user_by_id(&session.user_id).await?;
        self.wrap_session_body(session, user)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct AccessLoginData<'t> {
    pub user_account: Option<&'t str>,
    pub login_ip: Option<&'t str>,
    pub device_id: Option<&'t str>,
    pub device_name: Option<&'t str>,
    pub expire_time: u64,
    pub session_data: Vec<(&'t str, &'t str)>,
}
impl AccessAuth {
    //強制指定應用全部下線
    pub async fn clear_app_login(&self, user_app_id: &u64) -> AccessResult<()> {
        let time = now_time()?;
        let status = SessionStatus::Delete.to();
        let change = lsys_core::model_option_set!(SessionModelRef,{
            status:status,
            logout_time:time,
        });
        Update::<SessionModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("user_app_id={} ", user_app_id)),
                &self.db,
            )
            .await?;
        let mut start_id = 0;
        loop {
            let sql = sql_format!(
                "select 
                    id,
                    oauth_app_id,
                    token_data
                from {} 
                where logout_time ={} 
                and status={}
                and user_app_id={}
                and id>{}
                order by id asc
                limit 100",
                SessionModel::table_name(),
                time,
                status,
                user_app_id,
                start_id
            );
            let res = sqlx::query_as::<_, (u64, u64, String)>(sql.as_str())
                .fetch_all(&self.db)
                .await?;
            if res.is_empty() {
                break;
            }
            for (next_id, oauth_app_id, token_data) in res {
                self.cache()
                    .del_session(user_app_id, &oauth_app_id, &token_data)
                    .await?;
                start_id = next_id
            }
        }
        Ok(())
    }
}

pub struct AccessAuthLoginData<'t, TS: ToString> {
    pub app_id: u64,
    pub oauth_app_id: u64,
    pub user_data: TS,
    pub user_name: &'t str,
    pub token_data: Option<&'t str>,
    pub login_type: &'t str,
    pub login_data: Option<&'t AccessLoginData<'t>>,
}

impl AccessAuth {
    //登录
    pub async fn do_login<TS: ToString>(
        &self,
        login_param: &AccessAuthLoginData<'_, TS>,
    ) -> AccessResult<SessionBody> {
        let time = now_time()?;
        let user_data = login_param.user_data.to_string();
        let token_data = login_param
            .token_data
            .map(|e| e.to_owned())
            .unwrap_or_else(|| rand_str(RandType::Upper, 32));
        if token_data.len() < 16 {
            return Err(AccessError::System(
                fluent_message!("access-bad-token-data",{
                    "data":token_data
                }),
            ));
        }

        let user_account = login_param
            .login_data
            .as_ref()
            .map(|e| e.user_account.to_owned().unwrap_or_default())
            .unwrap_or_default();

        let user_id = self
            .user
            .sync_user(
                login_param.app_id,
                user_data.as_str(),
                Some(login_param.user_name),
                Some(user_account),
            )
            .await?;

        let device_id = login_param
            .login_data
            .as_ref()
            .map(|e| e.device_id.unwrap_or_default().to_string())
            .unwrap_or_default();

        let expire_time = login_param
            .login_data
            .as_ref()
            .map(|e| e.expire_time)
            .unwrap_or_default();
        let login_ip = login_param
            .login_data
            .as_ref()
            .map(|e| e.login_ip.unwrap_or_default().to_string())
            .unwrap_or_default();
        let device_name = login_param
            .login_data
            .as_ref()
            .map(|e| e.device_name.unwrap_or_default().to_string())
            .unwrap_or_default();
        let login_type = login_param.login_type.to_owned();
        let mut db = self.db.begin().await?;

        let status = SessionStatus::Enable as i8;

        let vdata = lsys_core::model_option_set!(SessionModelRef,{
            user_id:user_id,
            user_app_id:login_param.app_id,
            oauth_app_id:login_param.oauth_app_id,
            token_data:token_data,
            login_type:login_type,
            login_ip:login_ip,
            device_id:device_id,
            device_name:device_name,
            status:status,
            add_time:time,
            expire_time:expire_time,
            logout_time:0,
        });

        let sid = match Insert::<SessionModel, _>::new(vdata)
            .execute(&mut *db)
            .await
        {
            Ok(id) => id.last_insert_id(),
            Err(err) => {
                db.rollback().await?;
                return Err(err.into());
            }
        };
        let session_data = login_param
            .login_data
            .as_ref()
            .map(|e| e.session_data.to_owned())
            .unwrap_or_default();
        if !session_data.is_empty() {
            let tmps = session_data
                .iter()
                .map(|e| (e.0.to_owned(), e.1.to_string()))
                .collect::<Vec<_>>();
            let mut session_data = Vec::with_capacity(tmps.len());
            for t in tmps.iter() {
                session_data.push(model_option_set!(SessionDataModelRef,{
                    session_id:sid,
                    data_key:t.0,
                    data_val:t.1,
                    change_time:time,
                }));
            }

            if let Err(err) = Insert::<SessionDataModel, _>::new_vec(session_data)
                .execute(&mut *db)
                .await
            {
                db.rollback().await?;
                return Err(err.into());
            }
        }

        db.commit().await?;

        // self.user_cache.del(&user_id).await;
        self.cache()
            .login_data(login_param.app_id, login_param.oauth_app_id, &token_data)
            .await
    }

    //延长登录时间
    pub async fn extend_login(
        &self,
        session_body: &SessionBody,
        add_time: u64,
    ) -> AccessResult<SessionBody> {
        session_body.valid()?;
        let mut session = session_body.session().to_owned();
        if add_time == 0 || session.expire_time == 0 {
            return Ok(SessionBody::new(session_body.user().to_owned(), session));
        }
        let expire_time = add_time + session.expire_time;
        let change = lsys_core::model_option_set!(SessionModelRef,{
            expire_time:expire_time,
        });
        Update::<SessionModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("id={} ", session.id)),
                &self.db,
            )
            .await?;
        self.cache()
            .del_session(
                &session.user_app_id,
                &session.oauth_app_id,
                &session.token_data,
            )
            .await?;
        session.expire_time = expire_time;
        self.load_session_body(session).await
    }
    //重新登陆
    pub async fn refresh_login(
        &self,
        session_body: &SessionBody,
        expire_time: Option<u64>,
        token_data: Option<&str>,
    ) -> AccessResult<SessionBody> {
        let expire_time = expire_time.unwrap_or(0);
        let token_data = token_data
            .map(|e| e.to_owned())
            .unwrap_or_else(|| rand_str(RandType::Upper, 32));
        let time = now_time()?;
        if SessionStatus::Refresh.eq(session_body.session().status)
            && session_body.session().expire_time > time
        {
            match sqlx::query_as::<_,SessionModel>(&sql_format!(
                    "select * from {} where user_app_id={} and source_token_data={} and status={} and oauth_app_id={} ",
                    SessionModel::table_name(),
                    session_body.session().user_app_id,
                    session_body.session().token_data,
                    SessionStatus::Enable as i8,
                    session_body.session().oauth_app_id,
                )).fetch_one(&self.db).await
            {
                Ok(e) => return self.load_session_body(e).await,
                Err(e) => match e {
                    sqlx::Error::RowNotFound => {}
                    _ => return Err(AccessError::Sqlx(e)),
                },
            };
        }
        session_body.valid()?;
        let status = SessionStatus::Refresh.to();
        let change = lsys_core::model_option_set!(SessionModelRef,{
            status:status,
            logout_time:time,
        });
        let mut db = self.db.begin().await?;

        if let Err(err) = Update::<SessionModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("id={} ", session_body.session().id)),
                &mut *db,
            )
            .await
        {
            db.rollback().await?;
            return Err(err.into());
        };
        let add_status = SessionStatus::Enable.to();

        let vdata = lsys_core::model_option_set!(SessionModelRef,{
            user_id: session_body.session().user_id,
            user_app_id: session_body.session().user_app_id,
            oauth_app_id: session_body.session().user_app_id,
            token_data:token_data,
            source_token_data: session_body.session().token_data,
            login_type: session_body.session().login_type,
            login_ip: session_body.session().login_ip,
            device_id: session_body.session().device_id,
            device_name: session_body.session().device_name,
            status:add_status,
            expire_time:expire_time,
            add_time:time,
            logout_time:0,
        });
        let sid = match Insert::<SessionModel, _>::new(vdata)
            .execute(&mut *db)
            .await
        {
            Ok(id) => id.last_insert_id(),
            Err(err) => {
                db.rollback().await?;
                return Err(err.into());
            }
        };

        let change = lsys_core::model_option_set!(SessionDataModelRef,{
            session_id:sid
        });
        let mut db = self.db.begin().await?;
        if let Err(err) = Update::<SessionDataModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("session_id={} ", session_body.session().id)),
                &mut *db,
            )
            .await
        {
            db.rollback().await?;
            return Err(err.into());
        };
        db.rollback().await?;

        self.cache()
            .del_session(
                &session_body.session().user_app_id,
                &session_body.session().oauth_app_id,
                &session_body.session().token_data,
            )
            .await?;
        self.cache()
            .login_data(
                session_body.session().user_app_id,
                session_body.session().oauth_app_id,
                &token_data,
            )
            .await
    }
    //退出登录
    pub async fn do_logout(&self, session_body: &SessionBody) -> AccessResult<()> {
        let time = now_time()?;
        let status = SessionStatus::Delete.to();
        let change = lsys_core::model_option_set!(SessionModelRef,{
            status:status,
            logout_time:time,
        });
        Update::<SessionModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("id={} ", session_body.session().id)),
                &self.db,
            )
            .await?;
        self.cache()
            .del_session(
                &session_body.session().user_app_id,
                &session_body.session().oauth_app_id,
                &session_body.session().token_data,
            )
            .await?;
        Ok(())
    }
    //获取登录数据
    pub async fn login_data(
        &self,
        app_id: u64,
        oauth_app_id: u64,
        token_data: &str,
    ) -> AccessResult<SessionBody> {
        self.load_session_body(self.load_session(app_id, oauth_app_id, token_data).await?)
            .await
    }
    async fn load_session(
        &self,
        app_id: u64,
        oauth_app_id: u64,
        token_data: &str,
    ) -> AccessResult<SessionModel> {
        if token_data.is_empty() {
            return Err(AccessError::NotLogin);
        }

        let data =  sqlx::query_as::<_,SessionModel>(&sql_format!(
                "select * from {} where user_app_id={} and token_data={} and status={}  and oauth_app_id={}",
                SessionModel::table_name(),
                app_id,
                token_data,
                SessionStatus::Enable as i8,
                oauth_app_id,
            )).fetch_one(&self.db).await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AccessError::NotLogin,
                _ => AccessError::Sqlx(e),
            })?;
        Ok(data)
    }
    //登陆附带数据获取
    pub async fn session_get_data(
        &self,
        session_body: &SessionBody,
        data_key: &str,
    ) -> AccessResult<Option<String>> {
        Ok(self
            .session_get_vec_data(session_body, &[data_key])
            .await?
            .into_iter()
            .find(|e| e.0.as_str() == data_key)
            .map(|e| e.1))
    }
    //登陆附带数据批量获取
    pub async fn session_get_vec_data(
        &self,
        session_body: &SessionBody,
        data_key: &[&str],
    ) -> AccessResult<Vec<(String, String)>> {
        session_body.valid()?;
        if data_key.is_empty() {
            return Ok(vec![]);
        }
        let data = sqlx::query_as::<_, SessionDataModel>(&sql_format!(
            "select * from {} where session_id={} and data_key in ({})",
            SessionDataModel::table_name(),
            session_body.session().id,
            data_key
        ))
        .fetch_all(&self.db)
        .await?;
        Ok(data
            .into_iter()
            .map(|e| (e.data_key, e.data_val))
            .collect::<Vec<_>>())
    }
    //登陆附带数据设置
    pub async fn session_set_data(
        &self,
        session_body: &SessionBody,
        data_key: &str,
        data_val: &str,
    ) -> AccessResult<()> {
        self.session_set_vec_data(session_body, &[(data_key, data_val)])
            .await
    }
    //登陆附带数据批量设置
    pub async fn session_set_vec_data(
        &self,
        session_body: &SessionBody,
        data: &[(&str, &str)],
    ) -> AccessResult<()> {
        session_body.valid()?;

        let time = now_time()?;

        let mut db = self.db.begin().await?;

        for (data_key, data_val) in data {
            let data_key = data_key.to_string();
            let data_val = data_val.to_string();
            let vdata = lsys_core::model_option_set!(SessionDataModelRef,{
                session_id:session_body.session().id,
                data_key:data_key,
                data_val:data_val,
                change_time:time,
            });
            let change = lsys_core::model_option_set!(SessionDataModelRef,{
                data_val:data_val,
                change_time:time,
            });
            if let Err(err) = Insert::<SessionDataModel, _>::new(vdata)
                .execute_update(&Update::<SessionDataModel, _>::new(change), &mut *db)
                .await
            {
                db.rollback().await?;
                return Err(err.into());
            }
        }
        db.commit().await?;
        self.cache()
            .del_session_data(
                session_body.session(),
                data.iter().map(|e| e.0).collect::<Vec<_>>().as_slice(),
            )
            .await?;
        Ok(())
    }
    pub fn cache(&'_ self) -> AccessAuthCache<'_> {
        AccessAuthCache { dao: self }
    }
}

pub struct AccessAuthCache<'t> {
    pub dao: &'t AccessAuth,
}
impl AccessAuthCache<'_> {
    //获取登陆附带数据
    pub async fn session_get_data(
        &self,
        session_body: &SessionBody,
        data_key: &str,
    ) -> AccessResult<Option<String>> {
        Ok(self
            .session_get_vec_data(session_body, &[data_key])
            .await?
            .into_iter()
            .find(|e| e.0.as_str() == data_key)
            .map(|e| e.1))
    }
    //批量获取登陆附带数据
    pub async fn session_get_vec_data(
        &self,
        session_body: &SessionBody,
        data_key: &[&str],
    ) -> AccessResult<Vec<(String, String)>> {
        session_body.valid()?;
        if data_key.is_empty() {
            return Ok(vec![]);
        }
        let cache_key = AccessAuthSessionCacheKey {
            app_id: session_body.session().user_app_id,
            oauth_app_id: session_body.session().oauth_app_id,
            token_data: session_body.session().token_data.to_owned(),
        };
        let dvs = self.dao.session_data_cache.get(&cache_key).await;
        let mut out = Vec::with_capacity(data_key.len());
        let find_keys = match &dvs {
            Some(dat) => {
                let mut find = vec![];
                for dk in data_key {
                    match dat.iter().find(|e| e.0.as_str() == *dk) {
                        Some(tmp) => out.push(tmp.to_owned()),
                        None => {
                            find.push(*dk);
                        }
                    }
                }
                find
            }
            None => data_key.to_vec(),
        };
        if !find_keys.is_empty() {
            let data = self
                .dao
                .session_get_vec_data(session_body, &find_keys)
                .await?;
            let mut set_val = dvs.unwrap_or_default();
            set_val.extend(data);
            self.dao.session_data_cache.set(cache_key, set_val, 0).await;
        }
        Ok(out)
    }
    //获取登陆数据
    pub async fn login_data(
        &self,
        app_id: u64,
        oauth_app_id: u64,
        token_data: &str,
    ) -> AccessResult<SessionBody> {
        if token_data.is_empty() {
            return Err(AccessError::NotLogin);
        }
        let cache_key = AccessAuthSessionCacheKey {
            app_id,
            oauth_app_id,
            token_data: token_data.to_owned(),
        };
        let data = self.dao.session_cache.get(&cache_key).await;
        let session_model = match data {
            Some(session_model) => session_model,
            None => {
                let session_model = self
                    .dao
                    .load_session(app_id, oauth_app_id, token_data)
                    .await?;
                self.dao
                    .session_cache
                    .set(cache_key, session_model.clone(), 0)
                    .await;
                session_model
            }
        };
        self.dao.load_session_body(session_model).await
    }
    async fn del_session(
        &self,
        app_id: &u64,
        oauth_app_id: &u64,
        token_data: &str,
    ) -> AccessResult<()> {
        let cache_key = AccessAuthSessionCacheKey {
            app_id: *app_id,
            oauth_app_id: *oauth_app_id,
            token_data: token_data.to_owned(),
        };
        self.dao.session_cache.clear(&cache_key).await;
        self.dao.session_data_cache.clear(&cache_key).await;
        Ok(())
    }
    async fn del_session_data(
        &self,
        session: &SessionModel,
        data_key: &[&str],
    ) -> AccessResult<()> {
        if data_key.is_empty() {
            return Ok(());
        }
        let cache_key = AccessAuthSessionCacheKey {
            app_id: session.user_app_id,
            oauth_app_id: session.oauth_app_id,
            token_data: session.token_data.to_owned(),
        };
        let dvs = self.dao.session_data_cache.get(&cache_key).await;
        if let Some(mut dat) = dvs {
            dat.retain(|e| !data_key.contains(&e.0.as_str()));
            self.dao.session_data_cache.set(cache_key, dat, 0).await;
        }
        Ok(())
    }
}
