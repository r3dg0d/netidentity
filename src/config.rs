use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default = "default_true")]
    pub fetch_public_ip: bool,
    #[serde(default)]
    pub public_ip_urls: Option<Vec<String>>,
    #[serde(default)]
    pub redact_patterns: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Config {
    pub fn load(explicit: Option<&Path>) -> Result<(Self, Option<PathBuf>)> {
        if let Some(p) = explicit {
            let text =
                fs::read_to_string(p).with_context(|| format!("reading config {}", p.display()))?;
            let cfg: Config = serde_json::from_str(&text)?;
            return Ok((cfg, Some(p.to_path_buf())));
        }
        if let Some(dirs) = ProjectDirs::from("dev", "r3dg0d", "netidentity") {
            let path = dirs.config_dir().join("config.json");
            if path.exists() {
                let text = fs::read_to_string(&path)?;
                let cfg: Config = serde_json::from_str(&text)?;
                return Ok((cfg, Some(path)));
            }
        }
        Ok((Config::default(), None))
    }

    pub fn snapshot_dir() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("dev", "r3dg0d", "netidentity")
            .ok_or_else(|| anyhow::anyhow!("cannot resolve XDG dirs"))?;
        let p = dirs.data_dir().join("snapshots");
        fs::create_dir_all(&p)?;
        Ok(p)
    }

    #[allow(dead_code)]
    pub fn config_dir() -> Option<PathBuf> {
        ProjectDirs::from("dev", "r3dg0d", "netidentity").map(|d| d.config_dir().to_path_buf())
    }
}
