use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "netidentity",
    author = "r3dg0d",
    version,
    about = "Network identity snapshot — capture, compare, and report local network identity"
)]
pub struct Cli {
    #[arg(long, global = true)]
    pub json: bool,

    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Do not fetch public IP or other remote data
    #[arg(long, global = true)]
    pub offline: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Capture and save a network identity snapshot
    Snapshot {
        /// Print only; do not write to disk (also implied by --dry-run)
        #[arg(long)]
        no_save: bool,
    },
    /// Compare two snapshots by id or path
    Diff {
        /// Left snapshot id or path
        left: String,
        /// Right snapshot id or path
        right: String,
    },
    /// Human-readable report from a new or existing snapshot
    Report {
        /// Optional snapshot id or path (default: live collect)
        snapshot: Option<String>,
    },
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    Powershell,
    Zsh,
}

impl From<Shell> for clap_complete::Shell {
    fn from(s: Shell) -> Self {
        match s {
            Shell::Bash => Self::Bash,
            Shell::Elvish => Self::Elvish,
            Shell::Fish => Self::Fish,
            Shell::Powershell => Self::PowerShell,
            Shell::Zsh => Self::Zsh,
        }
    }
}
