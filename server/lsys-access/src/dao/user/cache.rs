use std::collections::{HashMap, HashSet};

use crate::model::UserModel;

use super::{
    info::{UserInfo, UserInfoSet},
    AccessResult, AccessUser, AccessUserAppUserKey,
};

pub struct AccessUserCache<'t> {
    pub dao: &'t AccessUser,
}
impl AccessUserCache<'_> {
    //通过ID获取用户
    lsys_core::impl_cache_fetch_one!(find_by_id, dao, user_cache, u64, AccessResult<UserModel>);
    lsys_core::impl_cache_fetch_vec!(
        find_by_ids,
        dao,
        user_cache,
        u64,
        AccessResult<HashMap<u64, UserModel>>
    );
    ///获取用户信息
    pub async fn find_user_by_data(
        &self,
        app_id: u64,
        user_data: impl ToString,
        privacy: bool,
    ) -> AccessResult<UserInfo> {
        let user_data = user_data.to_string();
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
        Ok(UserInfo::from_user_model(user_model, privacy))
    }
    ///获取用户信息
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
}
