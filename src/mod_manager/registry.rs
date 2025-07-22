use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModRegistry {
    #[serde(default)]
    pub mods: BTreeMap<String, ModEntry>,
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

    /// Check if paths are owned by another mod already.
    ///
    /// Returns a [`Vec`] with the tuple `(owned_mod_name, path)`.
    pub fn crossover_paths<I, T, S>(&self, mod_name: S, paths: I) -> Vec<(String, String)>
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
        S: AsRef<str>,
    {
        let mod_name = mod_name.as_ref();
        let mut overlaps = vec![];
        let incoming = paths.into_iter().map(Into::into).collect::<Vec<_>>();

        for path in incoming {
            for (name, mod_entry) in &self.mods {
                if mod_entry.files.iter().any(|f| f == &path) && *name != mod_name {
                    overlaps.push((name.to_owned(), path.clone()));
                }
            }
        }

        overlaps
    }
}
