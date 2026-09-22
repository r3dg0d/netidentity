use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DnsState {
    pub nameservers: Vec<String>,
    pub search: Vec<String>,
    pub stub_resolved: bool,
}

pub fn collect() -> DnsState {
    let mut st = DnsState::default();
    if let Ok(text) = fs::read_to_string("/etc/resolv.conf") {
        for line in text.lines() {
            let t = line.trim();
            if t.starts_with('#') {
                if t.to_lowercase().contains("systemd-resolved") {
                    st.stub_resolved = true;
                }
                continue;
            }
            let mut p = t.split_whitespace();
            match p.next() {
                Some("nameserver") => {
                    if let Some(ns) = p.next() {
                        if ns == "127.0.0.53" {
                            st.stub_resolved = true;
                        }
                        st.nameservers.push(ns.to_string());
                    }
                }
                Some("search") | Some("domain") => {
                    st.search.extend(p.map(|s| s.to_string()));
                }
                _ => {}
            }
        }
    }
    st
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_ok() {
        let _ = collect();
    }
}
