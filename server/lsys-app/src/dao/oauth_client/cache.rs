

use crate::{dao:: AppResult, model::{AppModel, AppOAuthClientModel}};
use super::AppOAuthClient;
impl AppOAuthClient {
    pub fn cache(&'_ self) ->AppOAuthClientCache<'_> {
        AppOAuthClientCache { dao: self }
    }
}
pub struct AppOAuthClientCache<'t> {
    pub dao: &'t AppOAuthClient,
}

impl AppOAuthClientCache<'_> {
    pub async fn find_by_app(&self, app: &AppModel) -> AppResult<AppOAuthClientModel> {
        self.dao.oauth_check(app).await?;
        match self.dao.oauth_client_cache.get(&app.id).await{
            Some(dat) => Ok(dat),
            None => {
                let oa=self.dao.inner_find_by_app(app).await?;
                self.dao.oauth_client_cache.set(app.id, oa.clone(), 0).await;
                Ok(oa)
            },
        }
    }
}