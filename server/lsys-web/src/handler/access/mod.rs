use lsys_app::model::AppsModel;
use lsys_rbac::dao::RoleRelationKey;

fn app_relation_key(app: &AppsModel) -> Vec<RoleRelationKey> {
    vec![RoleRelationKey::system(format!("app-{}", app.id))]
}

mod admin;
mod app_oauth;
mod app_sender;
mod rbac;
mod user;
pub use admin::*;
pub use app_oauth::*;
pub use app_sender::*;
pub use rbac::*;
pub use user::*;
