use std::fs;

use anyhow::anyhow;
use args::{Command, CyberArgs};
use chrono::Utc;
use chrono_humanize::HumanTime;
use clap::Parser;
use init::{CyberToml, Init};
use inline_colorization::*;
use mod_manager::handler::{ModHandler, Move};

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
            let mut ret = 0;
            let config_path = Init::get_config().ok_or(anyhow!("Cannot get config file"))?;
            let config: CyberToml = toml::from_str(&fs::read_to_string(&config_path)?)?;
            let toml = ModHandler::new(config.main.path.into()).load_toml()?;

            for (mod_name, contents) in &toml.mods {
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
                let deps = toml.satisfied_deps(mod_name);
                if !deps.is_empty() {
                    ret = 1;
                    println!("  - Missing dependencies:");
                    for dep in &deps {
                        println!("      > `{color_red}{dep}{style_reset}`");
                    }
                }
                if let Some(dependencies) = &contents.dependencies {
                    let dependencies: Vec<_> = dependencies
                        .iter()
                        .filter(|dep| !deps.contains(dep))
                        .collect();
                    if !dependencies.is_empty() {
                        println!("  - Dependencies:");
                        for dep in dependencies {
                            println!("      > `{dep}`");
                        }
                    }
                }
            }

            std::process::exit(ret);
        }
        Command::Add {
            file,
            name,
            version,
            dependencies,
        } => {
            let config_path = Init::get_config().ok_or(anyhow!("Cannot get config file"))?;
            let config: CyberToml = toml::from_str(&fs::read_to_string(&config_path)?)?;
            let handler = ModHandler::new(config.main.path.into());

            handler.add_mod(&file, name, version, &dependencies)?;
        }
        Command::Disable { name } => {
            let config_path = Init::get_config().ok_or(anyhow!("Cannot get config file"))?;
            let config: CyberToml = toml::from_str(&fs::read_to_string(&config_path)?)?;
            let handler = ModHandler::new(config.main.path.into());

            handler.move_mod(&name, Move::Disable)?;
            println!(":: Disabled `{name}`");
        }
        Command::Enable { name } => {
            let config_path = Init::get_config().ok_or(anyhow!("Cannot get config file"))?;
            let config: CyberToml = toml::from_str(&fs::read_to_string(&config_path)?)?;
            let handler = ModHandler::new(config.main.path.into());

            handler.move_mod(&name, Move::Enable)?;
            println!(":: Enabled `{name}`");
        }
        Command::List { name } => {
            let config_path = Init::get_config().ok_or(anyhow!("Cannot get config file"))?;
            let config: CyberToml = toml::from_str(&fs::read_to_string(&config_path)?)?;

            let toml = ModHandler::new(config.main.path.into()).load_toml()?;

            if let Some(mod_name) = toml.mods.get(&name) {
                for file in &mod_name.files {
                    println!("{file}");
                }
            } else {
                eprintln!("No mod named `{name}` found");
                std::process::exit(1);
            }
        }
    };

    Ok(())
}
