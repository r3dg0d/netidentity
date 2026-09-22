use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirewallState {
    pub backend: Option<String>,
    pub readable: bool,
    pub summary: Vec<String>,
}

pub fn detect() -> FirewallState {
    let mut st = FirewallState::default();

    if command_exists("nft") {
        if let Ok(out) = Command::new("nft").args(["list", "ruleset"]).output() {
            st.backend = Some("nftables".into());
            if out.status.success() {
                st.readable = true;
                let text = String::from_utf8_lossy(&out.stdout);
                st.summary.push(format!("nft ruleset lines: {}", text.lines().count()));
                for line in text.lines().filter(|l| {
                    let t = l.trim();
                    t.starts_with("table ") || t.starts_with("chain ")
                }).take(30) {
                    st.summary.push(line.trim().to_string());
                }
                return st;
            } else {
                st.summary.push(format!(
                    "nft present but not readable: {}",
                    String::from_utf8_lossy(&out.stderr).trim().chars().take(200).collect::<String>()
                ));
            }
        }
    }

    if command_exists("iptables") {
        if let Ok(out) = Command::new("iptables").args(["-S"]).output() {
            st.backend = Some(st.backend.take().unwrap_or_else(|| "iptables".into()));
            if out.status.success() {
                st.readable = true;
                let text = String::from_utf8_lossy(&out.stdout);
                st.summary.push(format!("iptables -S lines: {}", text.lines().count()));
                for line in text.lines().take(40) {
                    st.summary.push(line.to_string());
                }
            } else {
                st.summary.push("iptables present but not readable (need root?)".into());
            }
        }
    }

    if st.backend.is_none() {
        st.summary.push("no nft/iptables detected in PATH".into());
    }
    st
}

fn command_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|p| p.join(name).is_file()))
        .unwrap_or(false)
}
