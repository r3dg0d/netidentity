use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenEntry {
    pub protocol: String,
    pub local: String,
    pub state: String,
    pub process: Option<String>,
}

pub fn list() -> Vec<ListenEntry> {
    if let Some(v) = from_ss() {
        return v;
    }
    from_proc()
}

fn from_ss() -> Option<Vec<ListenEntry>> {
    let out = Command::new("ss").args(["-tulpnH"]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let mut entries = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }
        // Netid State Recv-Q Send-Q Local Address:Port Peer Address:Port Process
        let protocol = parts[0].to_string();
        let state = parts[1].to_string();
        if state != "LISTEN" && protocol != "udp" && protocol != "udp6" {
            // ss -tulpn includes UDP without LISTEN
        }
        let local = parts[4].to_string();
        let process = parts.get(6).map(|s| s.to_string());
        entries.push(ListenEntry {
            protocol,
            local,
            state,
            process,
        });
    }
    Some(entries)
}

fn from_proc() -> Vec<ListenEntry> {
    let mut entries = Vec::new();
    for (file, proto) in [
        ("/proc/net/tcp", "tcp"),
        ("/proc/net/tcp6", "tcp6"),
        ("/proc/net/udp", "udp"),
        ("/proc/net/udp6", "udp6"),
    ] {
        let Ok(text) = fs::read_to_string(file) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            if i == 0 {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                continue;
            }
            // local_address remote_address st
            let local = parse_proc_addr(parts[1]);
            let state = parts[3].to_string();
            // TCP listen == 0A
            if proto.starts_with("tcp") && state != "0A" {
                continue;
            }
            entries.push(ListenEntry {
                protocol: proto.into(),
                local,
                state,
                process: None,
            });
        }
    }
    entries
}

pub fn parse_proc_addr(s: &str) -> String {
    let Some((ip_hex, port_hex)) = s.split_once(':') else {
        return s.to_string();
    };
    let port = u16::from_str_radix(port_hex, 16).unwrap_or(0);
    if ip_hex.len() == 8 {
        // IPv4 little-endian hex
        if let Ok(n) = u32::from_str_radix(ip_hex, 16) {
            let b = n.to_le_bytes();
            return format!("{}.{}.{}.{}:{port}", b[0], b[1], b[2], b[3]);
        }
    }
    format!("{ip_hex}:{port}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_ipv4_proc() {
        // 0100007F:0050 = 127.0.0.1:80
        assert_eq!(parse_proc_addr("0100007F:0050"), "127.0.0.1:80");
    }
}
