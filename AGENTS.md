# Repository Guidelines

## Build / Run / Test
- Build: `cargo build` (debug) or `cargo build --release`.
- Run app: `cargo run` (use `RUST_LOG=info cargo run` for logs).
- Run all tests: `cargo test`.
- Run a single test: `cargo test <module>::<test_name>` (or use `cargo test <pattern>`).
- Debug tests: `RUST_BACKTRACE=1 cargo test -- --nocapture`.
- Format: `cargo fmt` (project uses rustfmt defaults).
- Lint: `cargo clippy --all-targets -- -D warnings`.

## Code Style & Conventions
- Edition: Rust 2021; follow `rustfmt` formatting (4-space indent by default).
- Imports: group and order as `std`, external crates, then local modules; avoid `use *` glob imports.
- Naming: `snake_case` for functions/variables, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Error handling: return `Result<T, E>` and propagate with `?`; prefer `anyhow` or `thiserror` for application errors; avoid `unwrap`/`expect` except in tests or early initialization.
- Mutability: prefer immutable bindings; only mark `mut` when necessary.
- Types: prefer explicit types on public APIs; use `impl Trait` for return-position where appropriate.
- Tests: keep unit tests small and deterministic; avoid GPU or windowing in unit tests (mock or extract logic).

## Commits & PRs
- Commit messages: imperative, scoped (e.g., "Add WGSL syntax highlight tokenization").
- PRs: explain why, link issues, include screenshots for UI changes.

## Agent Notes
- Agents must only modify files within this repo; run `cargo test` after API/logic edits.
- When adding dependencies, update `Cargo.toml` and run `cargo build` + `cargo test`.

(If you have repository-specific Cursor or Copilot rules, add them here; none were found.)