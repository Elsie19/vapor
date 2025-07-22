use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct CyberArgs {
    #[command(subcommand)]
    pub cmds: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize `cyber`.
    Init,
    /// Get status of mods.
    Status,
    /// Add a mod.
    Add {
        file: PathBuf,
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        version: String,
    },
}
