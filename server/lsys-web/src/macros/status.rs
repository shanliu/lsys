// #[macro_export]
// macro_rules! status_fluent {
//     ($enum:ident :: $variant:ident) => {
//         lsys_core::FluentMessage {
//             id: concat!("status-", stringify!($enum), "-", stringify!($variant)).to_string(),
//             crate_name: env!("CARGO_PKG_NAME").to_string(),
//             data: vec![],
//         }
//     };
// }

#[macro_export]
macro_rules! status_json {
    ($req_dao:expr,$enum:ident :: $variant:ident) => {
        serde_json::json!({
                "key":$enum::$variant as i8,
                "val":$req_dao.fluent.format_message(&lsys_core::FluentMessage {
                    id: concat!("status-", stringify!($enum), "-", stringify!($variant)).to_string(),
                    crate_name: env!("CARGO_PKG_NAME").to_string(),
                    data: vec![],
                }),
            })
    };
}
