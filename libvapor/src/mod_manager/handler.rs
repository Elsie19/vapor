use std::{
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::Write,
    ops::Not,
    path::{Component, Path, PathBuf},
};

use chrono::Utc;
use miette::{Diagnostic, NamedSource};
use thiserror::Error;
use zip::ZipArchive;

use super::{
    mod_file_formats::read_files,
    registry::{ModEntry, ModRegistry},
};

const VALID_ROOT_DIRS: &[&str] = &["r6", "archive", "bin", "red4ext", "engine"];

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Move {
    Enable,
    Disable,
}

impl Not for Move {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Move::Enable => Move::Disable,
            Move::Disable => Move::Enable,
        }
    }
}

pub enum Operation {
    /// Version.
    Added(String),
    Updated {
        old: String,
        new: String,
    },
    Move(Move),
}

impl Move {
    pub const fn installed(self) -> bool {
        matches!(self, Self::Enable)
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum ModError {
    #[error(transparent)]
    #[diagnostic(code(ModHandler::add_mod))]
    Io(#[from] std::io::Error),
    #[error("Deserialization error: `{0}`")]
    De(#[from] toml::de::Error),
    #[error("Serialization error: `{0}`")]
    Ser(#[from] toml::ser::Error),
    #[error("Missing mod: `{0}`")]
    MissingMod(String),
    #[error("Decompression issue: `{0}`")]
    ZipArchive(#[from] zip::result::ZipError),
    #[error("Files from `{incoming}` already exist in mod directory")]
    #[diagnostic(help("Ensure that mods are not trying to overwrite others."))]
    DoubleOwnedFiles {
        incoming: String,
        #[source_code]
        files: NamedSource<String>,
        raw_splits: Vec<(String, String)>,
        #[label = "Files(s) listed here are already owned by another mod"]
        span: std::ops::Range<usize>,
    },
    #[error("Extraction incomplete")]
    #[diagnostic(code(ModHandler::add_mod))]
    ExtractionIncomplete {
        #[source_code]
        files: NamedSource<String>,
        raw_splits: Vec<PathBuf>,
        #[label = "Files(s) listed here are could not be found after extraction"]
        span: std::ops::Range<usize>,
    },
    #[error("Missing file in dry-run: `{mod_name}` does not have `{path}`")]
    #[diagnostic(code(ModHandler::add_mod))]
    MissingFile { mod_name: String, path: String },
}

pub struct ModHandler {
    pub root: PathBuf,
    pub toml: PathBuf,
}

impl ModHandler {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root: root.clone(),
            toml: root.join("mods.toml"),
        }
    }

    fn term_link(&self, file: &str) -> String {
        let full_path = self.root.join(file);
        let path_str = full_path.to_string_lossy();
        let url = format!("file://{path_str}");
        format!("\x1b]8;;{url}\x1b\\{file}\x1b]8;;\x1b\\")
    }

    pub fn add_mod<S: Into<String>>(
        &self,
        path: &Path,
        name: S,
        version: S,
        dependencies: &[String],
    ) -> Result<Operation, ModError> {
        let name = name.into();
        let version = version.into();

        let mut toml = self.load_toml()?;

        let mut archive = ZipArchive::new(File::open(path)?).expect("Could not read zip file");

        let files = read_files(path);

        let crossed_paths = toml.crossover_paths(&name, files.clone());
        if !crossed_paths.is_empty() {
            let text = crossed_paths
                .iter()
                .map(|(owned, file)| format!("{owned} | {}", self.term_link(file)))
                .collect::<Vec<_>>()
                .join("\n");
            let span = 0..text.len();
            return Err(ModError::DoubleOwnedFiles {
                raw_splits: crossed_paths,
                incoming: name,
                files: NamedSource::new("conflicting files", text),
                span,
            });
        }

        archive.extract(self.root.clone())?;

        let extracted_files = files.iter().map(|f| self.root.join(f)).collect::<Vec<_>>();

        let missing: Vec<_> = extracted_files.iter().filter(|p| !p.exists()).collect();

        if !missing.is_empty() {
            let text = missing
                .iter()
                .map(|file| self.term_link(file.to_str().unwrap()))
                .collect::<Vec<_>>()
                .join("\n");
            let span = 0..text.len();
            return Err(ModError::ExtractionIncomplete {
                raw_splits: missing.into_iter().cloned().collect(),
                files: NamedSource::new("missing files", text),
                span,
            });
        }

        let old_version = toml.mods.get(&name).map(|entry| entry.version.clone());

        toml.mods.insert(
            name,
            ModEntry {
                version: version.clone(),
                file: path.to_string_lossy().to_string(),
                installed: true,
                installed_at: Some(Utc::now()),
                dependencies: if dependencies.is_empty() {
                    None
                } else {
                    Some(dependencies.to_vec())
                },
                files: read_files(path),
            },
        );

        let mut mods = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.toml)?;

        write!(&mut mods, "{}", toml::to_string_pretty(&toml)?)?;

        if let Some(old_version) = old_version {
            if old_version != version {
                return Ok(Operation::Updated {
                    old: old_version,
                    new: version,
                });
            }
        }

        Ok(Operation::Added(version))
    }

    pub fn move_mod<S: Into<String>>(
        &self,
        name: S,
        move_where: Move,
    ) -> Result<Operation, ModError> {
        let name = name.into();
        let mut toml = self.load_toml()?;

        let Some(entry) = toml.mods.get_mut(&name) else {
            return Err(ModError::MissingMod(name));
        };

        let installed = move_where.installed();

        if entry.installed == installed {
            return Err(ModError::MissingMod(name));
        }

        let old_root = match move_where {
            Move::Enable => self.root.join("Disabled Mods"),
            Move::Disable => self.root.clone(),
        };

        let new_root = match move_where {
            Move::Enable => self.root.clone(),
            Move::Disable => self.root.join("Disabled Mods"),
        };

        for file in &entry.files {
            let from = old_root.join(file);
            if !from.exists() {
                return Err(ModError::MissingFile {
                    mod_name: name,
                    path: file.to_owned(),
                });
            }

            let to = new_root.join(file);

            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::rename(&from, &to)?;

            if let Some(parent) = from.parent() {
                Self::clean_upwards(parent, &old_root);
            }
        }

        entry.installed = installed;
        entry.installed_at = if installed { Some(Utc::now()) } else { None };

        let mut mods = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.toml)?;

        write!(&mut mods, "{}", toml::to_string_pretty(&toml)?)?;

        Ok(Operation::Move(!move_where))
    }

    pub fn load_toml(&self) -> Result<ModRegistry, ModError> {
        let toml_string = fs::read_to_string(&self.toml)?;

        Ok(toml::from_str(&toml_string)?)
    }

    fn clean_upwards(mut path: &Path, stop: &Path) {
        while path.starts_with(stop) && path != stop {
            if let Some(name) = path.file_name() {
                if VALID_ROOT_DIRS.contains(&name.to_str().unwrap()) {
                    break;
                }
            }

            match fs::remove_dir(path) {
                Ok(()) => {}
                Err(_) => break,
            }

            path = path.parent().unwrap();
        }
    }

    fn root_dir_common_filter(path: &Path) -> bool {
        if let Some(first) = path.components().next()
            && let Component::Normal(name) = first
        {
            return VALID_ROOT_DIRS
                .iter()
                .any(|&valid| OsStr::new(valid) == name);
        }

        false
    }
}
