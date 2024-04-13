use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct LogBarCodeCreateConfig {
    pub action: &'static str,
    pub barcode_type: String,
    pub image_format: String,
    pub image_width: i32,
    pub image_height:i32,
    pub margin: i32,
    pub image_color:String,
    pub image_background: String,
  
}

impl ChangeLogData for LogBarCodeCreateConfig {
    fn log_type<'t>() -> &'t str {
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
pub(crate) struct LogBarCodeParseRecord {
    pub action: &'static str,
    pub count:usize,
    pub message: &'static str,
}

impl ChangeLogData for LogBarCodeParseRecord {
    fn log_type<'t>() -> &'t str {
        "barcode-parse-record"
    }
    fn message(&self) -> String {
        format!("{} record", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}