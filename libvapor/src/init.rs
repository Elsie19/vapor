use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use demand::Input;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum InitError {
    #[error("io error: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("missing config at `{0}`")]
    #[diagnostic(help("Vapor attempted to find this config file but failed"))]
    MissingConfig(PathBuf),
}

/// Main config file.
#[derive(Serialize, Deserialize)]
pub struct CyberToml {
    pub main: MainToml,
}

/// Inner contents of [`CyberToml`].
#[derive(Serialize, Deserialize)]
pub struct MainToml {
    /// Path to `Cyberpunk 2077` directory.
    pub path: String,
    /// Time created.
    pub created: DateTime<Utc>,
}

/// Create a new Vapor install.
pub struct Init {
    pub path: PathBuf,
}

impl Init {
    pub fn new() -> Result<Self, InitError> {
        let t = Input::new("Enter the path to your `Cyberpunk 2077` directory")
            .description("We will use this as a base directory for storing and managing mods.")
            .prompt("Path: ")
            .validation(|path| {
                if Path::new(path).exists() {
                    Ok(())
                } else {
                    Err("Path does not exist")
                }
            });

        Ok(Self {
            path: PathBuf::from(t.run()?),
        })
    }

    pub fn setup_cyber(&self) -> Result<(), std::io::Error> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("vapor");
        let config_path = xdg_dirs.place_config_file("Vapor.toml")?;

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

    pub fn get_config() -> Result<PathBuf, InitError> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("vapor");

        xdg_dirs
            .get_config_file("Vapor.toml")
            .ok_or(InitError::MissingConfig(
                xdg_dirs.get_config_home().unwrap().join("Vapor.toml"),
            ))
    }
}

impl FromStr for CyberToml {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}
