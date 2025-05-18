#[macro_export]
macro_rules! status_format {
     (json $req_dao:expr,$enum:ident :: $variant:ident) => {
         serde_json::json!({
            "key":$enum::$variant as i8,
            "val":$req_dao.fluent.format_message(&($enum::$variant).fluent()),
        })
    };
    (json $req_dao:expr, $var:expr) => {
         serde_json::json!({
            "key":$var,
            "val":$req_dao.fluent.format_message(&lsys_core::FluentMessage {
                 id: format!("dict-{}",stringify!($var)),
                crate_name: env!("CARGO_PKG_NAME").to_string(),
                data: vec![],
            }),
        })
    };
    ($req_dao:expr,$enum:ident :: $variant:ident) => {
         $req_dao.fluent.format_message(&($enum::$variant).fluent())
    };
    ($req_dao:expr,$var:expr) => {
        $req_dao.fluent.format_message(&lsys_core::FluentMessage {
            id: format!("dict-{}",stringify!($var)),
            crate_name: env!("CARGO_PKG_NAME").to_string(),
            data: vec![],
        })
    };
}
