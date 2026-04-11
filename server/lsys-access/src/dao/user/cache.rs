use std::collections::{HashMap, HashSet};

use crate::{dao::AccessError, model::UserModel};

use super::{
    AccessResult, AccessUser, AccessUserAppUserKey,
    info::{UserInfo, UserInfoSet},
};

pub struct AccessUserCache<'t> {
    pub dao: &'t AccessUser,
}
impl AccessUserCache<'_> {
    pub async fn find_by_id(&self, id: &u64) -> AccessResult<UserModel> {
        self.dao
            .user_cache
            .get_or_fetch(id, || self.dao.find_by_id(id))
            .await
    }
    pub async fn find_by_ids(&self, ids: &[u64]) -> AccessResult<HashMap<u64, UserModel>> {
        self.dao
            .user_cache
            .get_or_fetch_many(ids, |missing| async move {
                self.dao.find_by_ids(&missing).await
            })
            .await
    }
    ///获取用户信息
    pub async fn find_user_by_data(
        &self,
        app_id: u64,
        user_data: impl ToString,
    ) -> AccessResult<UserInfo> {
        let user_data = user_data.to_string(); //cache层不校验
        let key = AccessUserAppUserKey {
            app_id,
            user_data: user_data.clone(),
        };
        let user_model = match self.dao.app_user_data.get(&key).await {
            Some(data) => self.find_by_id(&data).await,
            None => match self.dao.find_by_data(app_id, &user_data).await {
                Ok(data) => {
                    self.dao.user_cache.set(data.id, data.clone(), 0).await;
                    self.dao.app_user_data.set(key, data.id, 0).await;
                    Ok(data)
                }
                Err(e) => Err(e),
            },
        }?;
        Ok(UserInfo::from(user_model))
    }
    ///批量获取用户信息
    pub async fn find_users_by_ids(&self, ids: &[u64]) -> AccessResult<UserInfoSet> {
        let ids = ids
            .iter()
            .copied()
            .collect::<HashSet<u64>>()
            .into_iter()
            .collect::<Vec<_>>();
        if ids.is_empty() {
            return Ok(UserInfoSet::new(HashMap::new()));
        }
        Ok(UserInfoSet::new(self.find_by_ids(&ids).await?))
    }
    ///获取一个用户信息
    pub async fn find_user_by_id(&self, id: u64) -> AccessResult<UserInfo> {
        Ok(UserInfo::from(self.find_by_id(&id).await?))
    }
    //带缓存的同步用户
    pub async fn sync_user(
        &self,
        app_id: u64,
        user_data: impl ToString,
        user_nickname: Option<&str>,
        user_account: Option<&str>,
    ) -> AccessResult<UserInfo> {
        let res = self.find_user_by_data(app_id, &user_data.to_string()).await;
        match res {
            Ok(e) => {
                let mut is_sync = false;
                if let Some(name_val) = user_nickname
                    && e.user_nickname.as_str() != name_val
                {
                    is_sync = true;
                }
                if let Some(account_val) = user_account
                    && e.user_account.as_str() != account_val
                {
                    is_sync = true;
                }
                if !is_sync {
                    Ok(e)
                } else {
                    let id = self
                        .dao
                        .sync_user(app_id, user_data, user_nickname, user_account)
                        .await?;
                    self.find_by_id(&id).await.map(|e| e.into())
                }
            }
            Err(err) => {
                if let AccessError::Sqlx(terr) = &err {
                    if matches!(terr, sqlx::Error::RowNotFound) {
                        let id = self
                            .dao
                            .sync_user(app_id, user_data, user_nickname, user_account)
                            .await?;
                        self.find_by_id(&id).await.map(|e| e.into())
                    } else {
                        Err(err)
                    }
                } else {
                    Err(err)
                }
            }
        }
    }
}
