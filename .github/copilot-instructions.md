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
- File extension: `.charts`
- Parser uses `Chumsky` for combinator-based parsing
- Store example `.charts` files in `examples/` for testing and documentation

### Output Formats
- **SVG**: Vector output, scalable for any size (primary/intermediate format)
- **PNG**: Raster output via `resvg` (SVG → PNG)
- **PDF**: Print-ready via `svg2pdf` (SVG → PDF)

Rendering pipeline: Parse → Model → SVG → (PNG | PDF)

### Architecture Layers
```
┌─────────────┐
│  .charts    │  Input: plain text DSL
└──────┬──────┘
       │ Chumsky parser
       ▼
┌─────────────┐
│   Model     │  Domain types: Lines with text or chords
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

## AI Agent Notes
- When adding features, create small focused modules
- Write unit tests alongside new functionality
- Keep the CLI interface simple and Unix-philosophy friendly
- Do not add anything without approval
