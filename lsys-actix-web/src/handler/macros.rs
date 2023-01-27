macro_rules! handler_not_found {
    (display $path:expr) => {
        Err(lsys_web::JsonData::message($path)
            .set_code(404)
            .set_sub_code("method_not_found"))
    };
    ($path:expr) => {
        handler_not_found!(display format!("method not find : {}", $path))
    };
    () => {
        handler_not_found!(display format!("method not find"))
    };
}
