use crate::cli::Cli;
use crate::collect::{self, CollectOpts};
use crate::commands::opts_from;
use crate::config::Config;
use crate::output::{self, print_json};
use anyhow::Result;
use std::fs;

pub fn run(cli: &Cli, no_save: bool) -> Result<i32> {
    let opts = opts_from(cli);
    let (cfg, _) = Config::load(cli.config.as_deref())?;

    let snap = collect::collect(CollectOpts {
        offline: cli.offline,
        fetch_public_ip: cfg.fetch_public_ip && !cli.offline,
    });

    let should_save = !no_save && !cli.dry_run;
    let mut saved_path = None;
    if should_save {
        let dir = Config::snapshot_dir()?;
        let path = dir.join(format!("{}.json", snap.id));
        let text = serde_json::to_string_pretty(&snap)?;
        fs::write(&path, text)?;
        saved_path = Some(path);
        tracing::info!(path = %saved_path.as_ref().unwrap().display(), "saved snapshot");
    } else if cli.dry_run {
        output::human(opts, "dry-run: snapshot not written");
    }

    if opts.json {
        print_json(opts, &snap)?;
    } else if !opts.quiet {
        println!("snapshot {}", snap.id);
        println!("  created:   {}", snap.created_at);
        println!("  hostname:  {}", snap.hostname);
        println!("  public_ip: {}", snap.public_ip.as_deref().unwrap_or("(none)"));
        println!("  gateway:   {}", snap.default_gateway.as_deref().unwrap_or("(none)"));
        println!("  ifaces:    {}", snap.interfaces.len());
        println!("  vpn hints: {}", snap.vpn.active_hints.join(", "));
        if let Some(ref ssid) = snap.wifi.ssid {
            println!("  wifi:      {ssid}");
        }
        if let Some(p) = saved_path {
            println!("  saved:     {}", p.display());
        }
    }
    Ok(0)
}
