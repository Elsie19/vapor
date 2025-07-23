use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use demand::Input;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitError {
    #[error("io error: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("parse error: `{0}`")]
    ParseError(#[from] keyvalues_serde::Error),
}

#[derive(Deserialize, Debug)]
struct LibraryFolders(pub HashMap<String, LibraryEntry>);

#[derive(Deserialize, Debug)]
struct LibraryEntry {
    pub path: String,
    pub label: Option<String>,
    pub contentid: Option<String>,
    pub totalsize: Option<String>,
    pub update_clean_bytes_tally: Option<String>,
    pub time_last_update_verified: Option<String>,

    pub apps: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
pub struct CyberToml {
    pub main: MainToml,
}

#[derive(Serialize, Deserialize)]
pub struct MainToml {
    pub path: String,
    pub created: DateTime<Utc>,
}

pub struct Init {
    pub path: PathBuf,
}

fn find_vdf_path() -> Option<PathBuf> {
    let candidates = [
        // Flatpak.
        "~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/",
        // Native install.
        "~/.steam/steam/steamapps/",
        "~/.steam/steamapps/",
        "~/.local/share/Steam/steamapps/",
        // Old or symlinked layout.
        "~/.steam/root/steamapps/",
        "~/.steam/root/SteamApps/",
    ];

    for path in &candidates {
        let full = shellexpand::tilde(path).to_string();
        let vdf = Path::new(&full).join("libraryfolders.vdf");
        if vdf.exists() {
            return Some(vdf);
        }
    }

    None
}

fn exists(path: &str) -> Result<(), &'static str> {
    if Path::new(path).exists() {
        Ok(())
    } else {
        Err("Path does not exist")
    }
}

impl Init {
    pub fn get_path() -> Result<Self, InitError> {
        let tried_paths = find_vdf_path();
        let mut suggestions = vec![];

        if let Some(paths) = tried_paths {
            let handle = File::open(paths)?;
            let platform: LibraryFolders = keyvalues_serde::from_reader(handle)?;
            for (_, library_entry) in platform.0 {
                suggestions.push(format!(
                    "{}/{}/{}",
                    library_entry.path, "steamapps", "common"
                ));
            }
        }

        let binding = suggestions
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();

        let t = Input::new("Enter the path to your `Cyberpunk 2077` directory")
            .description("We will use this as a base directory for storing and managing mods.")
            .prompt("Path: ")
            .suggestions(&binding)
            .validation(exists);

        Ok(Self {
            path: PathBuf::from(t.run()?),
        })
    }

    pub fn setup_cyber(&self) -> Result<(), std::io::Error> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("cyber");
        let config_path = xdg_dirs.place_config_file("Cyber.toml")?;

        let mut config_file = File::create_new(config_path)?;

        write!(
            &mut config_file,
            "{}",
            toml::to_string_pretty(&CyberToml {
                main: MainToml {
                    path: self.path.to_string_lossy().to_string(),
                    created: Utc::now(),
                }
            })
            .expect("Could not serialize")
        )?;

        File::create_new(self.path.join("mods.toml"))?;

        fs::create_dir(self.path.join("Disabled Mods"))?;

        Ok(())
    }

    pub fn get_config() -> Option<PathBuf> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("cyber");

        xdg_dirs.get_config_file("Cyber.toml")
    }
}
