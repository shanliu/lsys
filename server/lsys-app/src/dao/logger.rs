use lsys_logger::dao::ChangeLogData;
use serde::Serialize;
//日志
#[derive(Serialize)]
pub(crate) struct AppLog<'t> {
    pub action: &'t str,
    pub name: &'t str,
    pub user_id: u64,
    pub status: i8,
    pub parent_app_id: u64,
    pub user_app_id: u64,
    pub client_id: &'t str,
    pub client_secret: Option<&'t str>,
}

impl ChangeLogData for AppLog<'_> {
    fn log_type() -> &'static str {
        "app"
    }
    fn message(&self) -> String {
        format!("{}:{}[{}]", self.action, self.name, self.client_id)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct AppRequestLog<'t> {
    pub action: &'t str,
    pub parent_app_id: u64,
    pub app_id: u64,
    pub user_id: u64,
    pub request_type: i8,
    pub status: i8,
    pub req_data: Option<&'t str>,
}

impl ChangeLogData for AppRequestLog<'_> {
    fn log_type() -> &'static str {
        "app-request"
    }
    fn message(&self) -> String {
        format!("parent app {}", self.parent_app_id,)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct AppOAuthClientSetDomainLog<'t> {
    pub parent_app_id: u64,
    pub app_id: u64,
    pub user_id: u64,
    pub callback_domain: &'t str,
}

impl ChangeLogData for AppOAuthClientSetDomainLog<'_> {
    fn log_type() -> &'static str {
        "app-oauth-client-set"
    }
    fn message(&self) -> String {
        format!("parent app {}", self.parent_app_id,)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct AppOAuthClientSecretSetLog<'t> {
    pub action: &'t str,
    pub parent_app_id: u64,
    pub app_id: u64,
    pub user_id: u64,
    pub oauth_secret: &'t str,
}

impl ChangeLogData for AppOAuthClientSecretSetLog<'_> {
    fn log_type() -> &'static str {
        "app-oauth-client-secret-set"
    }
    fn message(&self) -> String {
        format!("parent app {}", self.parent_app_id,)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct AppOAuthServerSetLog<'t> {
    pub parent_app_id: u64,
    pub app_id: u64,
    pub user_id: u64,
    pub scope_data: &'t str,
}

impl ChangeLogData for AppOAuthServerSetLog<'_> {
    fn log_type() -> &'static str {
        "app-oauth-server-set"
    }
    fn message(&self) -> String {
        format!("parent app {}", self.parent_app_id,)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct AppViewSecretLog<'t> {
    pub action: &'static str,
    pub app_id: u64,
    pub user_id: u64,
    pub app_name: &'t str,
    pub secret_data: &'t str,
}

impl ChangeLogData for AppViewSecretLog<'_> {
    fn log_type() -> &'static str {
        "app-view-secret"
    }
    fn message(&self) -> String {
        format!("{} view secret ", self.app_name)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
