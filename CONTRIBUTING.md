# Contributing

Thank you for contributing! This project is a Rust `egui`/`wgpu` app. Please read the Repository Guidelines first: see AGENTS.md.

## Getting Started
- Prereqs: Rust stable (rustup), Cargo.
- Build: `cargo build`
- Run: `cargo run` (enable logs: `RUST_LOG=info cargo run`)
- Tests: `cargo test` (single: `cargo test -- <name>`)

## Development Flow
1. Create a feature branch from `main`.
2. Make focused changes; keep commits small and scoped.
3. Before opening a PR, run locally:
   - `cargo fmt`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

## Code Style
- Follow the conventions in AGENTS.md (Rust 2021 idioms, import grouping, naming).
- Avoid `unwrap/expect` outside tests or explicit init paths.
- Return `Result<T, E>` and propagate with `?`.

## Tests
- Add unit tests near code under `#[cfg(test)]`.
- For integration tests, use `tests/` when it adds value.
- Keep tests deterministic and fast; avoid GPU/windowing in unit tests.

## Dependencies
- Update `Cargo.toml` thoughtfully; prefer well-maintained crates.
- After adding/updating deps: `cargo build && cargo test`.

## Pull Requests
- Title: imperative and scoped (e.g., "Add WGSL syntax highlighting").
- Description: what changed, why, and any trade-offs.
- UI changes: include screenshots or short clips.
- Note breaking changes and manual migration steps.

## Project Layout Notes
- App code: `src/` (`main.rs`, `shader_pipeline.rs`, `wgsl_highlight.rs`, `toast.rs`).
- Optional bins: `src/bin/`.
- Experiments: `ideas/` (separate Cargo project) — build with `cd ideas && cargo run`.

## Issues
- When filing, include OS, Rust version, repro steps, and logs (`RUST_LOG=debug`).
