# Repository Guidelines

## Project Structure & Module Organization
- `src/primitives/`: OpenGL wrappers (`shader.rs`, `buffer.rs`, `texture.rs`).
- `src/renderer/`: High-level drawing (`quad.rs`).
- `src/ui/`: UI widgets (`button.rs`, `checkbox.rs`, `label.rs`, `textbox.rs`, `panel.rs`).
- `examples/demo.rs`: Showcases all components.
- `tests/ui_components.rs`: Integration tests for widget logic.
- `Cargo.toml`: Crate metadata and dependencies.

## Build, Test, and Development Commands
- Build library: `cargo build` (use `--release` for optimized builds).
- Run tests: `cargo test` (add `-v` for verbose output).
- Lint with Clippy: `cargo clippy -- -D warnings`.
- Format code: `cargo fmt` (run before committing).
- Run example: `cargo run --example demo`.
  - Note: the demo requires GLFW and system X11 libs on Linux. Install: Ubuntu `sudo apt-get install libx11-dev libxrandr-dev libxi-dev`.

## Coding Style & Naming Conventions
- Rust 2021 edition; follow `rustfmt` defaults (4-space indent, 100-line soft wrap).
- Naming: modules/files `snake_case`; types/traits `PascalCase`; functions/variables `snake_case`; constants `SCREAMING_SNAKE_CASE`.
- Prefer explicit types at public boundaries; keep modules small and focused.
- Use `clippy` to catch smells; fix or justify with `#[allow(...)]` locally.

## Testing Guidelines
- Framework: `cargo test` (integration tests in `tests/`).
- Tests should assert state/interaction behavior without requiring an OpenGL context.
- Name tests descriptively (e.g., `button_pressed_updates_state`).
- Add tests alongside features; keep helpers in `tests/` or private `mod tests` blocks for unit cases.

## Commit & Pull Request Guidelines
- Commits: imperative mood, concise subject (≤72 chars), include rationale in body when needed.
  - Examples: `Add QuadRenderer outline drawing`, `Fix checkbox hit detection`.
- PRs: clear description, link issues (`Closes #123`), outline changes and impact, include screenshots/GIFs for visual behavior when relevant.
- Ensure `cargo fmt`, `cargo clippy`, and `cargo test` pass before requesting review.

## Security & Configuration Tips
- Examples may require GLFW; ensure it’s added as a dev dependency and platform libs are installed.
- Avoid panics in library code; return `Result` where failures are possible.
