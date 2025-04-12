use std::fmt::Display;

use serde_json::{json, Value};

use super::JsonData;

#[derive(Debug, Clone)]
pub struct JsonResponse {
    data: JsonData,
    message: String,
}
impl Default for JsonResponse {
    fn default() -> Self {
        JsonResponse {
            data: JsonData::default(),
            message: "ok".to_string(),
        }
    }
}
impl JsonResponse {
    pub fn data(data: JsonData) -> Self {
        JsonResponse {
            data,
            ..Default::default()
        }
    }
    pub fn message<T: Display>(msg: T) -> Self {
        JsonResponse {
            message: msg.to_string(),
            ..Default::default()
        }
    }
    pub fn set_data(mut self, data: JsonData) -> Self {
        self.data = data;
        self
    }
    pub fn set_message<T: ToString>(mut self, msg: T) -> Self {
        self.message = msg.to_string();
        self
    }
    pub fn to_value(&self) -> Value {
        let mut out_data = serde_json::Map::from_iter([(
            "result".to_string(),
            json!({
                "code": self.data.code,
                "state":self.data.sub_code,
                "message": self.message,
            }),
        )]);
        if let Some(ref body) = self.data.body {
            out_data.insert("response".to_string(), body.to_owned());
        }
        Value::Object(out_data)
    }
}
