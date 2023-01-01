use std::collections::HashMap;

use crate::proto::log_level::LogLevel;

#[derive(Debug, Clone, PartialEq)]
pub struct LevelFilter {
    matrix: HashMap<Option<String>, LogLevel>,
}

impl LevelFilter {
    pub fn show(&self, target: Option<String>, level: &LogLevel) -> bool {
        let filter = self.matrix.get(&target).or_else(|| self.matrix.get(&None));

        if let Some(filter) = filter {
            level >= filter
        } else {
            false
        }
    }

    pub fn new(matrix: HashMap<Option<String>, LogLevel>) -> Self {
        Self { matrix }
    }

    pub fn matrix(&self) -> &HashMap<Option<String>, LogLevel> {
        &self.matrix
    }

    pub fn set_level(mut self, target: Option<String>, level: LogLevel) -> Self {
        let filter = self.matrix.entry(target).or_default();
        *filter = level;
        Self {
            matrix: self.matrix,
        }
    }

    pub fn remove(mut self, target: &Option<String>) -> Self {
        self.matrix.remove(target);
        Self {
            matrix: self.matrix,
        }
    }
}
