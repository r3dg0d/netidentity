use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WifiState {
    pub ssid: Option<String>,
    pub device: Option<String>,
    pub method: Option<String>,
}

pub fn detect() -> WifiState {
    let mut st = WifiState::default();

    if command_exists("nmcli") {
        if let Ok(out) = Command::new("nmcli")
            .args(["-t", "-f", "ACTIVE,SSID,DEVICE", "dev", "wifi"])
            .output()
        {
            if out.status.success() {
                for line in String::from_utf8_lossy(&out.stdout).lines() {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.first() == Some(&"yes") {
                        st.ssid = parts.get(1).map(|s| s.to_string()).filter(|s| !s.is_empty());
                        st.device = parts.get(2).map(|s| s.to_string());
                        st.method = Some("nmcli".into());
                        return st;
                    }
                }
            }
        }
        // connection show
        if let Ok(out) = Command::new("nmcli").args(["-t", "-f", "NAME,TYPE,DEVICE", "connection", "show", "--active"]).output() {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                if line.contains("wireless") || line.contains("wifi") {
                    let parts: Vec<&str> = line.split(':').collect();
                    st.ssid = parts.first().map(|s| s.to_string());
                    st.device = parts.get(2).map(|s| s.to_string());
                    st.method = Some("nmcli-connection".into());
                    return st;
                }
            }
        }
    }

    if command_exists("iw") {
        if let Ok(out) = Command::new("iw").args(["dev"]).output() {
            let text = String::from_utf8_lossy(&out.stdout);
            let mut iface = None;
            for line in text.lines() {
                let t = line.trim();
                if let Some(rest) = t.strip_prefix("Interface ") {
                    iface = Some(rest.to_string());
                }
                if let Some(rest) = t.strip_prefix("ssid ") {
                    st.ssid = Some(rest.to_string());
                    st.device = iface.clone();
                    st.method = Some("iw".into());
                    return st;
                }
            }
        }
    }

    st
}

fn command_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|p| p.join(name).is_file()))
        .unwrap_or(false)
}
