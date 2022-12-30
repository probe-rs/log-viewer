use std::fmt::Display;

use serde::{Deserialize, Serialize};
use yew::{html, Callback, Html, UseStateHandle};

use crate::{context_menu::ContextMenuItemProps, level_filter::LevelFilter, pill::Pill};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = format!("{:?}", self).to_ascii_uppercase();
        write!(f, "{}", text)
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
                        level_filter.set((*level_filter).clone().only_level(None, level));
                    })
                },
                title: format!("Only show {self}"),
            },
            ContextMenuItemProps {
                callback: Callback::from(move |_| {
                    level_filter.set((*level_filter).clone().remove_level(None, &level));
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
        }
    }
}
