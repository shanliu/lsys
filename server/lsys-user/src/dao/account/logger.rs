use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct LogUserAddress {
    pub action: &'static str,
    pub address_code: String,
    pub address_info: String,
    pub address_detail: String,
    pub name: String,
    pub mobile: String,
}

impl ChangeLogData for LogUserAddress {
    fn log_type<'t>() -> &'t str {
        "user-address"
    }
    fn message(&self) -> String {
        format!("{} role tag ", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogUserEmail {
    pub action: &'static str,
    pub email: String,
    pub status: i8,
}

impl ChangeLogData for LogUserEmail {
    fn log_type<'t>() -> &'t str {
        "user-email"
    }
    fn message(&self) -> String {
        format!("{} role tag ", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogUserExternal {
    pub action: &'static str,
    pub config_name: String,
    pub external_type: String,
    pub external_id: String,
    pub external_name: String,
    pub external_gender: String,
    pub external_link: String,
    pub external_pic: String,
    pub external_nikename: String,
    pub status: i8,
    pub token_data: String,
    pub token_timeout: u64,
}

impl ChangeLogData for LogUserExternal {
    fn log_type<'t>() -> &'t str {
        "user-external"
    }
    fn message(&self) -> String {
        format!("{} {} ", self.action, self.external_id)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogUserInfo {
    pub gender: i32,
    pub headimg: String,
    pub birthday: String,
    pub reg_ip: String,
    pub reg_from: String,
}

impl ChangeLogData for LogUserInfo {
    fn log_type<'t>() -> &'t str {
        "user-info"
    }
    fn message(&self) -> String {
        "set user info ".to_string()
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogUserMobile {
    pub action: &'static str,
    pub area_code: String,
    pub mobile: String,
    pub status: i8,
}

impl ChangeLogData for LogUserMobile {
    fn log_type<'t>() -> &'t str {
        "user-mobile"
    }
    fn message(&self) -> String {
        format!("{} ", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogUserName {
    pub action: &'static str,
    pub username: String,
}

impl ChangeLogData for LogUserName {
    fn log_type<'t>() -> &'t str {
        "user-name"
    }
    fn message(&self) -> String {
        format!("{} name {} ", self.action, self.username)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogUser {
    pub action: &'static str,
    pub nickname: String,
    pub status: i8,
}

impl ChangeLogData for LogUser {
    fn log_type<'t>() -> &'t str {
        "user"
    }
    fn message(&self) -> String {
        format!("{} user {} ", self.action, self.nickname)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
