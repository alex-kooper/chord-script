# Agent instructions for chord-script

Rust CLI/library for clean, chord-first music charts focused on form and structure.

## Architecture

Parse → Model → SVG → (PNG | PDF, planned)

| Layer | Path | Responsibility |
|-------|------|----------------|
| Parser | `src/parser/` | Text → `Chart`; no rendering |
| Model | `src/model/` | Pure domain types; no I/O |
| Render | `src/render/` | `Chart` → SVG |

Layers must stay independent. Parser and render must not depend on each other.

## Coding standards

- Files should not exceed 300 lines. Split large files into smaller modules with meaningful names.
- Functions/methods should not exceed 50 lines. Extract helpers with clear, descriptive names.
- Prefer domain-specific types over primitives (`String`, `usize`, etc.), especially in the model layer. Use newtypes, enums, and structs to make invalid states unrepresentable.
- Use [`derive_more`](https://docs.rs/derive_more) to reduce boilerplate on structs and enums (`From`, `Display`, `Into`, `AsRef`, etc.).
- Use [`nutype`](https://docs.rs/nutype) for validated newtypes with sanitization and constraints (e.g. non-empty strings).

### Error handling

Use typed errors in library code; use `anyhow` only at the application boundary.

**Library code — `thiserror`:** Each module owns its error types; expose `pub type Result<T>`.
- Parser may also use `miette` for source spans and pretty reporting

**Application boundary — `anyhow`:** Binaries and examples only.
- `fn main() -> anyhow::Result<()>`
- Attach context with `.context("...")?` for I/O and CLI boundaries

**Do not:**
- Use `anyhow` inside `src/model/`, `src/parser/`, or `src/render/`
- Use `unwrap()` / `expect()` on user input or I/O in library code
- Panic for recoverable or expected failure modes
