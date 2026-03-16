# SecretSync

A CLI utility for syncing secrets between Vault, local .env files, and Kubernetes secrets.
Provides a unified interface for managing secrets across multiple sources with support for
diffs and preset configurations.

## Repo organization

Root
├─ /src/config.rs: the core domain design
├─ /src/secrets.rs: the core domain design
├─ /src/job/: Job execution and orchestration
└─ /src/sources/: Secret source implementations (Vault, File, Kubernetes)

## Verifying (MUST BE RUN BEFORE CONSIDERING A TASK COMPLETE)

- `cargo fmt --all -- --check`
- `cargo test --all-features`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-features`
