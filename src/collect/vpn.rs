use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VpnState {
    pub mullvad: Option<String>,
    pub wireguard_ifaces: Vec<String>,
    pub tailscale: Option<String>,
    pub active_hints: Vec<String>,
}

pub fn detect() -> VpnState {
    let mut st = VpnState::default();

    if command_exists("mullvad") {
        if let Ok(out) = Command::new("mullvad").arg("status").output() {
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !text.is_empty() {
                if text.to_lowercase().contains("connected") {
                    st.active_hints.push("mullvad-connected".into());
                }
                st.mullvad = Some(text.chars().take(500).collect());
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("wg")
                || name.contains("wireguard")
                || name.starts_with("mullvad")
            {
                let state = std::fs::read_to_string(e.path().join("operstate"))
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "?".into());
                st.wireguard_ifaces.push(format!("{name}:{state}"));
                if state == "up" {
                    st.active_hints.push(format!("wg-up:{name}"));
                }
            }
        }
    }

    // wg show
    if command_exists("wg") {
        if let Ok(out) = Command::new("wg").arg("show").arg("interfaces").output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for iface in text.split_whitespace() {
                    let entry = format!("{iface}:wg-tool");
                    if !st.wireguard_ifaces.iter().any(|x| x.starts_with(iface)) {
                        st.wireguard_ifaces.push(entry);
                    }
                    st.active_hints.push(format!("wg-iface:{iface}"));
                }
            }
        }
    }

    if command_exists("tailscale") {
        if let Ok(out) = Command::new("tailscale").arg("status").arg("--json").output() {
            if out.status.success() {
                st.tailscale = Some("tailscale status --json ok".into());
                st.active_hints.push("tailscale-present".into());
            } else if let Ok(out) = Command::new("tailscale").arg("status").output() {
                let t = String::from_utf8_lossy(&out.stdout);
                st.tailscale = Some(t.chars().take(300).collect());
                if !t.to_lowercase().contains("stopped") {
                    st.active_hints.push("tailscale-status".into());
                }
            }
        }
        // iface
        if std::path::Path::new("/sys/class/net/tailscale0").exists() {
            st.active_hints.push("tailscale0-iface".into());
        }
    }

    st
}

fn command_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|p| p.join(name).is_file()))
        .unwrap_or(false)
}
