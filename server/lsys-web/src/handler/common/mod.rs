//所有接口公共部分

macro_rules! get_area {
    ($area:expr) => {
        match $area.as_ref() {
            Some(area) => area,
            None => {
                return Ok(JsonData::message("area function is disable"));
            }
        }
    };
}

pub mod app;
pub mod rbac;
pub mod sender;
pub mod user;
pub mod utils;
