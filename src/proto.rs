pub mod log_level;

use serde::{Deserialize, Serialize};

use self::log_level::LogLevel;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fields {
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub fields: Fields,
    pub level: LogLevel,
    pub span: Option<Span>,
    pub spans: Option<Vec<Span>>,
    pub target: String,
}
