use super::{data::AppOAuthServerScopeData, AppOAuthServer};
use crate::{dao::AppResult, model::AppModel};
impl AppOAuthServer {
    pub fn cache(&'_ self) -> AppOAuthServerCache<'_> {
        AppOAuthServerCache { dao: self }
    }
}
pub struct AppOAuthServerCache<'t> {
    pub dao: &'t AppOAuthServer,
}

impl AppOAuthServerCache<'_> {
    pub async fn get_scope(&self, app: &AppModel) -> AppResult<Vec<AppOAuthServerScopeData>> {
        app.app_status_check()?;
        match self.dao.oauth_server_scope_cache.get(&app.id).await {
            Some(dat) => Ok(dat),
            None => {
                let oa = self.dao.get_scope(app).await?;
                self.dao
                    .oauth_server_scope_cache
                    .set(app.id, oa.clone(), 0)
                    .await;
                Ok(oa)
            }
        }
    }
}
