#[derive(Debug)]
pub struct FluentMessage {
    pub id: String,
    pub crate_name: String,
    pub data: Vec<(String, String)>,
}

impl std::fmt::Display for FluentMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.data.is_empty() {
            write!(f, "{}", self.id)
        } else {
            write!(f, "{}:{}", self.id, serde_json::json!(self.data))
        }
    }
}

#[macro_export]
macro_rules! fluent_message {
    ($key:literal) => {
        {
            $crate::FluentMessage {
                id: $key.to_owned(),
                crate_name:env!("CARGO_PKG_NAME").to_string(),
                data:vec![]
            }
        }
    };
    ($key:literal,{$($msg_key:literal:$msg_val:expr),+$(,)*}) => {
        {
            $crate::FluentMessage {
                id: $key.to_owned(),
                crate_name:env!("CARGO_PKG_NAME").to_string(),
                data:vec![
                    $( ($msg_key.to_owned(), $msg_val.to_string()) ),*
                ]
            }
        }
    };
    ($key:literal,$msg_val:expr) => {
        {
            $crate::FluentMessage {
                id: $key.to_owned(),
                crate_name:env!("CARGO_PKG_NAME").to_string(),
                data:vec![("msg".to_owned(),$msg_val.to_string())]
            }
        }
    };
}

impl From<String> for FluentMessage {
    fn from(value: String) -> Self {
        fluent_message!("app-error", value)
    }
}
impl From<&String> for FluentMessage {
    fn from(value: &String) -> Self {
        fluent_message!("app-error", value)
    }
}
impl From<&str> for FluentMessage {
    fn from(value: &str) -> Self {
        fluent_message!("app-error", value)
    }
}
