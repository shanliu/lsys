macro_rules! handler_not_found {
    (display $path:expr) => {
        Err(lsys_web::common::JsonError::JsonData(
            lsys_web::common::JsonData::default()
            .set_sub_code("method_not_found"),
            lsys_core::fluent_message!("method_not_found",{"msg":$path})
        ))
    };
    ($path:expr) => {
        handler_not_found!(display format!("method not find : {}", $path))
    };
    () => {
        handler_not_found!(display format!("method not find"))
    };
}
