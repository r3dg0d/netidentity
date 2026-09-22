#[test]
fn help_works() {
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_netidentity"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(o.status.success());
}

#[test]
fn snapshot_offline_json() {
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_netidentity"))
        .args(["--json", "--offline", "snapshot", "--no-save"])
        .output()
        .unwrap();
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(v.get("id").is_some());
    assert!(v.get("hostname").is_some());
}

#[test]
fn completions() {
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_netidentity"))
        .args(["completions", "zsh"])
        .output()
        .unwrap();
    assert!(o.status.success());
}
