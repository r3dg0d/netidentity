use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MdnsUpnpState {
    pub avahi_running: Option<bool>,
    pub mdns_ports_listening: bool,
    pub upnp_hints: Vec<String>,
    pub notes: Vec<String>,
}

pub fn detect() -> MdnsUpnpState {
    let mut st = MdnsUpnpState {
        notes: vec!["mDNS/UPnP detection is best-effort without heavy SSDP libraries.".into()],
        ..Default::default()
    };

    // avahi
    if Path::new("/run/avahi-daemon/socket").exists()
        || Path::new("/var/run/avahi-daemon/socket").exists()
    {
        st.avahi_running = Some(true);
    } else if command_exists("systemctl") {
        if let Ok(out) = Command::new("systemctl")
            .args(["is-active", "avahi-daemon"])
            .output()
        {
            let t = String::from_utf8_lossy(&out.stdout).trim().to_string();
            st.avahi_running = Some(t == "active");
        }
    }

    // port 5353
    if let Ok(out) = Command::new("ss").args(["-ulnH"]).output() {
        let text = String::from_utf8_lossy(&out.stdout);
        st.mdns_ports_listening = text.contains(":5353");
    }

    // UPnP / IGD hints: listening 1900/udp, miniupnpd unit
    if let Ok(out) = Command::new("ss").args(["-ulnH"]).output() {
        let text = String::from_utf8_lossy(&out.stdout);
        if text.contains(":1900") {
            st.upnp_hints.push("udp/1900 listening (SSDP)".into());
        }
    }
    if command_exists("systemctl") {
        if let Ok(out) = Command::new("systemctl")
            .args(["is-active", "miniupnpd"])
            .output()
        {
            if String::from_utf8_lossy(&out.stdout).trim() == "active" {
                st.upnp_hints.push("miniupnpd active".into());
            }
        }
    }
    if Path::new("/etc/miniupnpd").exists() {
        st.upnp_hints.push("/etc/miniupnpd present".into());
    }

    st
}

fn command_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|p| p.join(name).is_file()))
        .unwrap_or(false)
}
