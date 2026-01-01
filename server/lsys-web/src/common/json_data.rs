use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct JsonData {
    pub code: String,
    pub sub_code: String,
    pub body: Option<Value>,
}
impl Default for JsonData {
    fn default() -> Self {
        JsonData {
            body: None,
            code: "200".to_string(),
            sub_code: "ok".to_string(),
        }
    }
}
impl JsonData {
    pub fn error() -> Self {
        JsonData::default().set_code(500).set_sub_code("system")
    }
    pub fn body(value: Value) -> Self {
        JsonData::default().set_body(value)
    }
    pub fn set_body(mut self, value: Value) -> Self {
        self.body = Some(value);
        self
    }
    pub fn set_total_body<T: ToString>(mut self, value: Value, total: T) -> Self {
        self.body = Some(json!({
            "total":total.to_string(),
            "data":value,
        }));
        self
    }
    pub fn set_code<T: ToString>(mut self, code: T) -> Self {
        self.code = code.to_string();
        self
    }
    pub fn set_sub_code<T: ToString>(mut self, sub_code: T) -> Self {
        self.sub_code = sub_code.to_string();
        self
    }
}
