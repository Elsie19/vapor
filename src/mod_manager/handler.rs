use std::{
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
};

use chrono::Utc;
use thiserror::Error;
use zip::ZipArchive;

use super::{
    mod_file_formats::read_files,
    registry::{ModEntry, ModRegistry},
};

#[derive(PartialEq, Eq)]
pub enum Move {
    Enable,
    Disable,
}

impl Move {
    pub const fn installed(&self) -> bool {
        matches!(self, Self::Enable)
    }
}

#[derive(Error, Debug)]
pub enum ModError {
    #[error("io error: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("deserialization error: `{0}`")]
    De(#[from] toml::de::Error),
    #[error("serialization error: `{0}`")]
    Ser(#[from] toml::ser::Error),
    #[error("missing mod: `{0}`")]
    MissingMod(String),
    #[error("decompression issue: `{0}`")]
    ZipArchive(#[from] zip::result::ZipError),
    #[error(
        "files from `{incoming}` are attempting to write to:\n{file_listing}", file_listing = .files.iter().map(|(owned, file)| format!("{owned} | {file}")).collect::<Vec<_>>().join("\n")
    )]
    DoubleOwnedFiles {
        incoming: String,
        files: Vec<(String, String)>,
    },
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

    pub fn add_mod<S: Into<String>>(
        &self,
        path: &Path,
        name: S,
        version: S,
        dependencies: &[String],
    ) -> Result<(), ModError> {
        let name = name.into();
        let version = version.into();

        let mut toml = self.load_toml()?;

        let mut archive = ZipArchive::new(File::open(path)?).expect("Could not read zip file");

        let files = read_files(path);

        let crossed_paths = toml.crossover_paths(&name, files);
        if !crossed_paths.is_empty() {
            return Err(ModError::DoubleOwnedFiles {
                incoming: name,
                files: crossed_paths,
            });
        }

        archive.extract_unwrapped_root_dir(self.root.clone(), Self::root_dir_common_filter)?;

        toml.mods.insert(
            name,
            ModEntry {
                version,
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

        Ok(())
    }

    pub fn move_mod<S: Into<String>>(&self, name: S, move_where: Move) -> Result<(), ModError> {
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

        Ok(())
    }

    pub fn load_toml(&self) -> Result<ModRegistry, ModError> {
        let toml_string = fs::read_to_string(&self.toml)?;

        Ok(toml::from_str(&toml_string)?)
    }

    fn clean_upwards(start: &Path, stop: &Path) {
        let mut dir = start;

        while dir != stop {
            if fs::remove_dir(dir).is_err() {
                break;
            }

            if let Some(parent) = dir.parent() {
                dir = parent;
            } else {
                break;
            }
        }
    }

    fn root_dir_common_filter(path: &Path) -> bool {
        const VALID_ROOT_DIRS: &[&str] = &["r6", "archive", "bin", "red4ext", "engine"];

        // Accept only if it's exactly one of the valid root dir names
        if path.components().count() == 1 {
            if let Some(dir_name) = path.file_name() {
                return VALID_ROOT_DIRS
                    .iter()
                    .any(|&valid| OsStr::new(valid) == dir_name);
            }
        }

        false
    }
}
