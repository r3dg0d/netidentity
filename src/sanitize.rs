//! Redact secrets from snapshot strings.

use regex::Regex;
use std::sync::OnceLock;

fn patterns() -> &'static [Regex] {
    static CELL: OnceLock<Vec<Regex>> = OnceLock::new();
    CELL.get_or_init(|| {
        vec![
            Regex::new(r"(?i)bearer\s+[a-z0-9\-._~+/]+=*").unwrap(),
            Regex::new(r"(?i)(api[_-]?key|token|secret|password|passwd|authorization)\s*[:=]\s*\S+.*").unwrap(),
            Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----").unwrap(),
            Regex::new(r"(?i)psk\s*[:=]\s*\S+").unwrap(),
        ]
    })
}

pub fn redact(input: &str) -> String {
    let mut out = input.to_string();
    for re in patterns() {
        out = re.replace_all(&out, "[REDACTED]").to_string();
    }
    out
}

pub fn redact_string_fields(s: &mut String) {
    *s = redact(s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_token() {
        let s = redact("Authorization: Bearer abcdef123456");
        assert!(s.contains("[REDACTED]"), "got: {s}");
        assert!(!s.contains("abcdef"), "got: {s}");
    }

    #[test]
    fn leaves_normal() {
        assert_eq!(redact("eth0 up 192.168.1.5"), "eth0 up 192.168.1.5");
    }
}
