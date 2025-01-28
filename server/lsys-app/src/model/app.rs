use lsys_core::fluent_message;

use crate::dao::{AppError, AppResult};

use super::{AppModel, AppStatus};

impl AppModel {
    pub fn app_status_check(&self) -> AppResult<()> {
        if !AppStatus::Enable.eq(self.status) {
            return Err(AppError::AppBadStatus);
        }
        Ok(())
    }
    pub fn is_system_app_check(&self) -> AppResult<()> {
        if self.parent_app_id == 0 {
            Ok(())
        } else {
            Err(AppError::System(fluent_message!("app-is-not-system-app",{
                "name":&self.name,
                "client_id":&self.client_id,
            })))
        }
    }
}
