use serde::Serialize;
use serde_json::{Value, json, to_value};

use super::page_data::{PageCursorValue, PageTotalRowValue};

/// 分页总数的 JSON 值
pub enum JsonPageTotal {
    None,
    SimpleU64(u64),
    SimpleI64(i64),
    SimpleUsize(usize),
    Complex(PageTotalRowValue),
}

impl From<i64> for JsonPageTotal {
    fn from(value: i64) -> Self {
        JsonPageTotal::SimpleI64(value)
    }
}

impl From<Option<i64>> for JsonPageTotal {
    fn from(value: Option<i64>) -> Self {
        match value {
            Some(v) => JsonPageTotal::SimpleI64(v),
            None => JsonPageTotal::None,
        }
    }
}

impl From<u64> for JsonPageTotal {
    fn from(value: u64) -> Self {
        JsonPageTotal::SimpleU64(value)
    }
}

impl From<Option<u64>> for JsonPageTotal {
    fn from(value: Option<u64>) -> Self {
        match value {
            Some(v) => JsonPageTotal::SimpleU64(v),
            None => JsonPageTotal::None,
        }
    }
}

impl From<PageTotalRowValue> for JsonPageTotal {
    fn from(value: PageTotalRowValue) -> Self {
        JsonPageTotal::Complex(value)
    }
}

impl From<Option<PageTotalRowValue>> for JsonPageTotal {
    fn from(value: Option<PageTotalRowValue>) -> Self {
        match value {
            Some(v) => JsonPageTotal::Complex(v),
            None => JsonPageTotal::None,
        }
    }
}

impl From<usize> for JsonPageTotal {
    fn from(value: usize) -> Self {
        JsonPageTotal::SimpleUsize(value)
    }
}

impl From<Option<usize>> for JsonPageTotal {
    fn from(value: Option<usize>) -> Self {
        match value {
            Some(v) => JsonPageTotal::SimpleUsize(v),
            None => JsonPageTotal::None,
        }
    }
}

impl Serialize for JsonPageTotal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            JsonPageTotal::None => serializer.serialize_none(),
            JsonPageTotal::SimpleU64(v) => v.serialize(serializer),
            JsonPageTotal::SimpleI64(v) => v.serialize(serializer),
            JsonPageTotal::SimpleUsize(v) => v.serialize(serializer),
            JsonPageTotal::Complex(v) => v.serialize(serializer),
        }
    }
}

/// 用于将不同类型转换为 JSON Body 的 trait
pub trait JsonIntoBody: Serialize + Sized {
    fn into_body(self) -> Value {
        to_value(self).unwrap_or(Value::Null)
    }
}

impl<T: Serialize> JsonIntoBody for T {}

/// 分页数据的 JSON 响应对象。
///
/// - `data`：数据内容
/// - `total`：总数（可选，支持 i64、Option<i64>、PageTotalRowValue、Option<PageTotalRowValue>）
/// - `cursor`：游标分页信息（可选）
pub struct JsonPageData {
    pub data: Value,
    pub total: JsonPageTotal,
    pub cursor: Option<PageCursorValue>,
    pub value: Option<serde_json::Map<String, Value>>,
}

impl JsonPageData {
    /// 创建带 total 的分页数据
    pub fn total<T: Into<JsonPageTotal>, D: Serialize>(data: D, total: T) -> Self {
        Self {
            data: to_value(data).unwrap_or(Value::Null),
            total: total.into(),
            cursor: None,
            value: None,
        }
    }

    /// 创建带 cursor 和 total 的分页数据
    pub fn cursor<T: Into<JsonPageTotal>, D: Serialize>(
        data: D,
        cursor: PageCursorValue,
        total: T,
    ) -> Self {
        Self {
            data: to_value(data).unwrap_or(Value::Null),
            total: total.into(),
            cursor: Some(cursor),
            value: None,
        }
    }

    /// 设置额外的键值对
    pub fn set_extra<K: Into<String>, V: Serialize>(mut self, key: K, value: V) -> Self {
        if self.value.is_none() {
            self.value = Some(serde_json::Map::new());
        }
        if let Some(map) = &mut self.value {
            map.insert(key.into(), to_value(value).unwrap_or(Value::Null));
        }
        self
    }
}

