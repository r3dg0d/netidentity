# Contributing

```bash
cargo test
cargo build --release
cargo run -- snapshot --offline --dry-run
```

- Keep redaction conservative (false positives OK; leaking secrets is not)
- Add unit tests for parsers (`/proc` addresses, diff)
- `cargo fmt` + `clippy` before PR
