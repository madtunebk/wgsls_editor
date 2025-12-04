# Repository Guidelines

## Project Structure & Modules
- Root crate: Rust 2021 binary using `eframe/egui` and `wgpu`.
- Source: `src/` with `main.rs` plus modules `shader_pipeline.rs`, `wgsl_highlight.rs`, `toast.rs`. Optional binaries in `src/bin/`.
- Experiments: `ideas/` holds standalone examples and WGSL assets (separate Cargo project). Build by `cd ideas && cargo run`.
- Artifacts: `target/` is Cargo output; do not commit.

## Build, Test, and Development
- Build app: `cargo build`.
- Run app: `cargo run` (add logging with `RUST_LOG=info cargo run`).
- Test all: `cargo test`; single test: `cargo test -- <name>` or `cargo test <module>::<fn>`.
- Debug tests: `RUST_BACKTRACE=1 cargo test -- --nocapture`.
- Format: `cargo fmt` before pushing.
- Lint: `cargo clippy --all-targets -- -D warnings`.

## Coding Style & Naming
- Rust 2021 idioms; 4-space indent (rustfmt default).
- Imports: group `std`, external crates, then local modules; avoid `*` globs.
- Naming: `snake_case` functions/vars, `CamelCase` types, `SCREAMING_SNAKE_CASE` consts.
- Errors: use `Result<T, E>` + `?`; prefer `anyhow`/`thiserror` internally. Avoid `unwrap/expect` outside tests and init.

## Testing Guidelines
- Use Rust’s built-in test framework. Place unit tests in the same file under `#[cfg(test)]` or add integration tests in `tests/`.
- Keep tests small, deterministic, and fast; avoid GPU/windowing in unit tests. Mock logic around `wgpu` where possible.

## Commit & PR Guidelines
- Commits: imperative, scoped messages (e.g., "Add WGSL syntax highlight tokenization"). One concern per commit.
- PRs: describe what/why, link issues, and include screenshots for UI changes. Call out breaking changes and manual steps.

## Agent-Specific Instructions
- Modify files only within this repo. After API/logic changes, run `cargo test`.
- When adding dependencies, update `Cargo.toml` then run `cargo build` and `cargo test`.
- Ask before broad refactors; keep changes minimal and consistent with existing style.
