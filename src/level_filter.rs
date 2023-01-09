use std::{cmp::Ordering, collections::HashMap};

use crate::proto::log_level::LogLevel;

#[derive(Debug, Clone, PartialEq)]
pub struct LevelFilter {
    matrix: HashMap<Option<String>, LogLevel>,
}

impl LevelFilter {
    pub fn show(&self, target: Option<String>, level: &LogLevel) -> bool {
        let target = self
            .matrix
            .iter()
            .filter_map(|(key, _)| match (&target, key) {
                (Some(target), Some(key)) => {
                    if target.starts_with(key) {
                        Some(Some(key.clone()))
                    } else {
                        None
                    }
                }
                (_, None) => Some(key.clone()),
                _ => None,
            })
            .max_by(|a, b| match (a, b) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(a), Some(b)) => a.cmp(b),
            })
            .expect("at least one filter element");
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
