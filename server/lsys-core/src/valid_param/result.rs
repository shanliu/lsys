use serde_json::Value;

use crate::{fluent_message, FluentData, FluentMessage, IntoFluentMessage};

#[derive(Debug)]
pub struct ValidRuleError {
    message: FluentMessage,
    data: Option<Value>,
}
impl ValidRuleError {
    pub fn new(message: FluentMessage) -> Self {
        Self {
            message,
            data: None,
        }
    }
    pub fn set_data(&mut self, data: Value) {
        self.data = Some(data)
    }
}

#[derive(Debug)]
pub struct ValidError {
    items: Vec<(String, ValidRuleError)>,
}

impl ValidError {
    pub fn new(items: Vec<(String, ValidRuleError)>) -> Self {
        Self { items }
    }
    pub fn to_value(&self) -> Value {
        let mut map = serde_json::Map::new();
        for (k, v) in self.items.iter() {
            if let Some(data) = v.data.clone() {
                map.insert(k.to_owned(), data);
            }
        }
        Value::Object(map)
    }
}

impl IntoFluentMessage for ValidError {
    fn to_fluent_message(&self) -> FluentMessage {
        let mut data = vec![];
        for item in self.items.iter() {
            data.push(fluent_message!("valid-rule-item",{
                "name":&item.0,
                "error":item.1.message.to_owned(),
            }))
        }
        FluentMessage {
            id: "valid-error".to_string(),
            crate_name: env!("CARGO_PKG_NAME").to_string(),
            data: vec![("errors".to_string(), FluentData::MessageVec(data))],
        }
    }
}
