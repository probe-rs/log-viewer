use std::collections::{HashMap, HashSet};

use crate::proto::log_level::LogLevel;

#[derive(Debug, Clone, PartialEq)]
pub struct LevelFilter {
    matrix: HashMap<String, HashSet<LogLevel>>,
}

impl LevelFilter {
    pub fn show(&self, target: &str, level: &LogLevel) -> bool {
        let selection = if self.matrix.len() == 1 {
            self.matrix.get("")
        } else {
            self.matrix.get(target)
        };

        if let Some(selection) = selection {
            selection.contains(level)
        } else {
            false
        }
    }

    pub fn new(matrix: HashMap<String, HashSet<LogLevel>>) -> Self {
        Self { matrix }
    }

    pub fn add_level(mut self, target: Option<&str>, level: LogLevel) -> Self {
        if let Some(target) = target {
            let target = self.matrix.entry(target.to_string()).or_default();
            target.insert(level);
        } else {
            for target in self.matrix.values_mut() {
                target.insert(level);
            }
        }
        Self {
            matrix: self.matrix,
        }
    }

    pub fn remove_level(mut self, target: Option<&str>, level: &LogLevel) -> Self {
        if let Some(target) = target {
            let target = self.matrix.entry(target.to_string()).or_default();
            target.remove(level);
        } else {
            for target in self.matrix.values_mut() {
                target.remove(level);
            }
        }
        Self {
            matrix: self.matrix,
        }
    }

    pub fn only_level(mut self, target: Option<&str>, level: LogLevel) -> Self {
        let make_target = || {
            let mut filter = HashSet::new();
            filter.insert(level);
            filter
        };

        if let Some(target) = target {
            let target = self.matrix.entry(target.to_string()).or_default();
            *target = make_target();
        } else {
            for target in self.matrix.values_mut() {
                *target = make_target();
            }
        }
        Self {
            matrix: self.matrix,
        }
    }
}
