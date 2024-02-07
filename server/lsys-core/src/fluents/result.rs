use crate::{fluent_message, FluentMessage, IntoFluentMessage};

#[derive(Debug)]
pub enum FluentBundleError {
    Io(std::io::Error),
    System(String),
}
impl IntoFluentMessage for FluentBundleError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            FluentBundleError::Io(err) => fluent_message!("fluent-file-error", err),
            FluentBundleError::System(err) => fluent_message!("fluent-error", err),
        }
    }
}

impl From<std::io::Error> for FluentBundleError {
    fn from(err: std::io::Error) -> Self {
        FluentBundleError::Io(err)
    }
}
