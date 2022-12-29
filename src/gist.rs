use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// A Gist as received by Github's v3 API.
#[derive(Serialize, Deserialize, Debug)]
pub struct Gist {
    #[serde(skip_serializing, skip_deserializing)]
    pub token: String,

    #[serde(skip_serializing, skip_deserializing)]
    pub api: String,

    public: bool,
    pub files: BTreeMap<String, GistFile>,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl Gist {
    pub fn current_file(&self) -> Option<String> {
        self.files.values().next().map(|v| v.content.clone())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GistFile {
    #[serde(skip_serializing, default = "GistFile::default_name")]
    pub name: String,
    #[serde(rename(serialize = "content"))]
    pub content: String,
}

impl GistFile {
    pub fn default_name() -> String {
        String::from("content")
    }
}
