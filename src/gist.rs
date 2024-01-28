use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// A Gist as received by Github's v3 API.
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGist {
    pub public: bool,
    pub files: BTreeMap<String, CreateGistFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGistFile {
    pub content: String,
}

// A Gist as received by Github's v3 API.
#[derive(Serialize, Deserialize, Debug)]
pub struct Gist {
    #[serde(skip_serializing)]
    pub id: Option<String>,
    pub public: bool,
    pub files: BTreeMap<String, GistFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Gist {
    pub fn current_file(&self) -> Option<&GistFile> {
        self.files.values().next()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GistFile {
    pub filename: String,
    pub content: String,
    pub raw_url: String,
    pub truncated: bool,
}
