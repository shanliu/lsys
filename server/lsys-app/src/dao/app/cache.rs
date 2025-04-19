use crate::model::{AppSecretType, AppStatus};
use crate::{
    dao::{app::App, AppError, AppResult},
    model::AppModel,
};
use lsys_core::fluent_message;
impl App {
    pub fn cache(&'_ self) -> AppCache<'_> {
        AppCache { dao: self }
    }
}
pub struct AppCache<'t> {
    pub dao: &'t App,
}

impl AppCache<'_> {
    //通过ID查找应用
    lsys_core::impl_cache_fetch_one!(find_by_id, dao, id_cache, u64, AppResult<AppModel>);
    //通过CLIENT_ID查找应用
    pub async fn find_by_client_id(&self, client_id: &str) -> AppResult<AppModel> {
        match self.dao.client_id_cache.get(&client_id.to_string()).await {
            Some(data) => match data {
                Some(did) => self.find_by_id(&did).await,
                None => Err(AppError::AppNotFound(client_id.to_owned())),
            },
            None => match self.dao.find_by_client_id(client_id).await {
                Ok(data) => {
                    self.dao
                        .client_id_cache
                        .set(client_id.to_owned(), Some(data.id), 0)
                        .await;
                    self.dao.id_cache.set(data.id, data.clone(), 0).await;
                    Ok(data)
                }
                Err(e) => {
                    if let AppError::AppNotFound(_) = &e {
                        self.dao
                            .client_id_cache
                            .set(client_id.to_owned(), None, 0)
                            .await;
                    }
                    Err(e)
                }
            },
        }
    }
    pub async fn find_notify_secret_by_app_id(&self, app_id: u64) -> AppResult<String> {
        let secret_data = self
            .dao
            .app_secret
            .cache()
            .single_find_secret_app_id(app_id, AppSecretType::Notify)
            .await?;
        Ok(secret_data.secret_data)
    }
    //内部APP secret 获取
    pub async fn find_app_secret_by_client_id(
        &self,
        client_id: &str,
    ) -> Result<Vec<String>, AppError> {
        let apps = self.find_by_client_id(client_id).await;
        match apps {
            Ok(app) => {
                if !AppStatus::Enable.eq(app.status) {
                    return Err(
                        AppError::System(fluent_message!("app-find-bad-status",{
                            "client_id":client_id
                        })), //,"your app id [{$client_id}] not confrim "
                    );
                }
                let sercet_data = self
                    .dao
                    .app_secret
                    .cache()
                    .multiple_find_secret_by_app_id(app.id, AppSecretType::App)
                    .await?;
                Ok(sercet_data
                    .into_iter()
                    .map(|e| e.secret_data)
                    .collect::<Vec<_>>())
            }
            Err(err) => Err(err),
        }
    }
    //获取子应用信息
    pub async fn find_sub_app_by_client_id(
        &self,
        app: &AppModel,
        client_id: &str,
    ) -> AppResult<AppModel> {
        match self.find_by_client_id(client_id).await {
            Ok(apps) => {
                if apps.parent_app_id != app.id {
                    return Err(AppError::AppNotFound(client_id.to_string()));
                }
                if !AppStatus::Enable.eq(app.status) {
                    return Err(
                        AppError::System(fluent_message!("app-find-bad-status",{
                            "client_id":client_id
                        })), //,"your app id [{$client_id}] not confrim "
                    );
                }
                Ok(apps)
            }
            Err(err) => Err(err),
        }
    }
}

//feature
impl AppCache<'_> {
    //检测指定功能是否启用
    pub async fn feature_check(&self, app: &AppModel, featuer_data: &[&str]) -> AppResult<()> {
        let mut cdat = self
            .dao
            .feature_cache
            .get(&app.id)
            .await
            .unwrap_or_default();
        let mut bdat = vec![];
        for tmp in featuer_data {
            if !cdat.iter().any(|ctmp| ctmp.1 && ctmp.0.as_str() == *tmp) {
                bdat.push(*tmp);
            }
        }
        if let Err(err) = self.dao.feature_check(app, &bdat).await {
            match err {
                AppError::AppBadFeature(name, bad_fs) => {
                    for tmp in bdat {
                        let stmp = tmp.to_string();
                        if !bad_fs.contains(&stmp) {
                            cdat.push((stmp, true));
                        }
                    }
                    for tmp in bad_fs.clone() {
                        cdat.push((tmp.to_string(), false));
                    }
                    self.dao.feature_cache.set(app.id, cdat, 0).await;
                    return Err(AppError::AppBadFeature(name, bad_fs));
                }
                _ => return Err(err),
            }
        }
        for tmp in bdat {
            cdat.push((tmp.to_string(), true));
        }
        self.dao.feature_cache.set(app.id, cdat, 0).await;
        Ok(())
    }
    //检测指定外部功能是否启用
    pub async fn exter_feature_check(
        &self,
        app: &AppModel,
        featuer_data: &[&str],
    ) -> AppResult<()> {
        let feature_key = featuer_data
            .iter()
            .map(|e| self.dao.exter_feature_key(e))
            .collect::<Vec<String>>();
        let check_key = &feature_key.iter().map(|e| e.as_str()).collect::<Vec<_>>();
        if app.parent_app_id > 0 {
            let papp = self.find_by_id(&app.parent_app_id).await?;
            self.feature_check(&papp, check_key).await?;
        }
        self.feature_check(app, check_key).await
    }
}
