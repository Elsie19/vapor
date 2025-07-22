use std::fs;

use anyhow::anyhow;
use args::{Command, CyberArgs};
use clap::Parser;
use init::{CyberToml, Init};
use mod_manager::{handler::ModHandler, mod_file_formats::read_files};

mod args;
mod init;
mod mod_manager;

fn main() -> anyhow::Result<()> {
    let cli = CyberArgs::parse();

    match cli.cmds {
        Command::Init => {
            let cyber_directory = Init::get_path()?;
            cyber_directory.setup_cyber()?;
        }
        Command::Status => {
            println!("Doing ok");
        }
        Command::Add {
            file,
            name,
            version,
        } => {
            let config_path = Init::get_config().ok_or(anyhow!("Cannot get config file"))?;
            let config: CyberToml = toml::from_str(&fs::read_to_string(&config_path)?)?;
            let handler = ModHandler::new(config.main.path.into());

            handler.add_mod(&file, name, version)?;
        }
    };

    Ok(())
}
