use crate::cli::Cli;
use crate::collect::{self, CollectOpts};
use crate::commands::{opts_from, resolve_snapshot};
use crate::config::Config;
use crate::output::print_json;
use anyhow::Result;
use serde::Serialize;

#[derive(Serialize)]
struct ReportOut<'a> {
    tool: &'a str,
    version: &'a str,
    snapshot: &'a collect::Snapshot,
}

pub fn run(cli: &Cli, snapshot: Option<&str>) -> Result<i32> {
    let opts = opts_from(cli);
    let (cfg, _) = Config::load(cli.config.as_deref())?;

    let snap = if let Some(spec) = snapshot {
        resolve_snapshot(spec)?
    } else {
        collect::collect(CollectOpts {
            offline: cli.offline,
            fetch_public_ip: cfg.fetch_public_ip && !cli.offline,
        })
    };

    let out = ReportOut {
        tool: "netidentity",
        version: env!("CARGO_PKG_VERSION"),
        snapshot: &snap,
    };

    if opts.json {
        print_json(opts, &out)?;
    } else if !opts.quiet {
        println!("netidentity report v{}", out.version);
        println!("id:        {}", snap.id);
        println!("created:   {}", snap.created_at);
        println!("hostname:  {}", snap.hostname);
        println!("public_ip: {}", snap.public_ip.as_deref().unwrap_or("(none)"));
        if let Some(ref e) = snap.public_ip_error {
            println!("public_ip_note: {e}");
        }
        println!("gateway:   {}", snap.default_gateway.as_deref().unwrap_or("(none)"));
        println!("ipv6:      {}", snap.ipv6_enabled);
        println!();
        println!("== Interfaces ==");
        for i in &snap.interfaces {
            println!(
                "  {} {:?} mac={:?} v4={:?} vpn={}",
                i.name, i.operstate, i.mac, i.ipv4, i.is_vpn_like
            );
        }
        println!();
        println!("== DNS ==");
        println!("  nameservers: {}", snap.dns.nameservers.join(", "));
        println!("  stub_resolved: {}", snap.dns.stub_resolved);
        println!();
        println!("== VPN ==");
        println!("  hints: {}", snap.vpn.active_hints.join(", "));
        if let Some(ref m) = snap.vpn.mullvad {
            println!("  mullvad: {m}");
        }
        println!();
        println!("== Firewall ==");
        println!(
            "  backend={:?} readable={}",
            snap.firewall.backend, snap.firewall.readable
        );
        for s in snap.firewall.summary.iter().take(15) {
            println!("  {s}");
        }
        println!();
        println!("== Listening (sample) ==");
        for p in snap.listening_ports.iter().take(25) {
            println!("  {} {} {}", p.protocol, p.local, p.state);
        }
        println!();
        println!("== Wi-Fi / mDNS / UPnP ==");
        println!("  ssid={:?}", snap.wifi.ssid);
        println!("  avahi={:?}", snap.mdns_upnp.avahi_running);
        println!("  mdns_port={}", snap.mdns_upnp.mdns_ports_listening);
        println!("  upnp={:?}", snap.mdns_upnp.upnp_hints);
        println!();
        for n in &snap.notes {
            println!("note: {n}");
        }
    }
    Ok(0)
}
