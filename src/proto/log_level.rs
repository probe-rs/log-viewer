use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use yew::{classes, function_component, html, Html, Properties};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    #[default]
    None,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = format!("{:?}", self).to_ascii_uppercase();
        f.pad(&text)
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.num().cmp(&other.num()))
    }
}

impl FromStr for LogLevel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "none" => LogLevel::None,
            "error" => LogLevel::Error,
            "warn" => LogLevel::Warn,
            "info" => LogLevel::Info,
            "debug" => LogLevel::Debug,
            "trace" => LogLevel::Trace,
            variant => anyhow::bail!("No variant named {variant}"),
        })
    }
}

impl LogLevel {
    pub fn color(&self) -> &str {
        match self {
            LogLevel::Trace => "gray-500",
            LogLevel::Debug => "blue-500",
            LogLevel::Info => "green-500",
            LogLevel::Warn => "orange-500",
            LogLevel::Error => "red-500",
            LogLevel::None => "white",
        }
    }

    fn num(&self) -> usize {
        match self {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
            LogLevel::None => 5,
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct LogLevelLabelProps {
    pub level: LogLevel,
}

lazy_static::lazy_static! {
    /// This is an example for using doc comment attributes
    static ref CACHE: Vec<(String, String)> = [
        LogLevel::Trace,
    LogLevel::Debug,
    LogLevel::Info,
    LogLevel::Warn,
    LogLevel::Error,
    LogLevel::None].iter().map(|level|{
        let color = level.color().to_string();
        let label = format!("[{level}]");
        (format!(
            "{label: <7}",
        ), format!("bg-{}", color))
    }).collect();
}

#[function_component(LogLevelLabel)]
pub fn log_level_label(props: &LogLevelLabelProps) -> Html {
    let (label, color) = &CACHE[props.level.num()];

    html! {<span class={classes!["mr-1", "p-1", "rounded-md", color]}>{label}</span>}
}
