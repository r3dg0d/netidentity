pub mod diff_cmd;
pub mod report;
pub mod snapshot;

use crate::cli::Cli;
use crate::collect::Snapshot;
use crate::config::Config;
use crate::output::OutputOpts;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn opts_from(cli: &Cli) -> OutputOpts {
    OutputOpts {
        json: cli.json,
        quiet: cli.quiet,
        verbose: cli.verbose > 0,
    }
}

pub fn resolve_snapshot(spec: &str) -> Result<Snapshot> {
    let path = resolve_path(spec)?;
    let text = fs::read_to_string(&path)
        .with_context(|| format!("reading snapshot {}", path.display()))?;
    let snap: Snapshot = serde_json::from_str(&text)
        .with_context(|| format!("parsing snapshot {}", path.display()))?;
    Ok(snap)
}

pub fn resolve_path(spec: &str) -> Result<PathBuf> {
    let p = Path::new(spec);
    if p.exists() {
        return Ok(p.to_path_buf());
    }
    let dir = Config::snapshot_dir()?;
    let candidate = dir.join(format!("{spec}.json"));
    if candidate.exists() {
        return Ok(candidate);
    }
    // prefix match
    if let Ok(entries) = fs::read_dir(&dir) {
        let mut matches = Vec::new();
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with(spec) && name.ends_with(".json") {
                matches.push(e.path());
            }
        }
        if matches.len() == 1 {
            return Ok(matches.remove(0));
        }
        if matches.len() > 1 {
            anyhow::bail!("ambiguous snapshot id '{spec}' ({} matches)", matches.len());
        }
    }
    anyhow::bail!("snapshot not found: {spec}")
}
