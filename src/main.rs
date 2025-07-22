use args::{Command, CyberArgs};
use clap::Parser;
use init::Init;

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
    };

    Ok(())
}
