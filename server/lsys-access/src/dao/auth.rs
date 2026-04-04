use std::fmt::Display;

use std::str::FromStr;
use std::sync::Arc;

use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::utils::{
    now_time, rand_str, string_clear, RandType, StringClear, STRING_CLEAR_FORMAT,
};
use lsys_core::valid_key;
use lsys_core::valid_param::{ValidIp, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen};

use crate::dao::AccessUser;
use lsys_core::db::{BatchInsert, Insert, QueryBuilderExt, TableMeta, Update};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool, QueryBuilder};

use crate::model::{SessionDataModel, SessionModel, SessionStatus, UserModel};

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
    pub async fn find_user_by_id(&self, id: &u64) -> AccessResult<UserModel> {
        Ok(
            lsys_core::db::utils::Fetch::<MySql, UserModel>::one(&self.db, |qb| {
                qb.field_eq("id", *id);
            })
            .await?,
        )
    }
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
    pub async fn clear_app_login(&self, user_app_id: u64) -> AccessResult<()> {
        if user_app_id == 0 {
            return Ok(());
        }
        let time = now_time()?;
        let status = SessionStatus::Delete as i8;
        Update::<_, SessionModel>::new()
            .set(SessionModel::STATUS, status)
            .set(SessionModel::LOGOUT_TIME, time)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("user_app_id", user_app_id);
            })
            .await?;
        let mut start_id = 0;
        loop {
            let mut qb = QueryBuilder::<MySql>::new(format!(
                "select id, oauth_app_id, token_data from {}",
                SessionModel::table_name()
            ));
            qb.push_where().field_eq("logout_time", time);
            qb.push_and().field_eq("status", status);
            qb.push_and().field_eq("user_app_id", user_app_id);
            qb.push_and().field_gt("id", start_id);
            qb.push(" ORDER BY id ASC LIMIT 100");
            
            let res = qb
                .build_query_as::<(u64, u64, String)>()
                .fetch_all(&self.db)
                .await?;
            if res.is_empty() {
                break;
            }
            for (next_id, oauth_app_id, token_data) in res {
                self.cache()
                    .del_session(user_app_id, oauth_app_id, &token_data)
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
    pub user_nickname: &'t str,
    pub token_data: Option<&'t str>,
    pub login_type: &'t str,
    pub login_data: Option<&'t AccessLoginData<'t>>,
}

impl AccessAuth {
    //登录
    async fn do_login_param_valid<TS: ToString>(
        &self,
        login_param: &AccessAuthLoginData<'_, TS>,
    ) -> AccessResult<String> {
        let user_data = login_param.user_data.to_string();
        let mut valid_param = ValidParam::default();
        valid_param
            .add(
                valid_key!("user_data"),
                &user_data,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(1, 32))
                    .add_rule(ValidPattern::Ident),
            )
            .add(
                valid_key!("user_nickname"),
                &login_param.user_nickname,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(0, 32))
                    .add_rule(ValidPattern::NotFormat),
            );

        if let Some(ref token_data) = login_param.token_data {
            valid_param.add(
                valid_key!("token_data"),
                token_data,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(16, 64))
                    .add_rule(ValidPattern::Ident),
            );
        }
        if let Some(login_data) = &login_param.login_data {
            if let Some(ref user_account) = login_data.user_account {
                valid_param.add(
                    valid_key!("user_account"),
                    user_account,
                    &ValidParamCheck::default()
                        .add_rule(ValidPattern::NotFormat)
                        .add_rule(ValidStrlen::max(128)),
                );
            }
            if let Some(ref login_ip) = login_data.login_ip {
                valid_param.add(
                    valid_key!("login_ip"),
                    login_ip,
                    &ValidParamCheck::default().add_rule(ValidIp::default()),
                );
            }
            for (key, _) in &login_data.session_data {
                valid_param.add(
                    valid_key!("session_data_key"),
                    key,
                    &ValidParamCheck::default()
                        .add_rule(ValidPattern::Ident)
                        .add_rule(ValidStrlen::range(1, 12)),
                );
            }
        }
        if let Some(token_data) = login_param.token_data {
            valid_param.add(
                valid_key!("token_data"),
                &token_data,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::max(64)),
            );
        }
        if let Some(user_account) = login_param.login_data
            && let Some(user_account) = user_account.user_account {
                valid_param.add(
                    valid_key!("user_account"),
                    &user_account,
                    &ValidParamCheck::default()
                        .add_rule(ValidPattern::NotFormat)
                        .add_rule(ValidStrlen::max(128)),
                );
            }
        valid_param.check()?;
        Ok(user_data)
    }
    //登录
    pub async fn do_login<TS: ToString>(
        &self,
        login_param: &AccessAuthLoginData<'_, TS>,
    ) -> AccessResult<SessionBody> {
        let user_data = self.do_login_param_valid(login_param).await?;
        let token_data = login_param
            .token_data
            .map(|e| e.to_owned())
            .unwrap_or_else(|| rand_str(RandType::LowerHex, 32));

        let user_account = login_param
            .login_data
            .as_ref()
            .map(|e| e.user_account.to_owned().unwrap_or_default())
            .unwrap_or_default();

        let time = now_time().unwrap_or_default();

        let user_id = self
            .user
            .sync_user(
                login_param.app_id,
                user_data.as_str(),
                Some(login_param.user_nickname),
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
        let session_data = login_param
            .login_data
            .as_ref()
            .map(|e| e.session_data.to_owned())
            .unwrap_or_default();
        let session_data_tmp = session_data
            .iter()
            .map(|e| (e.0.to_owned(), e.1.to_string()))
            .collect::<Vec<_>>();

        match sqlx::query_as::<_, (u64, i8, String, String)>(&format!(
            "select id,status,device_id,login_ip from {} where user_app_id=? and token_data = ?",
            SessionModel::table_name(),
        ))
        .bind(login_param.app_id)
        .bind(token_data.clone())
        .fetch_one(&self.db)
        .await
        {
            Ok((sid, db_status, db_device_id, db_login_ip)) => {
                if !SessionStatus::Enable.eq(db_status)
                    || device_id != db_device_id
                    || login_ip != db_login_ip
                {
                    return Err(AccessError::LoginTokenDataExit(sid));
                }
                let mut db = self.db.begin().await?;
                if let Err(err) = Update::<_, SessionModel>::new()
                    .set(SessionModel::DEVICE_NAME, device_name)
                    .set(SessionModel::EXPIRE_TIME, expire_time)
                    .execute(&self.db, |qb| {
                        qb.push_where().field_eq("id", sid);
                    })
                    .await
                {
                    db.rollback().await?;
                    return Err(err.into());
                }
                if !session_data_tmp.is_empty() {
                    for t in session_data_tmp.iter() {
                        if let Err(err) = Insert::<_, SessionDataModel>::new()
                            .set(SessionDataModel::SESSION_ID, sid)
                            .set(SessionDataModel::DATA_KEY, &t.0)
                            .set(SessionDataModel::DATA_VAL, &t.1)
                            .set(SessionDataModel::CHANGE_TIME, time)
                            .execute_update(
                                Update::<_, SessionDataModel>::new()
                                    .set(SessionDataModel::DATA_VAL, &t.1)
                                    .set(SessionDataModel::CHANGE_TIME, time),
                                &mut *db,
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(err.into());
                        }
                    }
                };
                db.commit().await?;

                self.cache()
                    .del_session(login_param.app_id, login_param.oauth_app_id, &token_data)
                    .await?;
            }
            Err(sqlx::Error::RowNotFound) => {
                let mut db = self.db.begin().await?;

                let sid = match Insert::<_, SessionModel>::new()
                    .set(SessionModel::USER_ID, user_id)
                    .set(SessionModel::USER_APP_ID, login_param.app_id)
                    .set(SessionModel::OAUTH_APP_ID, login_param.oauth_app_id)
                    .set(SessionModel::TOKEN_DATA, &token_data)
                    .set(SessionModel::LOGIN_TYPE, login_type)
                    .set(SessionModel::LOGIN_IP, login_ip)
                    .set(SessionModel::DEVICE_ID, device_id)
                    .set(SessionModel::DEVICE_NAME, device_name)
                    .set(SessionModel::STATUS, SessionStatus::Enable as i8)
                    .set(SessionModel::ADD_TIME, time)
                    .set(SessionModel::EXPIRE_TIME, expire_time)
                    .set(SessionModel::LOGOUT_TIME, 0u64)
                    .execute(&mut *db)
                    .await
                {
                    Ok(id) => id.last_insert_id(),
                    Err(err) => {
                        db.rollback().await?;
                        return Err(err.into());
                    }
                };

                if !session_data.is_empty() {
                    let mut batch =
                        BatchInsert::<_, SessionDataModel>::with_capacity(session_data_tmp.len());
                    for t in session_data_tmp.iter() {
                        batch = batch.push(
                            Insert::<_, SessionDataModel>::new()
                                .set(SessionDataModel::SESSION_ID, sid)
                                .set(SessionDataModel::DATA_KEY, &t.0)
                                .set(SessionDataModel::DATA_VAL, &t.1)
                                .set(SessionDataModel::CHANGE_TIME, time),
                        );
                    }
                    if let Err(err) = batch.execute(&mut *db).await {
                        db.rollback().await?;
                        return Err(err.into());
                    }
                }
                db.commit().await?;
            }
            Err(err) => Err(err)?,
        };

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
        Update::<_, SessionModel>::new()
            .set(SessionModel::EXPIRE_TIME, expire_time)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", session.id);
            })
            .await?;
        self.cache()
            .del_session(
                session.user_app_id,
                session.oauth_app_id,
                &session.token_data,
            )
            .await?;
        session.expire_time = expire_time;
        self.load_session_body(session).await
    }
    //退出登录
    pub async fn do_logout(&self, session_body: &SessionBody) -> AccessResult<()> {
        let time = now_time()?;
        let status = SessionStatus::Delete as i8;
        Update::<_, SessionModel>::new()
            .set(SessionModel::STATUS, status)
            .set(SessionModel::LOGOUT_TIME, time)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", session_body.session().id);
            })
            .await?;
        self.cache()
            .del_session(
                session_body.session().user_app_id,
                session_body.session().oauth_app_id,
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
        let token_data = string_clear(
            token_data,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(65),
        );
        if token_data.is_empty() {
            return Err(AccessError::NotLogin);
        }

        let data = sqlx::query_as::<_, SessionModel>(&format!(
            "select * from {} where user_app_id=? and token_data=? and status=? and oauth_app_id=?",
            SessionModel::table_name(),
        ))
        .bind(app_id)
        .bind(token_data)
        .bind(SessionStatus::Enable as i8)
        .bind(oauth_app_id)
        .fetch_one(&self.db)
        .await
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
        let data_key = data_key
            .iter()
            .map(|e| string_clear(e, StringClear::Ident, Some(13)))
            .filter(|e| !e.is_empty())
            .collect::<Vec<String>>();
        session_body.valid()?;
        if data_key.is_empty() {
            return Ok(vec![]);
        }
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select * from {}",
            SessionDataModel::table_name(),
        ));
        qb.push_where().field_eq("session_id", session_body.session().id);
        qb.push_and().field_in_string("data_key", &data_key);
        let data = qb
            .build_query_as::<SessionDataModel>()
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
    async fn session_set_vec_data_param_valid(&self, data: &[(&str, &str)]) -> AccessResult<()> {
        let mut param_valid = ValidParam::default();
        for tmp in data {
            param_valid.add(
                valid_key!("session_data_key"),
                &tmp.0,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, 12)),
            );
            param_valid.add(
                valid_key!("session_data_val"),
                &tmp.1,
                &ValidParamCheck::default().add_rule(ValidStrlen::range(0, 20000)),
            );
        }
        param_valid.check()?;
        Ok(())
    }
    //登陆附带数据批量设置
    pub async fn session_set_vec_data(
        &self,
        session_body: &SessionBody,
        data: &[(&str, &str)],
    ) -> AccessResult<()> {
        self.session_set_vec_data_param_valid(data).await?;
        session_body.valid()?;

        let time = now_time()?;

        let mut db = self.db.begin().await?;

        for (data_key, data_val) in data {
            let data_key = data_key.to_string();
            let data_val = data_val.to_string();
            if let Err(err) = Insert::<_, SessionDataModel>::new()
                .set(SessionDataModel::SESSION_ID, session_body.session().id)
                .set(SessionDataModel::DATA_KEY, &data_key)
                .set(SessionDataModel::DATA_VAL, &data_val)
                .set(SessionDataModel::CHANGE_TIME, time)
                .execute_update(
                    Update::<_, SessionDataModel>::new()
                        .set(SessionDataModel::DATA_VAL, &data_val)
                        .set(SessionDataModel::CHANGE_TIME, time),
                    &mut *db,
                )
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
            set_val.extend(data.clone());
            self.dao.session_data_cache.set(cache_key, set_val, 0).await;
            out.extend(data);
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
        app_id: u64,
        oauth_app_id: u64,
        token_data: &str,
    ) -> AccessResult<()> {
        let cache_key = AccessAuthSessionCacheKey {
            app_id,
            oauth_app_id,
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
