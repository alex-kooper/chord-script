# clean-chart

A CLI for clean, chord-first charts focused on form and structure

Generate beautiful chord charts in SVG, PNG, and PDF formats from simple text files. Inspired by iRealPro, clean-chart creates professional-looking chord charts that focus on the essential harmonic and structural information.

## Features

- 📝 Simple text-based `.cchart` format for easy editing
- 🎨 Generate chord charts in SVG, PNG, and PDF formats
- 🎵 Support for sections (Verse, Chorus, Bridge, etc.)
- 🎹 Flexible chord notation (major, minor, 7ths, extensions, etc.)
- 📐 Clean, readable layout similar to iRealPro

## Installation

### From Source

```bash
cargo build --release
```

The binary will be available at `target/release/clean-chart`.

## Usage

```bash
clean-chart [OPTIONS] <FILE>

Arguments:
  <FILE>  Input .cchart file

Options:
  -f, --format <FORMAT>  Output format [default: svg] [possible values: svg, png, pdf]
  -o, --output <OUTPUT>  Output file (defaults to input name with appropriate extension)
  -h, --help             Print help
  -V, --version          Print version
```

### Examples

Generate an SVG chart (default):
```bash
clean-chart examples/simple.cchart
```

Generate a PNG chart:
```bash
clean-chart examples/simple.cchart -f png
```

Generate a PDF chart with custom output path:
```bash
clean-chart examples/blue_bossa.cchart -f pdf -o output/chart.pdf
```

## .cchart File Format

The `.cchart` format is a simple, human-readable text format for defining chord charts.

### Basic Structure

```
Title: Song Name
Composer: Composer Name
Key: C
Time: 4/4

[Section Name]
| Chord1 | Chord2 | Chord3 | Chord4 |
```

### Metadata

- `Title:` - Song title (required)
- `Composer:` - Composer or artist name (optional)
- `Key:` - Key signature (optional)
- `Time:` - Time signature (optional, defaults to 4/4)

### Sections

Sections are defined with square brackets:
```
[Verse]
[Chorus]
[Bridge]
[A Section]
[B Section]
```

### Measures

Measures are separated by `|` (pipe) characters. Each measure can contain:

- **One chord** (whole measure): `| Cmaj7 |`
- **Two chords** (half notes each): `| Cmaj7 Dm7 |`
- **Four chords** (quarter notes each): `| C Dm Em F |`
- **Repeat previous measure**: `| - |`
- **Empty measure**: `| |` or `| % |`

### Chord Notation

Chords follow standard jazz notation:
- Root note: `C`, `D`, `E`, `F`, `G`, `A`, `B`
- Accidentals: `C#`, `Db`, `F#`, `Bb`
- Qualities: `m` (minor), `maj7`, `m7`, `7`, `dim`, `aug`, `sus4`, etc.

Examples:
- `C` - C major
- `Cm` - C minor
- `Cmaj7` - C major 7th
- `Dm7` - D minor 7th
- `G7` - G dominant 7th
- `Am7b5` - A minor 7 flat 5 (half-diminished)
- `F#m` - F# minor

### Complete Example

```
Title: Blue Bossa
Composer: Kenny Dorham
Key: Cm
Time: 4/4

[A Section]
| Cm6 | - | Fm7 | - |
| Dm7b5 | G7b9 | Cm6 | - |

[B Section]
| Ebm7 | Ab7 | Dbmaj7 | - |
| Dm7b5 | G7b9 | Cm6 | - |
```

See the `examples/` directory for more examples.

## Output Formats

### SVG
- Vector format, scalable without quality loss
- Can be edited in vector graphics software
- Best for web display

### PNG
- Raster image format
- 800x1000 pixels (default)
- Best for sharing/embedding

### PDF
- Document format with embedded image
- Best for printing
- Professional appearance

## License

See the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

