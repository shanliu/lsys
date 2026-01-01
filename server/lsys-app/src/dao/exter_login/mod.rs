mod feature;
use super::App;
use sqlx::MySql;
use sqlx::Pool;
use std::sync::Arc;
pub struct AppExterLogin {
    db: Pool<MySql>,
    app: Arc<App>,
}

impl AppExterLogin {
    pub fn new(db: Pool<MySql>, app: Arc<App>) -> Self {
        Self { db, app }
    }
}
