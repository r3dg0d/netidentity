use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Iface {
    pub name: String,
    pub operstate: Option<String>,
    pub mac: Option<String>,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    pub is_loopback: bool,
    pub is_vpn_like: bool,
}

pub fn list_interfaces() -> Vec<Iface> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/net") else {
        return out;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let operstate = read_trim(path.join("operstate"));
        let mac = read_trim(path.join("address")).filter(|m| m != "00:00:00:00:00:00");
        let (ipv4, ipv6) = addrs_for(&name);
        let is_loopback = name == "lo";
        let is_vpn_like = vpn_like(&name);
        out.push(Iface {
            name,
            operstate,
            mac,
            ipv4,
            ipv6,
            is_loopback,
            is_vpn_like,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn read_trim(p: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(p).ok().map(|s| s.trim().to_string())
}

fn addrs_for(iface: &str) -> (Vec<String>, Vec<String>) {
    let mut v4 = Vec::new();
    let mut v6 = Vec::new();
    if let Ok(out) = std::process::Command::new("ip")
        .args(["-o", "addr", "show", "dev", iface])
        .output()
    {
        if out.status.success() {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    match parts[2] {
                        "inet" => v4.push(parts[3].to_string()),
                        "inet6" => v6.push(parts[3].to_string()),
                        _ => {}
                    }
                }
            }
        }
    }
    (v4, v6)
}

pub fn vpn_like(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.starts_with("wg")
        || n.starts_with("tun")
        || n.starts_with("tap")
        || n.starts_with("mullvad")
        || n.starts_with("tailscale")
        || n.starts_with("nordlynx")
        || n.contains("wireguard")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vpn_names() {
        assert!(vpn_like("wg0"));
        assert!(!vpn_like("enp0s3"));
    }
}
