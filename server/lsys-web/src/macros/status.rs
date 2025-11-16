#[macro_export]
macro_rules! status_json_format {
     ($req_dao:expr,$enum:ident :: $variant:ident) => {
         serde_json::json!({
            "key":$enum::$variant as i8,
            "val":$req_dao.fluent.format_message(&($enum::$variant).fluent()),
        })
    };
}

#[macro_export]
macro_rules! const_json_format {
    ($req_dao:expr, $var:expr) => {
         serde_json::json!({
            "key":$var,
            "val":$req_dao.fluent.format_message(&lsys_core::FluentMessage {
                 id: format!("dict-{}",stringify!($var)),
                crate_name: env!("CARGO_PKG_NAME").to_string(),
                data: vec![],
            }),
        })
    };
}
