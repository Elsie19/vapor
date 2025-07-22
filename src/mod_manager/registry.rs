use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModRegistry {
    #[serde(default)]
    pub mods: HashMap<String, ModEntry>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ModEntry {
    pub version: String,
    pub file: String,
    pub installed: bool,
    pub installed_at: Option<DateTime<Utc>>,
    pub dependencies: Option<Vec<String>>,
    pub files: Vec<String>,
}

impl ModRegistry {
    /// Check if dependencies are satisfied.
    ///
    /// Returns a list of dependencies that could not be found.
    pub fn satisfied_deps<S: Into<String>>(&self, name: S) -> Vec<String> {
        let name = name.into();
        let mut broken_deps = vec![];

        let Some(mod_entry) = self.mods.get(&name) else {
            return broken_deps;
        };

        let Some(dependencies) = &mod_entry.dependencies else {
            return broken_deps;
        };

        for dep in dependencies {
            if !self.mods.contains_key(dep) {
                broken_deps.push(dep.to_owned());
            }
        }

        broken_deps
    }
}
