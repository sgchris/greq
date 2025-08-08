# GitHub Copilot Instructions for This Rust Project

Welcome to this Rust project. When using GitHub Copilot, please follow the guidelines below to ensure the code it generates aligns with the project's goals: **clean, idiomatic, efficient, and well-structured Rust code.**

## ✨ General Principles

- Write **idiomatic Rust**: Prefer expressive constructs, clear ownership patterns, and match arms over verbose or complex logic.
- Code must be **human-readable** and **self-explanatory**.
- Prefer **composition over inheritance**, and **immutability over mutability**.
- Optimize for **clarity first**, then for **performance**, unless performance is a critical goal in the specific module.

---

## 📐 Project Design & Structure

- Follow **modular design**: Use multiple small modules (`mod`) instead of large monoliths.
- Keep modules **cohesive and loosely coupled**.
- Organize code using `lib.rs` / `main.rs` and feature-based or domain-driven folder structure (`src/domain/`, `src/utils/`, etc.).
- Avoid deeply nested modules or overly complex generics.

---

## 📦 Crates & Dependencies

- Use crates from [crates.io](https://crates.io) only when:
  - They are **well-maintained and documented**.
  - They solve the problem better than in-house implementation.
- Avoid over-engineering: Don’t include unnecessary dependencies.

---

## 🧠 Naming & Readability

- Use **clear, descriptive, and consistent names**.
  - Example: `fn calculate_checksum` instead of `fn calc_chksm`
- Prefer `snake_case` for functions/variables, `CamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants.
- Comment *why*, not *what*. Rust is expressive enough for the “what”.

---

## 🛠️ Code Style & Idioms

Use the following idiomatic Rust patterns:

- Prefer `Result<T, E>` over `Option<T>` when errors are possible.
- Use `?` operator instead of `match` unless additional logic is needed.
- Use `match` or `if let` over `unwrap`, `expect`, or panics unless it's truly unrecoverable.
- Use `iterators`, `map`, `filter`, and combinators where they improve clarity.
- Avoid cloning unless necessary—respect ownership and borrowing.
- Use `enum` and `match` to represent states clearly.
- Keep functions small and focused (preferably < 50 lines).

---

## ✅ Testing & Safety

- Write **unit tests** and **integration tests** for critical functionality.
- Use `#[test]` and organize tests in `mod tests` or `/tests/`.
- Use `assert_eq!`, `assert_matches!`, and error pattern testing.
- Avoid unsafe code unless absolutely necessary and well-justified.

---

## 🧪 Examples

Good patterns:
```rust
fn read_config(path: &Path) -> Result<Config, ConfigError> {
    let content = fs::read_to_string(path)?;
    toml::from_str(&content).map_err(ConfigError::Parse)
}
```

Bad patterns:
```rust
fn read_config(path: &str) -> Config {
    let content = fs::read_to_string(path).unwrap();
    toml::from_str(&content).unwrap()
}
```

## 🔧 Tooling
- Use rustfmt for consistent formatting.
- Use clippy to detect common mistakes and enforce linting rules.
- Use cargo check and cargo test regularly.

## 📚 Documentation
- Document all public functions, structs, enums, and modules with /// doc comments.
- Add examples when usage is non-obvious.
- Use //! for module-level explanations.

## 📈 Performance
- Only optimize after profiling (e.g., with perf, cargo flamegraph, or criterion).
- Avoid premature optimization unless writing performance-critical code (e.g., parsers, serializers).

## 🧹 Clean Code Commitments
- Avoid commented-out code blocks.
- Remove unused imports or variables.
- Prefer expressive errors and avoid leaking implementation details.
- Keep dependencies and features lean (Cargo.toml).


## Implementation details for AI

Implement the project in Rust language. Make it high quality, readable, understandable and efficient. Use Rust patterns and idioms.

Add tests whenever needed to make sure that all the important parts are robust and bugs free.


Please update cargo.toml with all the necessary libraries at their most up-to-date stable versions.

Use the most common library for logging and store all the detailed logs including the debug lines under AppData/Local (or equivalent in other OSs). The console should show only the important and relevant parts.

Make the console output of the app readable, structured, clear and understandable. add basic colors and display only valuable information. Log when you load base request or dependant request

All the code must be used. Don't add flags that allow keeping unused code.

Add a description above every method, struct or enum.

Make sure the build is successful without errors and warnings. Fix everything until it's compiling successfully.

You have full permissions to use `cargo build` and `cargo test`.

Parts of the app that may be executed in parallel should be executed in parallel.

At some point in the future, there will be extensions to this app. Make sure you write the code accordingly, and it won't require significant refactor once the extensions are added.
