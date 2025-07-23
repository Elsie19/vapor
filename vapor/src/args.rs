use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// A Cyberpunk 2077 mod manager for Linux.
#[derive(Parser, Debug)]
pub struct CyberArgs {
    #[command(subcommand)]
    pub cmds: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize `vapor`.
    Init,
    /// Get status of mods.
    Status {
        /// JSON output.
        #[arg(long)]
        json: bool,
    },
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
        ///
        /// This should be passed by a comma (`,`) delimited list.
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
    /// List mods or a mod's files
    List {
        /// Mod name.
        name: Option<String>,
    },
}
