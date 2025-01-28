use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct LogBarCodeCreateConfig<'t> {
    pub action: &'t str,
    pub user_id: u64,
    pub barcode_type: &'t str,
    pub image_format: &'t str,
    pub image_width: i32,
    pub image_height: i32,
    pub margin: i32,
    pub image_color: &'t str,
    pub image_background: &'t str,
}

impl ChangeLogData for LogBarCodeCreateConfig<'_> {
    fn log_type() -> &'static str {
        "barcode-create-config"
    }
    fn message(&self) -> String {
        format!("{} create config", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogBarCodeParseRecord<'t> {
    pub action: &'t str,
    pub count: usize,
    pub user_id: u64,
    pub message: &'t str,
}

impl ChangeLogData for LogBarCodeParseRecord<'_> {
    fn log_type() -> &'static str {
        "barcode-parse-record"
    }
    fn message(&self) -> String {
        format!("{} record", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
