use std::sync::Arc;

use lsys_access::dao::AccessDao;

pub struct WebAccess {
    pub access_dao: Arc<AccessDao>,
}

impl WebAccess {
    pub fn new(access_dao: Arc<AccessDao>) -> Self {
        Self { access_dao }
    }
    // 已登录用户列表 待实现
}
