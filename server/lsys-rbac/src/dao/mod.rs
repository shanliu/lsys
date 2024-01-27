use std::sync::Arc;

pub mod rbac;

pub use self::rbac::*;

use lsys_core::{AppCoreError, RemoteNotify};

use lsys_logger::dao::ChangeLogger;
use sqlx::{MySql, Pool};

pub struct RbacDao {
    //内部依赖
    pub db: Pool<MySql>,
    //   pub redis: deadpool_redis::Pool,
    // 权限相关
    pub rbac: Arc<Rbac>,
}

impl RbacDao {
    pub async fn new(
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        logger: Arc<ChangeLogger>,
        system_role: Option<Box<dyn SystemRoleCheckData>>,
        use_cache: bool,
    ) -> Result<RbacDao, AppCoreError> {
        let rbac = Arc::from(Rbac::new(
            // fluents_message.clone(),
            db.clone(),
            remote_notify,
            system_role,
            use_cache,
            logger,
        ));
        Ok(RbacDao {
            // fluent: fluents_message,
            rbac,
            db,
            // redis,
        })
    }
}
