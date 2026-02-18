mod base;

pub use base::*;

use async_trait::async_trait;
#[cfg(feature = "db")]
use sqlx::Pool;
#[cfg(feature = "tera")]
use tera::Tera;

use crate::FluentMgr;

use super::result::AppCoreError;
use super::AppCore;

#[async_trait]
pub trait AppCoreCreate: Sync + Send {
    async fn init_tracing(&self, config: &AppCore) -> Result<(), AppCoreError>;
    #[cfg(feature = "db")]
    async fn create_db(&self, app_core: &AppCore) -> Result<Pool<sqlx::MySql>, AppCoreError>;
    #[cfg(feature = "redis")]
    async fn create_redis_client(&self, app_core: &AppCore) -> Result<redis::Client, AppCoreError>;
    #[cfg(feature = "redis")]
    async fn create_redis_pool(
        &self,
        app_core: &AppCore,
    ) -> Result<deadpool_redis::Pool, AppCoreError>;
    #[cfg(feature = "tera")]
    async fn create_tera(&self, app_core: &AppCore) -> Result<Tera, AppCoreError>;
    async fn create_fluent(&self, app_core: &AppCore) -> Result<FluentMgr, AppCoreError>;
}
