use std::{fs, str::FromStr};

use args::{Command, CyberArgs};
use clap::Parser;
use libvapor::init::{CyberToml, Init};
use libvapor::mod_manager::handler::{ModHandler, Move, Operation};
use miette::{IntoDiagnostic, LabeledSpan, Result, miette};

mod args;

fn load_config() -> Result<CyberToml> {
    let config_path = Init::get_config()?;
    CyberToml::from_str(&fs::read_to_string(&config_path).into_diagnostic()?).into_diagnostic()
}

fn main() -> Result<()> {
    let cli = CyberArgs::parse();

    match cli.cmds {
        Command::Init => {
            Init::new()?.setup_cyber().into_diagnostic()?;
        }
        Command::Status { json } => {
            let config = load_config()?;
            let toml = ModHandler::new(config.main.path).load_toml()?;
            let (out, code) = toml.status(json);

            print!("{out}");

            std::process::exit(code);
        }
        Command::Add {
            file,
            name,
            version,
            dependencies,
        } => {
            let config = load_config()?;
            let handler = ModHandler::new(config.main.path);
            let change = handler.add_mod(&file, name.clone(), version, &dependencies)?;

            match change {
                Operation::Added(_) => println!("`{name}` is now active!"),
                Operation::Updated { old, new } => {
                    println!("Updated `{name}` from `{old}` ~> `{new}`")
                }
                Operation::Move(_) => unreachable!("Moving doesn't happen in `Add`"),
            }
        }
        ref at @ (Command::Disable { ref name } | Command::Enable { ref name }) => {
            let config = load_config()?;
            let handler = ModHandler::new(config.main.path);

            let which = match at {
                Command::Disable { .. } => Move::Disable,
                Command::Enable { .. } => Move::Enable,
                _ => unreachable!("How"),
            };
            let change = handler.move_mod(name, which)?;
            match change {
                Operation::Move(moved) => println!(
                    "{} `{name}`",
                    match moved {
                        Move::Enable => "Disabled",
                        Move::Disable => "Enabled",
                    }
                ),
                _ => unreachable!("Others not possible in disable or enable"),
            }
        }
        Command::List { name } => {
            let config = load_config()?;
            let toml = ModHandler::new(config.main.path).load_toml()?;

            match name {
                Some(name) if !name.is_empty() => {
                    if let Some(mod_name) = toml.mods.get(&name) {
                        for file in &mod_name.files {
                            println!("{file}");
                        }
                    } else {
                        let source = format!("vapor list {name}");
                        let report = miette!(
                            labels = vec![LabeledSpan::at(
                                source.len() - name.len()..source.len(),
                                "invalid mod name"
                            )],
                            help = "Specify a valid mod found in `vapor list`!",
                            "No mod named `{name}` found!"
                        )
                        .with_source_code(source);
                        eprintln!("{report:?}");
                        std::process::exit(1);
                    }
                }
                _ => {
                    for (mod_name, entry) in toml.mods {
                        if entry.installed {
                            println!("{mod_name}");
                        }
                    }
                }
            }
        }
        Command::Graph => {
            let config = load_config()?;
            let toml = ModHandler::new(config.main.path).load_toml()?;
            print!("{}", toml.graph());
        }
    }

    Ok(())
}
