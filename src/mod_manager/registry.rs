use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModRegistry {
    #[serde(default)]
    pub mods: HashMap<String, ModEntry>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ModEntry {
    pub version: String,
    pub file: String,
    pub installed: bool,
    pub installed_at: Option<DateTime<Utc>>,
    pub dependencies: Option<Vec<String>>,
    pub files: Vec<String>,
}
