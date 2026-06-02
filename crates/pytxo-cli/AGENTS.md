# pytxo-cli

Thin CLI only: parse arguments, load config, wire crates.

- No scheduling logic — use `pytxo-scheduler`.
- No git/process code — use `pytxo-runner`.
- No SQL — use `pytxo-store`.

Build: `cargo build -p pytxo-cli`  
Run: `cargo run -p pytxo-cli -- <subcommand>`
