use lsys_core::now_time;

use crate::dao::AppResult;
use crate::dao::AppSecret;
use crate::model::AppSecretType;

use super::{AppSecretCacheKey, AppSecretRecrod};
impl AppSecret {
    pub fn cache(&'_ self) -> AppSecretCache<'_> {
        AppSecretCache { dao: self }
    }
}
pub struct AppSecretCache<'t> {
    pub dao: &'t AppSecret,
}

impl AppSecretCache<'_> {
    //通过CLIENT_ID查找应用
    pub async fn multiple_find_secret_by_app_id(
        &self,
        app_id: u64,
        secret_type: AppSecretType,
    ) -> AppResult<Vec<AppSecretRecrod>> {
        let cache_key = AppSecretCacheKey {
            secret_type: secret_type as i8,
            app_id,
        };
        Ok(
            match self
                .dao
                .secret_cache
                .get(&cache_key)
                .await
                .and_then(|data| {
                    let tmp_len = data.len();
                    let ntime = now_time().unwrap_or_default();
                    let tmp_data = data
                        .into_iter()
                        .flat_map(|e| {
                            if e.time_out == 0 || e.time_out > ntime {
                                Some(e)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    if tmp_data.is_empty() {
                        return None;
                    }
                    Some((tmp_data.len() != tmp_len, tmp_data))
                }) {
                Some((change, data)) => {
                    if change {
                        self.dao.secret_cache.set(cache_key, data.clone(), 0).await;
                    }
                    data
                }
                None => {
                    let data = self
                        .dao
                        .multiple_find_secret_by_app_id(app_id, secret_type)
                        .await?;
                    self.dao.secret_cache.set(cache_key, data.clone(), 0).await;
                    data
                }
            },
        )
    }
    pub async fn single_find_secret_app_id(
        &self,
        app_id: u64,
        secret_type: AppSecretType,
    ) -> AppResult<AppSecretRecrod> {
        let cache_key = AppSecretCacheKey {
            secret_type: secret_type as i8,
            app_id,
        };
        let ntime = now_time().unwrap_or_default();
        let tmp = self
            .dao
            .secret_cache
            .get(&cache_key)
            .await
            .and_then(|mut e| {
                e.pop().and_then(|s| {
                    if s.time_out == 0 || s.time_out > ntime {
                        Some(s)
                    } else {
                        None
                    }
                })
            });
        Ok(match tmp {
            Some(data) => data,
            None => {
                let data = self
                    .dao
                    .single_find_secret_app_id(app_id, secret_type)
                    .await?;
                self.dao
                    .secret_cache
                    .set(cache_key, vec![data.clone()], 0)
                    .await;
                data
            }
        })
    }
}
