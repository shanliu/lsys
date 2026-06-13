use std::fmt;
use std::str::FromStr;

/// 运行环境
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppEnv {
    #[default]
    Production,
    Development,
    Test,
    Staging,
}

impl FromStr for AppEnv {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dev" | "development" => Ok(Self::Development),
            "test" | "testing" => Ok(Self::Test),
            "staging" | "stage" => Ok(Self::Staging),
            "prod" | "production" => Ok(Self::Production),
            _ => Err(format!("Unknown environment: {}", s)),
        }
    }
}

impl fmt::Display for AppEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Development => write!(f, "development"),
            Self::Test => write!(f, "test"),
            Self::Staging => write!(f, "staging"),
            Self::Production => write!(f, "production"),
        }
    }
}
