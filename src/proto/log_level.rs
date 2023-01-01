use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use yew::{html, Callback, Html, UseStateHandle};

use crate::{context_menu::ContextMenuItemProps, level_filter::LevelFilter, pill::Pill};

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
        write!(f, "{}", text)
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        fn num(level: &LogLevel) -> usize {
            match level {
                LogLevel::Trace => 0,
                LogLevel::Debug => 1,
                LogLevel::Info => 2,
                LogLevel::Warn => 3,
                LogLevel::Error => 4,
                LogLevel::None => 5,
            }
        }

        Some(num(self).cmp(&num(other)))
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
    pub fn draw(&self, level_filter: UseStateHandle<LevelFilter>) -> Html {
        let label = format!("[{self}]");
        let pad = 7 - label.len();
        let color = self.color().to_string();
        let level = *self;

        let label = format!(
            "{label}{}",
            std::iter::repeat(" ")
                .take(pad)
                .fold(String::new(), |a, b| a + b)
        );

        let context_menu = vec![
            ContextMenuItemProps {
                callback: {
                    let level_filter = level_filter.clone();
                    Callback::from(move |_| {
                        level_filter.set((*level_filter).clone().set_level(None, level));
                    })
                },
                title: format!("Only show {self}"),
            },
            ContextMenuItemProps {
                callback: Callback::from(move |_| {
                    level_filter.set((*level_filter).clone().set_level(None, level));
                }),
                title: format!("Don't show {self}"),
            },
        ];

        html! {<Pill
                {color}
                {context_menu}
            >
                {label}
        </Pill>}
    }

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
}
