# Repository Guidelines

## Project Structure & Module Organization
- Root crate: Rust 2021 binary using `eframe/egui` + `wgpu`.
- Source in `src/` with `main.rs` and modules like `shader_pipeline.rs`, `wgsl_highlight.rs`, `toast.rs`. Optional binaries in `src/bin/`.
- Experiments live in `ideas/` (separate Cargo project). Run with `cd ideas && cargo run`.
- Tests: unit tests inline via `#[cfg(test)]` or integration tests in `tests/`.
- Build artifacts in `target/` (not committed).

## Build, Test, and Development Commands
- Build: `cargo build` — compiles the app.
- Run: `cargo run` — launches the UI; add logs with `RUST_LOG=info cargo run`.
- Test: `cargo test` — all tests; single test with `cargo test <name>` or `cargo test <module>::<fn>`.
- Debug tests: `RUST_BACKTRACE=1 cargo test -- --nocapture`.
- Format: `cargo fmt` before pushing.
- Lint: `cargo clippy --all-targets -- -D warnings`.

## Coding Style & Naming Conventions
- Rust 2021 idioms; 4‑space indent (rustfmt default).
- Imports grouped: std, external crates, then local modules; avoid glob `*`.
- Names: `snake_case` for functions/vars, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for consts.
- Errors: use `Result<T, E>` + `?`; prefer `anyhow`/`thiserror` internally. Avoid `unwrap`/`expect` outside tests and init.

## Testing Guidelines
- Use Rust’s built‑in test framework. Keep tests small, deterministic, and fast.
- Avoid GPU/windowing in unit tests; isolate or mock logic around `wgpu` and UI.
- Place unit tests near code under `#[cfg(test)]`; integration tests in `tests/`.
- Name tests clearly by behavior; run subsets with `cargo test <pattern>`.

## Commit & Pull Request Guidelines
- Commits: imperative, scoped messages (e.g., "Add WGSL syntax highlight tokenization"). One concern per commit.
- PRs: describe what/why, link issues, and include screenshots for UI changes. Call out breaking changes and manual steps.

## Agent‑Specific Instructions
- Modify files only within this repo. After API/logic changes, run `cargo test`.
- When adding dependencies, update `Cargo.toml` then run `cargo build` and `cargo test`.
- Ask before broad refactors; keep changes minimal and consistent with existing style.

