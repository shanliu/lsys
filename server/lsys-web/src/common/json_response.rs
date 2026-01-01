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
            out_data.insert("response".to_string(), Self::convert_value(body.to_owned()));
        }
        Value::Object(out_data)
    }
    fn convert_value(value: Value) -> Value {
        match value {
            // 数字转为字符串
            Value::Number(n) => Value::String(n.to_string()),
            // 布尔值转为 "1" 或 "0"
            Value::Bool(b) => Value::String(if b { "1".to_string() } else { "0".to_string() }),
            // 处理嵌套对象
            Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    new_map.insert(k, Self::convert_value(v));
                }
                Value::Object(new_map)
            }
            // 处理嵌套数组
            Value::Array(vec) => {
                let new_vec: Vec<Value> = vec.into_iter().map(Self::convert_value).collect();
                Value::Array(new_vec)
            }
            // 其他类型（如字符串、null）保持原样
            _ => value,
        }
    }
}
