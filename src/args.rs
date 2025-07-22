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
        /// Path to mod archive.
        file: PathBuf,

        /// Name of mod.
        #[arg(short, long)]
        name: String,

        /// Mod version.
        #[arg(short, long)]
        version: String,

        /// Dependencies.
        #[arg(short, long, value_delimiter = ',')]
        dependencies: Vec<String>,
    },
    /// Disable a mod.
    Disable {
        /// Mod name.
        name: String,
    },
    /// Enable a mod.
    Enable {
        /// Mod name.
        name: String,
    },
}
