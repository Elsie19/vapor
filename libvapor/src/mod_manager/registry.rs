use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;
use std::io::Cursor;

use chrono::{DateTime, Utc};
use chrono_humanize::HumanTime;
use inline_colorization::*;
use ptree::{TreeBuilder, write_tree};
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

    #[allow(unused_must_use)]
    pub fn status(&self, json: bool) -> (String, i32) {
        use inline_colorization::*;

        let mut ret = 0;
        let mut out = String::new();
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
                writeln!(
                    &mut out,
                    "{style_bold}*{style_reset} {style_bold}{color_yellow}Name{style_reset}: `{mod_name}`"
                );
                writeln!(
                    &mut out,
                    "  - Enabled: {}",
                    if contents.installed {
                        format!("{color_green}true{style_reset}")
                    } else {
                        format!("{color_red}false{style_reset}")
                    }
                );
                writeln!(
                    &mut out,
                    "  - Version: {color_cyan}{}{style_reset}",
                    contents.version
                );
                if let Some(installed_at) = contents.installed_at {
                    writeln!(
                        &mut out,
                        "  - Installed: {}",
                        HumanTime::from(installed_at - Utc::now())
                    );
                }
                if !deps.is_empty() {
                    writeln!(&mut out, "  - Missing dependencies:");
                    for dep in &deps {
                        writeln!(&mut out, "      > `{color_red}{dep}{style_reset}`");
                    }
                }
                if !dependencies.is_empty() {
                    writeln!(&mut out, "  - Dependencies:");
                    for dep in dependencies {
                        writeln!(&mut out, "      > `{dep}`");
                    }
                }
            }
        }

        if json {
            (
                serde_json::to_string_pretty(&statuses).expect("could not format json"),
                ret,
            )
        } else {
            (out, ret)
        }
    }

    pub fn graph(&self) -> String {
        let mut out = String::new();
        for (mod_name, entry) in &self.mods {
            let mut seen = HashSet::new();
            let mut builder = TreeBuilder::new(format!(
                "* {style_bold}{mod_name}{style_reset} v{}",
                entry.version
            ));
            Self::build_tree(mod_name, &self.mods, &mut builder, &mut seen);
            let tree = builder.build();

            let mut buffer = Cursor::new(Vec::new());
            let _ = write_tree(&tree, &mut buffer);

            out.push_str(&String::from_utf8(buffer.into_inner()).unwrap());
            out.push('\n');
        }

        out
    }

    fn build_tree(
        mod_name: &str,
        map: &BTreeMap<String, ModEntry>,
        builder: &mut TreeBuilder,
        seen: &mut HashSet<String>,
    ) {
        if !seen.insert(mod_name.to_string()) {
            return;
        }

        if let Some(entry) = map.get(mod_name) {
            let deps = entry.dependencies.as_deref().unwrap_or(&[]);

            for dep in deps {
                if let Some(dep_entry) = map.get(dep) {
                    builder.begin_child(format!(
                        "{style_bold}{color_green}✔{style_reset} {style_bold}{dep}{style_reset} v{}",
                        dep_entry.version
                    ));
                    Self::build_tree(dep, map, builder, seen);
                    builder.end_child();
                } else {
                    builder
                        .begin_child(format!(
                            "{style_bold}{color_red}✘{style_reset} {style_bold}{dep}{style_reset}"
                        ))
                        .end_child();
                }
            }
        } else {
            builder
                .begin_child(format!("{style_bold}{color_red}✘{style_reset} {mod_name}"))
                .end_child();
        }
    }
}
