use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct LogAccountAddress<'t> {
    pub action: &'t str,
    pub address_code: &'t str,
    pub address_info: &'t str,
    pub address_detail: &'t str,
    pub name: &'t str,
    pub mobile: &'t str,
    pub account_id: u64,
}

impl ChangeLogData for LogAccountAddress<'_> {
    fn log_type() -> &'static str {
        "account-address"
    }
    fn message(&self) -> String {
        format!("{} account address", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogAccountEmail<'t> {
    pub action: &'t str,
    pub email: &'t str,
    pub status: i8,
    pub account_id: u64,
}

impl ChangeLogData for LogAccountEmail<'_> {
    fn log_type() -> &'static str {
        "account-email"
    }
    fn message(&self) -> String {
        format!("{} role tag ", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogAccountExternal<'t> {
    pub action: &'t str,
    pub config_name: &'t str,
    pub external_type: &'t str,
    pub external_id: &'t str,
    pub external_name: &'t str,
    pub external_gender: &'t str,
    pub external_link: &'t str,
    pub external_pic: &'t str,
    pub external_nikename: &'t str,
    pub status: i8,
    pub token_data: &'t str,
    pub token_timeout: u64,
    pub account_id: u64,
}

impl ChangeLogData for LogAccountExternal<'_> {
    fn log_type() -> &'static str {
        "account-external"
    }
    fn message(&self) -> String {
        format!("{} {} ", self.action, self.external_id)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogAccountInfo<'t> {
    pub gender: i32,
    pub headimg: &'t str,
    pub birthday: &'t str,
    pub reg_ip: &'t str,
    pub reg_from: &'t str,
}

impl ChangeLogData for LogAccountInfo<'_> {
    fn log_type() -> &'static str {
        "account-info"
    }
    fn message(&self) -> String {
        "set account info ".to_string()
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogAccountMobile<'t> {
    pub action: &'t str,
    pub area_code: &'t str,
    pub mobile: &'t str,
    pub status: i8,
    pub account_id: u64,
}

impl ChangeLogData for LogAccountMobile<'_> {
    fn log_type() -> &'static str {
        "account-mobile"
    }
    fn message(&self) -> String {
        format!("{} ", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogAccountName<'t> {
    pub action: &'t str,
    pub username: &'t str,
}

impl ChangeLogData for LogAccountName<'_> {
    fn log_type() -> &'static str {
        "account-name"
    }
    fn message(&self) -> String {
        format!("{} name {} ", self.action, self.username)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogAccount<'t> {
    pub action: &'t str,
    pub nickname: &'t str,
    pub status: i8,
}

impl ChangeLogData for LogAccount<'_> {
    fn log_type() -> &'static str {
        "user"
    }
    fn message(&self) -> String {
        format!("{} user {} ", self.action, self.nickname)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]

pub(crate) struct LogAccountPassWrod {
    pub account_id: u64,
}
impl ChangeLogData for LogAccountPassWrod {
    fn log_type() -> &'static str {
        "set-password"
    }
    fn message(&self) -> String {
        "set passwrod".to_string()
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
