use anyhow::{anyhow, Result};
use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

/// Fetch public IP using a minimal HTTPS-less approach via `curl` if present,
/// else a plain HTTP fallback to a known service (best-effort).
pub fn fetch() -> Result<String> {
    if let Ok(out) = std::process::Command::new("curl")
        .args([
            "-fsS",
            "--max-time",
            "5",
            "https://api.ipify.org",
        ])
        .output()
    {
        if out.status.success() {
            let ip = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if is_ip(&ip) {
                return Ok(ip);
            }
        }
    }

    // Fallback: curl to ifconfig.me
    if let Ok(out) = std::process::Command::new("curl")
        .args(["-fsS", "--max-time", "5", "https://ifconfig.me/ip"])
        .output()
    {
        if out.status.success() {
            let ip = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if is_ip(&ip) {
                return Ok(ip);
            }
        }
    }

    // Last resort: raw HTTP to ipify (may fail on HTTPS-only networks)
    if let Ok(mut stream) = TcpStream::connect_timeout(
        &"api.ipify.org:80"
            .to_socket_addrs_helper()?
            .next()
            .ok_or_else(|| anyhow!("resolve failed"))?,
        Duration::from_secs(5),
    ) {
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        use std::io::Write;
        stream.write_all(b"GET / HTTP/1.0\r\nHost: api.ipify.org\r\nConnection: close\r\n\r\n")?;
        let mut buf = String::new();
        stream.read_to_string(&mut buf)?;
        if let Some(body) = buf.split("\r\n\r\n").nth(1) {
            let ip = body.trim().to_string();
            if is_ip(&ip) {
                return Ok(ip);
            }
        }
    }

    Err(anyhow!("could not determine public IP"))
}

fn is_ip(s: &str) -> bool {
    s.parse::<std::net::IpAddr>().is_ok()
}

trait ResolveExt {
    fn to_socket_addrs_helper(&self) -> Result<std::vec::IntoIter<std::net::SocketAddr>>;
}

impl ResolveExt for str {
    fn to_socket_addrs_helper(&self) -> Result<std::vec::IntoIter<std::net::SocketAddr>> {
        use std::net::ToSocketAddrs;
        Ok(self.to_socket_addrs()?.collect::<Vec<_>>().into_iter())
    }
}
