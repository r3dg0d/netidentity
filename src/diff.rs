//! Diff two snapshots.

use crate::collect::Snapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReport {
    pub left_id: String,
    pub right_id: String,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub field: String,
    pub change_type: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

pub fn diff(left: &Snapshot, right: &Snapshot) -> DiffReport {
    let mut changes = Vec::new();

    scalar(&mut changes, "hostname", &left.hostname, &right.hostname);
    scalar(
        &mut changes,
        "public_ip",
        &opt(&left.public_ip),
        &opt(&right.public_ip),
    );
    scalar(
        &mut changes,
        "default_gateway",
        &opt(&left.default_gateway),
        &opt(&right.default_gateway),
    );
    scalar(
        &mut changes,
        "ipv6_enabled",
        &left.ipv6_enabled.to_string(),
        &right.ipv6_enabled.to_string(),
    );

    list_diff(&mut changes, "local_ips", &left.local_ips, &right.local_ips);
    list_diff(
        &mut changes,
        "dns.nameservers",
        &left.dns.nameservers,
        &right.dns.nameservers,
    );
    list_diff(
        &mut changes,
        "vpn.active_hints",
        &left.vpn.active_hints,
        &right.vpn.active_hints,
    );

    let left_ifaces: Vec<String> = left.interfaces.iter().map(|i| i.name.clone()).collect();
    let right_ifaces: Vec<String> = right.interfaces.iter().map(|i| i.name.clone()).collect();
    list_diff(&mut changes, "interfaces", &left_ifaces, &right_ifaces);

    let left_macs: Vec<String> = left
        .macs
        .iter()
        .map(|m| format!("{}={}", m.iface, m.mac))
        .collect();
    let right_macs: Vec<String> = right
        .macs
        .iter()
        .map(|m| format!("{}={}", m.iface, m.mac))
        .collect();
    list_diff(&mut changes, "macs", &left_macs, &right_macs);

    let left_ports: Vec<String> = left
        .listening_ports
        .iter()
        .map(|p| format!("{}:{}", p.protocol, p.local))
        .collect();
    let right_ports: Vec<String> = right
        .listening_ports
        .iter()
        .map(|p| format!("{}:{}", p.protocol, p.local))
        .collect();
    list_diff(&mut changes, "listening_ports", &left_ports, &right_ports);

    scalar(
        &mut changes,
        "wifi.ssid",
        &opt(&left.wifi.ssid),
        &opt(&right.wifi.ssid),
    );

    DiffReport {
        left_id: left.id.clone(),
        right_id: right.id.clone(),
        changes,
    }
}

fn opt(v: &Option<String>) -> String {
    v.clone().unwrap_or_else(|| "(none)".into())
}

fn scalar(changes: &mut Vec<Change>, field: &str, before: &str, after: &str) {
    if before != after {
        changes.push(Change {
            field: field.into(),
            change_type: "modified".into(),
            before: Some(before.into()),
            after: Some(after.into()),
        });
    }
}

fn list_diff(changes: &mut Vec<Change>, field: &str, left: &[String], right: &[String]) {
    for item in left {
        if !right.contains(item) {
            changes.push(Change {
                field: field.into(),
                change_type: "removed".into(),
                before: Some(item.clone()),
                after: None,
            });
        }
    }
    for item in right {
        if !left.contains(item) {
            changes.push(Change {
                field: field.into(),
                change_type: "added".into(),
                before: None,
                after: Some(item.clone()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::*;
    use chrono::Utc;

    fn empty_snap(id: &str, hostname: &str) -> Snapshot {
        Snapshot {
            id: id.into(),
            created_at: Utc::now(),
            hostname: hostname.into(),
            interfaces: vec![],
            local_ips: vec!["10.0.0.1".into()],
            public_ip: None,
            public_ip_error: None,
            macs: vec![],
            default_gateway: None,
            routes_summary: vec![],
            vpn: vpn::VpnState::default(),
            dns: dns::DnsState {
                nameservers: vec!["1.1.1.1".into()],
                search: vec![],
                stub_resolved: false,
            },
            ipv6_enabled: false,
            listening_ports: vec![],
            firewall: firewall::FirewallState::default(),
            wifi: wifi::WifiState::default(),
            mdns_upnp: mdns_upnp::MdnsUpnpState::default(),
            offline: true,
            notes: vec![],
        }
    }

    #[test]
    fn detects_hostname_and_ip_change() {
        let mut a = empty_snap("a", "host1");
        let mut b = empty_snap("b", "host2");
        b.local_ips = vec!["10.0.0.2".into()];
        let d = diff(&a, &b);
        assert!(d.changes.iter().any(|c| c.field == "hostname"));
        assert!(d.changes.iter().any(|c| c.field == "local_ips"));
        let _ = &mut a;
    }
}
