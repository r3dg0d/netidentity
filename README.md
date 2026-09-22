# netidentity

**Network identity snapshot** — capture hostname, interfaces, IPs, MACs, gateway, VPN, DNS, listening ports, firewall, Wi-Fi, and mDNS/UPnP hints. Compare snapshots over time.

Owner: **r3dg0d** · License: **MIT**

## Install

```bash
cargo build --release
# Nix:
nix build && ./result/bin/netidentity snapshot --offline
```

Completions: `netidentity completions bash`.

## Commands

```bash
netidentity snapshot                 # collect + save under XDG data dir
netidentity snapshot --offline       # skip public IP fetch
netidentity snapshot --dry-run       # collect but do not write
netidentity snapshot --no-save
netidentity diff <id|path> <id|path>
netidentity report                   # live report
netidentity report <id|path>
netidentity report --json
```

Snapshots: `~/.local/share/netidentity/snapshots/<uuid>.json`

### Global flags

`--json --verbose --quiet --config --dry-run --offline` plus `--help` / `--version`.

## What is collected

| Field | Source |
|-------|--------|
| Hostname | `/etc/hostname` / `hostname` |
| Interfaces, MACs, local IPs | sysfs + `ip` |
| Public IP | `curl` to ipify/ifconfig.me (disable: `--offline`) |
| Gateway / routes | `ip route` |
| VPN | mullvad / wg / tailscale heuristics |
| DNS | `/etc/resolv.conf` |
| Listening ports | `ss` or `/proc/net/*` |
| Firewall | `nft` / `iptables` if readable |
| Wi-Fi SSID | `nmcli` / `iw` |
| mDNS / UPnP | avahi socket, :5353, :1900, miniupnpd |

Secrets in free-form fields are redacted.

## Architecture

```
src/collect/  # gatherers
src/diff.rs   # snapshot comparison
src/sanitize.rs
src/commands/
```

## License

MIT © r3dg0d
