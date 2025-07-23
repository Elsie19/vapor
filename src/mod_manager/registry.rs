use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use chrono_humanize::HumanTime;
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

/// Used for output for [`ModRegistry::status`].
#[derive(Serialize)]
struct ModStatus<'a> {
    name: &'a str,
    enabled: bool,
    version: &'a str,
    installed_at: Option<String>,
    missing_dependencies: Vec<String>,
    dependencies: Vec<String>,
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

    pub fn status(&self, json: bool) -> i32 {
        use inline_colorization::*;

        let mut ret = 0;
        let mut statuses = vec![];

        for (mod_name, contents) in &self.mods {
            let deps = self.satisfied_deps(mod_name);
            let missing_dependencies = deps.clone();
            let dependencies = contents
                .dependencies
                .clone()
                .unwrap_or_default()
                .into_iter()
                .filter(|dep| !missing_dependencies.contains(dep))
                .collect::<Vec<_>>();

            if !missing_dependencies.is_empty() {
                ret = 1;
            }

            if json {
                statuses.push(ModStatus {
                    name: mod_name,
                    enabled: contents.installed,
                    version: &contents.version,
                    installed_at: contents.installed_at.map(|dt| dt.to_rfc3339()),
                    missing_dependencies,
                    dependencies,
                });
            } else {
                println!(
                    "{style_bold}*{style_reset} {style_bold}{color_yellow}Name{style_reset}: `{mod_name}`"
                );
                println!(
                    "  - Enabled: {}",
                    if contents.installed {
                        format!("{color_green}true{style_reset}")
                    } else {
                        format!("{color_red}false{style_reset}")
                    }
                );
                println!("  - Version: {color_cyan}{}{style_reset}", contents.version);
                if let Some(installed_at) = contents.installed_at {
                    println!(
                        "  - Installed: {}",
                        HumanTime::from(installed_at - Utc::now())
                    );
                }
                if !deps.is_empty() {
                    println!("  - Missing dependencies:");
                    for dep in &deps {
                        println!("      > `{color_red}{dep}{style_reset}`");
                    }
                }
                if !dependencies.is_empty() {
                    println!("  - Dependencies:");
                    for dep in dependencies {
                        println!("      > `{dep}`");
                    }
                }
            }
        }

        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&statuses).expect("could not format json")
            );
        }

        ret
    }
}
