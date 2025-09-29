# AGENTS for scrtsync
- Build: `cargo build` (release: `cargo build --release`)
- Test all: `cargo test`
- Run single test: `cargo test <pattern>` (e.g., `cargo test to_writer`)
- Lint: `cargo clippy -- -D warnings`
- Format check: `cargo fmt --all -- --check` (fix: `cargo fmt --all`)
- Run CLI help: `cargo run -- --help`
- CI: stable Rust; runs `check`, `test` (macOS+Linux), `fmt`, `clippy`.

## Code Style
- Formatting: rustfmt defaults; LF line endings + final newline (EditorConfig).
- Imports: no wildcards; group std/third-party/local; use braces (e.g., `use anyhow::{Context, Result};`).
- Types: use `anyhow::Result` + `?`; prefer `BTreeMap` for deterministic ordering; own `String` unless borrowing is obvious.
- Errors: avoid `unwrap/expect` outside tests; add `with_context(|| "...")` to fallible ops; return user-friendly messages.
- Naming: modules snake_case; types/traits CamelCase; functions snake_case; constants SCREAMING_SNAKE_CASE.
- Traits: prefer `Box<dyn Trait>` where trait objects are passed across boundaries (e.g., sources).
- IO: accept generic readers/writers (`T: Read`/`Write`) for flexibility and testability.
- URLs/validation: parse with `url::Url`; validate inputs early (config and args).
- Tests: colocate with modules under `#[cfg(test)]`; keep deterministic assertions and no external side effects.
- Cursor/Copilot: none found in repo; no additional editor rules apply.