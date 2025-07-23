use std::{
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

fn exists(path: &str) -> Result<(), &'static str> {
    if Path::new(path).exists() {
        Ok(())
    } else {
        Err("Path does not exist")
    }
}

impl Init {
    pub fn get_path() -> Result<Self, InitError> {
        let t = Input::new("Enter the path to your `Cyberpunk 2077` directory")
            .description("We will use this as a base directory for storing and managing mods.")
            .prompt("Path: ")
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
