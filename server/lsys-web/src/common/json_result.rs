// 定义一些公共返回

use lsys_core::{FluentBundle, FluentMessage};

use super::{FluentFormat, JsonData, JsonResponse};

pub trait JsonFluent: FluentFormat + 'static {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData;
}
pub enum JsonError {
    Error(Box<dyn JsonFluent>),
    Message(FluentMessage),
    JsonResponse(JsonData, FluentMessage),
}

impl<T: JsonFluent> From<T> for JsonError {
    fn from(value: T) -> Self {
        Self::Error(Box::new(value))
    }
}
impl JsonError {
    pub fn to_json_response(&self, fluent: &FluentBundle) -> JsonResponse {
        match self {
            JsonError::Error(fluent_error_json_response) => {
                JsonResponse::data(fluent_error_json_response.to_json_data(fluent))
                    .set_message(fluent_error_json_response.fluent_format(fluent))
            }
            JsonError::Message(message) => {
                JsonResponse::data(JsonData::error()).set_message(fluent.format_message(message))
            }
            JsonError::JsonResponse(json_data, message) => {
                JsonResponse::data(json_data.to_owned()).set_message(fluent.format_message(message))
            }
        }
    }
}

pub type JsonResult<T> = Result<T, JsonError>;
