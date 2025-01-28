#[macro_use]
mod macros;
mod access;
mod cache;
mod op;
mod res;
mod result;
mod role;

use lsys_core::cache::LocalCacheConfig;
use lsys_core::{AppCoreError, RemoteNotify};
use lsys_logger::dao::ChangeLoggerDao;
use std::sync::Arc;

use sqlx::{MySql, Pool};

pub use access::*;
pub use cache::RbacLocalCacheClear;
pub use op::*;
pub use res::*;
pub use result::{RbacError, RbacResult};
pub use role::*;

//RBAC授权系统

pub struct RbacConfig {
    pub root_id_data: Vec<u64>,
    pub res_cache: LocalCacheConfig,
    pub op_cache: LocalCacheConfig,
    pub role_relation_cache: LocalCacheConfig,
    pub role_access_cache: LocalCacheConfig,
    pub delay_audit_num: usize, //审计日志延迟插入队列最大数量,超高并发加大此值提高并发响应速度
}

impl RbacConfig {
    pub fn new(root_id_data: Vec<u64>, use_cache: bool) -> Self {
        Self {
            root_id_data,
            role_relation_cache: LocalCacheConfig::new(
                "rbac-relation",
                if use_cache { None } else { Some(0) },
                None,
            ),
            role_access_cache: LocalCacheConfig::new(
                "rbac-role",
                if use_cache { None } else { Some(0) },
                None,
            ),
            res_cache: LocalCacheConfig::new(
                "rbac-res",
                if use_cache { None } else { Some(0) },
                None,
            ),
            op_cache: LocalCacheConfig::new(
                "rbac-op",
                if use_cache { None } else { Some(0) },
                None,
            ),
            delay_audit_num: 500,
        }
    }
}

pub struct RbacDao {
    pub access: Arc<RbacAccess>,
    pub role: Arc<RbacRole>,
    pub res: Arc<RbacRes>,
    pub op: Arc<RbacOp>,
}

impl RbacDao {
    pub async fn new(
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        config: RbacConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Result<RbacDao, AppCoreError> {
        let role = Arc::new(RbacRole::new(
            db.clone(),
            remote_notify.clone(),
            config.role_access_cache,
            logger.clone(),
        ));
        let res = Arc::new(RbacRes::new(
            db.clone(),
            role.clone(),
            remote_notify.clone(),
            config.res_cache,
            logger.clone(),
        ));
        let op = Arc::new(RbacOp::new(
            db.clone(),
            res.clone(),
            remote_notify,
            config.res_cache,
            logger,
        ));
        Ok(Self {
            access: Arc::new(RbacAccess::new(
                config.root_id_data,
                db,
                role.clone(),
                res.clone(),
                op.clone(),
                config.delay_audit_num,
            )),
            res,
            role,
            op,
        })
    }
}
