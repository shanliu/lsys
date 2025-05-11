use serde_json::{json, Value};

use crate::{FluentData, FluentMessage, IntoFluentMessage};

use super::ValidRuleKey;

#[derive(Debug, Clone)]
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
    items: Vec<(ValidRuleKey, ValidRuleError)>,
}

impl ValidError {
    pub fn message(key: ValidRuleKey, msg: FluentMessage) -> Self {
        Self {
            items: vec![(key, ValidRuleError::new(msg))],
        }
    }
    pub fn new(items: Vec<(ValidRuleKey, ValidRuleError)>) -> Self {
        Self { items }
    }
    pub fn to_value(&self) -> Value {
        let mut map_data = serde_json::Map::new();
        for (k, v) in self.items.iter() {
            if let Some(data) = v.data.clone() {
                match map_data.get_mut(&k.name) {
                    Some(existing) => {
                        if let Value::Array(arr) = existing {
                            arr.push(data);
                        } else {
                            let current = existing.clone();
                            *existing = Value::Array(vec![current, data]);
                        }
                    }
                    None => {
                        map_data.insert(k.name.to_owned(), Value::Array(vec![data]));
                    }
                }
            }
        }
        let mut out_data = serde_json::Map::from_iter([(
            "field".to_string(),
            json!(self
                .items
                .iter()
                .map(|e| e.0.name.as_str())
                .collect::<Vec<&str>>()),
        )]);
        if !map_data.is_empty() {
            out_data.insert("info".to_string(), Value::Object(map_data));
        }
        Value::Object(out_data)
    }
}

impl IntoFluentMessage for ValidError {
    fn to_fluent_message(&self) -> FluentMessage {
        let mut data = vec![];
        for item in self.items.iter() {
            data.push(FluentMessage {
                id: "valid-rule-item".to_string(),
                crate_name: env!("CARGO_PKG_NAME").to_string(),
                data: vec![
                    (
                        "name".to_string(),
                        FluentData::Message(FluentMessage {
                            id: format!("valid-rule-name-{}", item.0.name),
                            crate_name: item.0.crate_name.to_string(),
                            data: vec![],
                        }),
                    ),
                    (
                        "error".to_string(),
                        FluentData::Message(item.1.message.to_owned()),
                    ),
                ],
            })
        }
        FluentMessage {
            id: "valid-error".to_string(),
            crate_name: env!("CARGO_PKG_NAME").to_string(),
            data: vec![("errors".to_string(), FluentData::MessageVec(data))],
        }
    }
}
