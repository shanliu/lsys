#[derive(Debug, Clone)]
pub enum FluentData {
    Message(FluentMessage),
    String(String),
}
impl From<FluentMessage> for FluentData {
    fn from(value: FluentMessage) -> Self {
        FluentData::Message(value)
    }
}
impl<T: ToString> From<T> for FluentData {
    fn from(value: T) -> Self {
        FluentData::String(value.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct FluentMessage {
    pub id: String,
    pub crate_name: String,
    pub data: Vec<(String, FluentData)>,
}

impl FluentMessage {
    pub fn default_format(&self) -> String {
        if self.data.is_empty() {
            self.id.to_owned()
        } else {
            let data = self
                .data
                .iter()
                .map(|e| {
                    format!(
                        "{}:'{}'",
                        e.0,
                        match &e.1 {
                            FluentData::Message(e1) => e1.default_format(),
                            FluentData::String(e1) => e1.to_owned(),
                        }
                    )
                })
                .collect::<Vec<String>>();
            format!("{}:{{{}}}", self.id, data.join(","))
        }
    }
}

pub trait IntoFluentMessage {
    fn to_fluent_message(&self) -> FluentMessage;
}

impl IntoFluentMessage for FluentMessage {
    fn to_fluent_message(&self) -> FluentMessage {
        self.to_owned()
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
                    $( ($msg_key.to_owned(),$crate::FluentData::from($msg_val)) ),*
                ]
            }
        }
    };
    ($key:literal,$msg_val:expr) => {
        {
            $crate::FluentMessage {
                id: $key.to_owned(),
                crate_name:env!("CARGO_PKG_NAME").to_string(),
                data:vec![("msg".to_owned(),$crate::FluentData::from($msg_val))]
            }
        }
    };
}

// impl From<String> for FluentMessage {
//     fn from(value: String) -> Self {
//         fluent_message!("app-error", value)
//     }
// }
// impl From<&String> for FluentMessage {
//     fn from(value: &String) -> Self {
//         fluent_message!("app-error", value)
//     }
// }
// impl From<&str> for FluentMessage {
//     fn from(value: &str) -> Self {
//         fluent_message!("app-error", value)
//     }
// }
