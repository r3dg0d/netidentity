//! Collect network identity facts from the local system.

pub mod dns;
pub mod firewall;
pub mod hostname;
pub mod interfaces;
pub mod listening;
pub mod mdns_upnp;
pub mod public_ip;
pub mod routes;
pub mod vpn;
pub mod wifi;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::sanitize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub hostname: String,
    pub interfaces: Vec<interfaces::Iface>,
    pub local_ips: Vec<String>,
    pub public_ip: Option<String>,
    pub public_ip_error: Option<String>,
    pub macs: Vec<MacEntry>,
    pub default_gateway: Option<String>,
    pub routes_summary: Vec<String>,
    pub vpn: vpn::VpnState,
    pub dns: dns::DnsState,
    pub ipv6_enabled: bool,
    pub listening_ports: Vec<listening::ListenEntry>,
    pub firewall: firewall::FirewallState,
    pub wifi: wifi::WifiState,
    pub mdns_upnp: mdns_upnp::MdnsUpnpState,
    pub offline: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacEntry {
    pub iface: String,
    pub mac: String,
}

#[derive(Debug, Clone)]
pub struct CollectOpts {
    pub offline: bool,
    pub fetch_public_ip: bool,
}

pub fn collect(opts: CollectOpts) -> Snapshot {
    let hostname = hostname::get_hostname();
    let ifaces = interfaces::list_interfaces();
    let mut local_ips = Vec::new();
    let mut macs = Vec::new();
    let mut ipv6_enabled = false;

    for i in &ifaces {
        for a in &i.ipv4 {
            local_ips.push(a.clone());
        }
        for a in &i.ipv6 {
            if !a.starts_with("fe80:") {
                ipv6_enabled = true;
            }
            local_ips.push(a.clone());
        }
        if let Some(ref m) = i.mac {
            macs.push(MacEntry {
                iface: i.name.clone(),
                mac: m.clone(),
            });
        }
    }

    let (public_ip, public_ip_error) = if opts.offline || !opts.fetch_public_ip {
        (
            None,
            if opts.offline {
                Some("offline mode".into())
            } else {
                Some("public IP fetch disabled".into())
            },
        )
    } else {
        match public_ip::fetch() {
            Ok(ip) => (Some(ip), None),
            Err(e) => (None, Some(e.to_string())),
        }
    };

    let routes = routes::summary();
    let default_gateway = routes::default_gateway(&routes);
    let vpn = vpn::detect();
    let dns = dns::collect();
    let listening_ports = listening::list();
    let firewall = firewall::detect();
    let wifi = wifi::detect();
    let mdns_upnp = mdns_upnp::detect();

    let mut snap = Snapshot {
        id: Uuid::new_v4().to_string(),
        created_at: Utc::now(),
        hostname,
        interfaces: ifaces,
        local_ips,
        public_ip,
        public_ip_error,
        macs,
        default_gateway,
        routes_summary: routes,
        vpn,
        dns,
        ipv6_enabled,
        listening_ports,
        firewall,
        wifi,
        mdns_upnp,
        offline: opts.offline,
        notes: vec![
            "Secrets are redacted when detected in free-form fields.".into(),
            "Public IP fetch uses HTTPS to commonly used echo services when not --offline.".into(),
        ],
    };

    sanitize_snapshot(&mut snap);
    snap
}

fn sanitize_snapshot(s: &mut Snapshot) {
    s.hostname = sanitize::redact(&s.hostname);
    for r in &mut s.routes_summary {
        sanitize::redact_string_fields(r);
    }
    for n in &mut s.notes {
        sanitize::redact_string_fields(n);
    }
    if let Some(ref mut e) = s.public_ip_error {
        sanitize::redact_string_fields(e);
    }
}
