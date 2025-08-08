# Contributing to greq

Thanks for your interest in contributing! This guide covers how to build, test, and submit changes to this Rust project.

## Quick start

- Rust toolchain: install from https://rustup.rs (stable channel recommended)
- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Format: `cargo fmt --all`

## Project layout

- `src/` — main library and binary sources
- `greq-examples/` — sample `.greq` files
- `README.md` — overview and usage

Run the CLI locally (example):

```sh
cargo run --release -- .\greq-examples\tests_with_dependency\base_greq_files\01-simple-get.greq
```

For playground, testing or just for fun, it's recommended to check out [Grecho - echo server](https://github.com/sgchris/grecho), or (GResources - Simple RESTFul API for resources management)[https://github.com/sgchris/gresources].


## How to contribute

### 1) Report bugs and request features

- Search existing issues before filing a new one
- Provide steps to reproduce, expected vs. actual behavior, and environment details

### 2) Improve documentation

- Fix typos, clarify explanations, add examples
- Keep README and examples accurate with code behavior

### 3) Submit code changes

1. Fork the repo and create a topic branch from the default branch
   - Example: `feat/placeholder-syntax`, `fix/headers-parsing`
2. Make small, focused commits with clear messages (Conventional Commits encouraged)
3. Ensure checks pass locally:
   - `cargo fmt --all`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test`
4. Add/adjust tests for changed behavior
5. Open a Pull Request (PR)
   - Describe the motivation, approach, and trade-offs
   - Link related issues; include screenshots/logs when useful

### Code style & quality

- Follow `rustfmt` defaults (use `cargo fmt`)
- Keep the code warning-free under `clippy` with `-D warnings`
- Prefer explicit types, clear error messages, and small functions
- Handle errors using `Result` and project error types; avoid panics in library code
- Add doc comments to public items when helpful

### Testing

- Write unit tests for core logic (happy path + edge cases)
- Prefer small, deterministic tests; avoid external network calls unless behind feature flags or integration tests
- Run `cargo test` before submitting PRs

### Commit messages

- Use present tense, imperative mood ("Add X", not "Added X")
- Include a short summary (<= 72 chars) and optional details in the body
- Conventional Commits (optional but welcome): `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`

### Security

- Do not commit secrets (API keys, tokens)
- Report sensitive security issues privately via a maintainer contact or by opening a minimal, non-disclosing issue asking for contact

### Releases

- Maintainers handle versioning and publishing
- Use semantic versioning; update `CHANGELOG.md` (if present) and `Cargo.toml` when releasing

## Local development tips

- Run a single test: `cargo test <test_name> -- --nocapture`
- Faster builds for iteration: `cargo build` (debug) and `RUST_LOG=info` for richer logs

## License

By contributing, you agree that your contributions will be licensed under the terms of the repository’s [MIT License](./LICENSE).
