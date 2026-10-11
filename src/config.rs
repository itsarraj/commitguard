use std::fs;
use std::path::{Path, PathBuf};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub allowed_types: Option<Vec<String>>,
    pub max_subject_len: Option<usize>,
}

impl Config {
    pub fn load(repo_root: &Path) -> Self {
        let config_path = repo_root.join(".commitguard.toml");
        if let Ok(content) = fs::read_to_string(config_path) {
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}
