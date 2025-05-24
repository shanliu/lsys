macro_rules! handler_not_found {
    (display $path:expr) => {
        Err(lsys_web::common::JsonError::JsonResponse(
            lsys_web::common::JsonData::default().set_sub_code("method-not-found").set_code(404),
            lsys_web::lsys_core::fluent_message!("method-not-found",{"path":$path})
        ))
    };
    ($path:expr) => {
        handler_not_found!(display format!("method not find : {}", $path))
    };
    () => {
        handler_not_found!(display format!("method not find"))
    };
}
