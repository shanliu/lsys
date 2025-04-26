use crate::{FluentMessage, IntoFluentMessage};

#[derive(Debug)]
pub enum ValidParamErrorType {
    None,
    Number,
}

#[derive(Debug)]
pub struct ValidParamError {
    pub message: FluentMessage,
    pub error_type: ValidParamErrorType,
}
impl IntoFluentMessage for ValidParamError {
    fn to_fluent_message(&self) -> FluentMessage {
        self.message.to_owned()
    }
}
