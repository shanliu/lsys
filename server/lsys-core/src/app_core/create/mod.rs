mod base;

pub use base::*;

use async_trait::async_trait;

use sqlx::Pool;

use tera::Tera;

use crate::FluentMgr;

use super::result::AppCoreError;
use super::AppCore;

#[async_trait]
pub trait AppCoreCreate: Sync + Send {
    async fn init_tracing(&self, config: &AppCore) -> Result<(), AppCoreError>;
    async fn create_db(&self, app_core: &AppCore) -> Result<Pool<sqlx::MySql>, AppCoreError>;
    async fn create_redis_client(&self, app_core: &AppCore) -> Result<redis::Client, AppCoreError>;
    async fn create_redis_pool(
        &self,
        app_core: &AppCore,
    ) -> Result<deadpool_redis::Pool, AppCoreError>;
    async fn create_tera(&self, app_core: &AppCore) -> Result<Tera, AppCoreError>;
    async fn create_fluent(&self, app_core: &AppCore) -> Result<FluentMgr, AppCoreError>;
}
