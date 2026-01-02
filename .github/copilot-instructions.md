# Copilot Instructions for chord-script

## Project Overview
A Rust CLI for generating clean, chord-first music charts focused on form and structure.

## Tech Stack
- **Language**: Rust (see `.gitignore` for Cargo setup)
- **Type**: Command-line application
- **License**: MIT

## Project Status
This is a greenfield project. No source code exists yet.

## Getting Started (Once Implemented)
```bash
cargo build          # Build the project
cargo run            # Run the CLI
cargo test           # Run tests
cargo fmt            # Format code
cargo clippy         # Lint code
```

## Conventions to Follow

### Rust Patterns
- Use `clap` for CLI argument parsing
- Prefer `thiserror` for custom error types
- Use `Result<T, E>` for fallible operations, avoid panics in library code
- Follow Rust API guidelines: https://rust-lang.github.io/api-guidelines/

### Input Format
- Custom plain-text DSL (inspired by/improving on chordsheet.com)
- File extension: `.cchart`
- Parser uses `nom` for combinator-based parsing
- Store example `.cchart` files in `examples/` for testing and documentation

### Output Formats
- **SVG**: Vector output, scalable for any size (primary/intermediate format)
- **PNG**: Raster output via `resvg` (SVG → PNG)
- **PDF**: Print-ready via `svg2pdf` (SVG → PDF)

Rendering pipeline: Parse → Model → SVG → (PNG | PDF)

### Architecture Layers
```
┌─────────────┐
│  .cchart    │  Input: plain text DSL
└──────┬──────┘
       │ nom parser
       ▼
┌─────────────┐
│   Model     │  Domain types: Chart, Section, Chord, Form
└──────┬──────┘
       │ render
       ▼
┌─────────────┐
│    SVG      │  Intermediate representation (always generated)
└──────┬──────┘
       │ convert (optional)
       ▼
┌─────────────┐
│ PNG or PDF  │  Final output (resvg / svg2pdf)
└─────────────┘
```

Each layer is independent:
- **Parser** (`parser/`): Text → Model, no knowledge of rendering
- **Model** (`model/`): Pure domain types, no I/O
- **Render** (`render/`): Model → SVG, format conversion

### Project Structure (Recommended)
```
src/
├── main.rs          # CLI entry point
├── lib.rs           # Library root (core logic)
├── parser/          # DSL parser (nom-based)
│   ├── mod.rs
│   ├── lexer.rs     # Tokenization
│   └── ast.rs       # Abstract syntax tree types
├── model/           # Domain types (Chord, Section, Chart)
└── render/          # Output formatting (SVG, PNG, PDF)
    ├── mod.rs
    ├── svg.rs       # SVG generation
    ├── png.rs       # PNG rasterization
    └── pdf.rs       # PDF export
examples/            # Sample .cchart files demonstrating DSL syntax
```

### Domain Concepts
- **Chart**: Complete song structure
- **Section**: Named parts (Verse, Chorus, Bridge)
- **Chord**: Musical chord notation (e.g., Cmaj7, Dm7)
- **Form**: High-level song structure (e.g., AABA)

## AI Agent Notes
- When adding features, create small focused modules
- Write unit tests alongside new functionality
- Keep the CLI interface simple and Unix-philosophy friendly
- Do not add any features or do any work until explicitly instructed to do so.
