mod cli;
mod collect;
mod commands;
mod config;
mod diff;
mod exit_codes;
mod output;
mod sanitize;

use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};

fn main() {
    let code = match real_main() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e:#}");
            exit_codes::GENERAL_ERROR
        }
    };
    std::process::exit(code);
}

fn real_main() -> anyhow::Result<i32> {
    let cli = Cli::parse();

    let filter = match (cli.quiet, cli.verbose) {
        (true, _) => "error",
        (false, 0) => "warn",
        (false, 1) => "info",
        (false, _) => "debug",
    };
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    ctrlc::set_handler(|| {
        eprintln!("\ninterrupted");
        std::process::exit(exit_codes::INTERRUPTED);
    })?;

    match &cli.command {
        Commands::Snapshot { no_save } => commands::snapshot::run(&cli, *no_save),
        Commands::Diff { left, right } => commands::diff_cmd::run(&cli, left, right),
        Commands::Report { snapshot } => commands::report::run(&cli, snapshot.as_deref()),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            clap_complete::generate(
                clap_complete::Shell::from(*shell),
                &mut cmd,
                name,
                &mut std::io::stdout(),
            );
            Ok(exit_codes::SUCCESS)
        }
    }
}
