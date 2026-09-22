pub fn summary() -> Vec<String> {
    let mut lines = Vec::new();
    if let Ok(out) = std::process::Command::new("ip")
        .args(["-4", "route", "show"])
        .output()
    {
        if out.status.success() {
            for line in String::from_utf8_lossy(&out.stdout).lines().take(40) {
                lines.push(line.to_string());
            }
        }
    }
    if let Ok(out) = std::process::Command::new("ip")
        .args(["-6", "route", "show"])
        .output()
    {
        if out.status.success() {
            for line in String::from_utf8_lossy(&out.stdout).lines().take(20) {
                lines.push(format!("v6: {line}"));
            }
        }
    }
    lines
}

pub fn default_gateway(routes: &[String]) -> Option<String> {
    for line in routes {
        let line = line.strip_prefix("v6: ").unwrap_or(line);
        if line.starts_with("default ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // default via GATEWAY dev IFACE
            if let Some(i) = parts.iter().position(|p| *p == "via") {
                if let Some(gw) = parts.get(i + 1) {
                    return Some(gw.to_string());
                }
            }
            return Some(line.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_gw() {
        let r = vec!["default via 192.168.1.1 dev eth0 proto dhcp".into()];
        assert_eq!(default_gateway(&r).as_deref(), Some("192.168.1.1"));
    }
}
