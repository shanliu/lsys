use crate::{fluent_message, FluentMessage, IntoFluentMessage};

//config
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Config(config::ConfigError),
}

impl IntoFluentMessage for ConfigError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            ConfigError::Io(err) => fluent_message!("file-error", err),
            ConfigError::Config(err) => fluent_message!("config-error", err),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Io(err)
    }
}
impl From<config::ConfigError> for ConfigError {
    fn from(err: config::ConfigError) -> Self {
        ConfigError::Config(err)
    }
}