impl Serialize for JsonPageData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = self.value.clone().unwrap_or_default();
        // 只有当 data 不为 Null 时才插入 data 字段
        if !self.data.is_null() {
            map.insert("data".to_string(), self.data.clone());
        }
        if let Some(cursor) = &self.cursor {
            map.insert("cursor".to_string(), json!(cursor));
        }
        if let Some(total) = match &self.total {
            JsonPageTotal::None => None,
            JsonPageTotal::SimpleU64(v) => Some(json!(v)),
            JsonPageTotal::SimpleI64(v) => Some(json!(v)),
            JsonPageTotal::SimpleUsize(v) => Some(json!(v)),
            JsonPageTotal::Complex(v) => Some(json!(v)),
        } {
            map.insert("total".to_string(), total);
        }
        Value::Object(map).serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize)]
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
    pub fn body<T: JsonIntoBody>(value: T) -> Self {
        JsonData::default().set_body(value)
    }
    pub fn set_body<T: JsonIntoBody>(mut self, value: T) -> Self {
        self.body = Some(value.into_body());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_data_body_formats() {
        // 1. 字符串
        let data = JsonData::body("hello world");
        println!(
            "String body: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 2. 数字
        let data = JsonData::body(123);
        println!(
            "Number body: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        let data = JsonData::body(456.78);
        println!(
            "Float body: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 3. 布尔值
        let data = JsonData::body(true);
        println!(
            "Boolean body: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 4. 数组
        let data = JsonData::body(vec![1, 2, 3]);
        println!(
            "Array body: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        let data = JsonData::body(vec!["a", "b", "c"]);
        println!(
            "String array body: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 5. 对象（使用 json! 宏）
        let data = JsonData::body(json!({"name": "test", "value": 123}));
        println!(
            "Object body (json!): {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 6. Option 类型
        let data = JsonData::body(Some(123));
        println!(
            "Option Some body: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        let data = JsonData::body(Option::<i32>::None);
        println!(
            "Option None body: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 7. JsonValue 直接
        let data = JsonData::body(Value::String("direct value".to_string()));
        println!(
            "JsonValue body: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        let data = JsonData::body(Value::Null);
        println!(
            "Null body: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 8. JsonPageData with total (u64)
        let page_data = JsonPageData::total(vec![1, 2, 3], 100u64);
        let data = JsonData::body(page_data);
        println!(
            "JsonPageData with total (u64): {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 9. JsonPageData with total (i64)
        let page_data = JsonPageData::total(vec!["a", "b"], 50i64);
        let data = JsonData::body(page_data);
        println!(
            "JsonPageData with total (i64): {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 10. JsonPageData with total (None)
        let page_data = JsonPageData::total(vec![1, 2, 3], JsonPageTotal::None);
        let data = JsonData::body(page_data);
        println!(
            "JsonPageData with total (None): {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 11. JsonPageData with initial value
        let page_data =
            JsonPageData::total(vec![1, 2, 3], 100u64).set_extra("extra_key", "extra_value");
        let data = JsonData::body(page_data);
        println!(
            "JsonPageData with initial value: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 12. JsonPageData with total (PageTotalRowValue with exact only)
        let total_row = PageTotalRowValue {
            exact: Some(100),
            over: None,
        };
        let page_data = JsonPageData::total(vec![1, 2, 3], total_row);
        let data = JsonData::body(page_data);
        println!(
            "JsonPageData with total (PageTotalRowValue exact only): {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 13. JsonPageData with total (PageTotalRowValue with over only)
        let total_row = PageTotalRowValue {
            exact: None,
            over: Some(200),
        };
        let page_data = JsonPageData::total(vec!["a", "b"], total_row);
        let data = JsonData::body(page_data);
        println!(
            "JsonPageData with total (PageTotalRowValue over only): {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 14. JsonPageData with total (PageTotalRowValue with both exact and over)
        let total_row = PageTotalRowValue {
            exact: Some(100),
            over: Some(50),
        };
        let page_data = JsonPageData::total(vec![1, 2, 3, 4], total_row);
        let data = JsonData::body(page_data);
        println!(
            "JsonPageData with total (PageTotalRowValue both): {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 15. 默认值（无 body）
        let data = JsonData::default();
        println!(
            "Default (no body): {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 13. 错误响应
        let data = JsonData::error();
        println!(
            "Error response: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );

        // 14. 链式调用设置多个字段
        let data = JsonData::body("test")
            .set_code(404)
            .set_sub_code("not_found");
        println!(
            "Chained calls: {}",
            serde_json::to_string_pretty(&data).unwrap()
        );
    }
}
