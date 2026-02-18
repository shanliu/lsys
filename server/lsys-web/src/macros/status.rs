#[macro_export]
macro_rules! status_json_format {
     ($req_dao:expr,$enum:ident :: $variant:ident) => {
         serde_json::json!({
            "key":$enum::$variant as i8,
            "val":$req_dao.fluent.format_message(&($enum::$variant).fluent()),
        })
    };

     // 第三个参数为 `{ ... }`，会被原样拼进 `json!({ ... })` 里（用于追加字段）
     ($req_dao:expr,$enum:ident :: $variant:ident, { $($extra:tt)* }) => {
         serde_json::json!({
            "key":$enum::$variant as i8,
            "val":$req_dao.fluent.format_message(&($enum::$variant).fluent()),
            $($extra)*
        })
    };
}

#[macro_export]
macro_rules! const_json_format {
    ($req_dao:expr, $var:expr) => {
         serde_json::json!({
            "key":$var,
            "val":$req_dao.fluent.format_message(&$crate::lsys_core::FluentMessage {
                 id: format!("const-{}",stringify!($var)),
                crate_name: env!("CARGO_PKG_NAME").to_string(),
                data: vec![],
            }),
        })
    };

    // 第三个参数为 `{ ... }`，会被原样拼进 `json!({ ... })` 里（用于追加字段）
    ($req_dao:expr, $var:expr, { $($extra:tt)* }) => {
         serde_json::json!({
            "key":$var,
            "val":$req_dao.fluent.format_message(&$crate::lsys_core::FluentMessage {
                 id: format!("const-{}",stringify!($var)),
                crate_name: env!("CARGO_PKG_NAME").to_string(),
                data: vec![],
            }),
            $($extra)*
        })
    };
}

#[macro_export]
macro_rules! var_json_format {
    // 传入表达式时（如变量），使用表达式的值
    ($req_dao:expr, $var:expr) => {
         serde_json::json!({
            "key": $var,
            "val": $req_dao.fluent.format_message(&lsys_core::FluentMessage {
                 id: format!("var-{}",$var),
                crate_name: env!("CARGO_PKG_NAME").to_string(),
                data: vec![],
            }),
        })
    };

    // 第三个参数为 `{ ... }`，会被原样拼进 `json!({ ... })` 里（用于追加字段）
    ($req_dao:expr, $var:expr, { $($extra:tt)* }) => {
         serde_json::json!({
            "key": $var,
            "val": $req_dao.fluent.format_message(&lsys_core::FluentMessage {
                 id: format!("var-{}",$var),
                crate_name: env!("CARGO_PKG_NAME").to_string(),
                data: vec![],
            }),
            $($extra)*
        })
    };
}
