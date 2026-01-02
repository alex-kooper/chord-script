# Chord Script DSL Specification

**Status:** Draft v0.1  
**Date:** December 29, 2025

---

## Overview

Chord Script uses a custom plain-text DSL (`.cchart` files) for defining music charts. The format is inspired by [chordsheet.com](https://chordsheet.com) but adds Markdown-like text formatting with flexible alignment.

### Design Goals

1. **Expressive text formatting** — not fixed fields, users design their own layout
2. **Markdown-inspired** — familiar feel, optimized for charts
3. **Control over:** size/weight, alignment (left/center/right)
4. **Concise chord syntax** — borrowed from chordsheet.com

---

## Text Lines

Text lines start with `=` characters. The number of `=` determines the weight (size).

### Weight (Line Prefix)

| Syntax | Weight | Typical Use |
|--------|--------|-------------|
| `===` | H1 (largest) | Song title |
| `==` | H2 (medium) | Artist, subtitle |
| `=` | H3 (smallest) | Section names, annotations |

### Alignment (Inline Markers)

Alignment markers appear **after** the weight prefix, **before** the text content.

| Syntax | Alignment |
|--------|-----------|
| `<<text` | Left |
| `>>text` | Right |
| `<<>>text` | Center |
| `text` (no marker) | Left (default) |

**Why double characters?** Single `<` and `>` appear in chord notation (push, accent, dynamics). Using `<<`, `>>`, `<<>>` avoids conflicts and reduces escaping needs.

### Examples

```
=== <<>>Rolling in the Deep     (H1, centered)
== <<>>Adele                    (H2, centered)
= <<>>2011 • *21*               (H3, centered)

= Verse 1                       (H3, left - default)
= >>page 1 of 2                 (H3, right)
```

### Multi-Zone Lines

A single line can have multiple alignment zones:

```
= <<Left Text <<>>Centered >>Right
```

Renders as:
```
Left Text           Centered           Right
```

Useful for headers/footers:
```
= <<transcribed by @alex >>page 1
```

### Rule: One Weight Per Line

You cannot mix weights on a single line. Use bold/italic for inline emphasis instead:

```
= Key: Am >>**Adele**           (same weight, bold for emphasis)
```

---

## Inline Formatting

Markdown-style inline formatting within text lines:

| Syntax | Result |
|--------|--------|
| `*italic*` | *italic* |
| `**bold**` | **bold** |
| `***bold italic***` | ***bold italic*** |

### Escaping

Use backslash to escape special characters:

| Literal | Escape |
|---------|--------|
| `*` | `\*` |
| `\` | `\\` |

Note: `<` and `>` rarely need escaping since alignment uses double characters (`<<`, `>>`).

---

## Chord Lines

Any line that does **not** start with `=` is treated as a chord line.

### Basic Chord Syntax (from chordsheet.com)

| Syntax | Meaning |
|--------|---------|
| `Am`, `Cmaj7`, `F#m7b5` | Chord names |
| `_` | Beat/subdivision separator |
| `,` | Empty beat / rest |
| `*` | Repeat previous chord |
| `%` | Repeat previous bar |
| `( ) Nx` | Repeat group N times |
| `1.` `2.` | First/second endings |
| `<Chord` | Push (anticipation) |
| `<>` | Accent/stab |
| `?` suffix | Ghost/optional chord |
| `N.C.` | No chord |
| `fermata` | Hold |
| `"text"` | Inline chord annotation |

### Chord Line Examples

```
Am %                              (Am for one bar, repeat)
Am_G_F_G                          (four chords, one per beat)
Am,,, ,<Em,, ,<G,, Em, _ G,       (complex rhythm with pushes)
(Am G F F _ G) 4x                 (repeat group 4 times)
(F G Em  1. F  2. E)              (with endings)
Am <> G <> F                      (accented chords)
Am? Dm?                           (ghost/optional chords)
(Am G F F _ G) 4x Am fermata      (ending with fermata)
```

---

## Complete Example

```
=== <<>>Rolling in the Deep
== <<>>Adele
= <<>>2011 • *21*

= Intro
Am %

= Verse 1
(Am,,, ,<Em,, ,<G,, Em, _ G,) 4x

= Pre-Chorus
(F G Em  1. F  2. E)

= Chorus
(Am G F F _ G)

= Verse 2
(Am,,, ,<Em,, ,<G,, ,<Em _ ,<G) 2x

= Pre-Chorus 2
(F G Em  1. F  2. E)

= Chorus 2
(Am G F F _ G)

= Interlude
F     G Am     G
F %  G %

= Verse 3 >>N.C.
(Am?,,, ,<Em?,, ,<G?,, Em?, _ G?,) 2x

= Chorus >>**build up**, no drums
(Am <> G <> F <> F _ G) 2x

= Chorus 3
(Am G F F _ G) 4x Am fermata

= <<transcribed by @alex >>page 1
```

---

## Design Decisions Log

This section captures the reasoning behind key decisions made during DSL design.

### Why not standard Markdown headings?

Standard Markdown uses `#` for largest, `###` for smaller. For music charts, you'd mostly use the smallest level (annotations), requiring `###` everywhere. That's verbose.

We inverted the intuition: fewer `=` = smaller text, which matches frequency of use.

### Why `=` instead of `-` or `#`?

- `=` feels "structural" and visually balanced
- `-` is used in chordsheet.com for text lines (potential confusion)
- `#` has strong Markdown associations that conflict

### Why double characters for alignment (`<<`, `>>`, `<<>>`)?

Single `<` and `>` appear naturally in chord notation:
- `<Em` = push/anticipation
- `<>` = accent
- `>` = decrescendo

Using double characters (`<<`, `>>`) avoids constant escaping in chord lines.

### Why allow multiple alignment zones per line?

Common use cases:
- `= <<Left Info >>Right Info` — header/footer layouts
- `= <<Key: Am <<>>Title >>Page 1` — three-column headers

### Why one weight per line?

Simplicity. Mixed weights would complicate:
- Parsing
- Rendering (vertical alignment of different sizes)
- Source readability

Use `**bold**` for inline emphasis instead.

### Text lines vs chord lines

Any line starting with `=` is a text line. Everything else is chords. Simple, unambiguous.

---

## Grammar (Informal)

```
document     = line*
line         = text_line | chord_line | blank_line

text_line    = weight SP alignment_zone+
weight       = "===" | "==" | "="

alignment_zone = alignment? text_content
alignment    = "<<>>" | "<<" | ">>"
text_content = (formatted_text | plain_char)*

formatted_text = bold_italic | bold | italic
bold_italic    = "***" text "***"
bold           = "**" text "**"
italic         = "*" text "*"

chord_line   = chord_element (SP chord_element)*
               (repeat_marker)?
               
# Chord syntax TBD - largely follows chordsheet.com
```

---

## Future Considerations

- **Metadata:** Key, tempo, time signature — as text lines or special syntax?
- **Form notation:** AABA structure markers?
- **Rendering pipeline:** Parse → Model → SVG → PNG/PDF
- **Editor support:** Syntax highlighting for `.cchart` files

---

## References

- [chordsheet.com](https://chordsheet.com) — inspiration for chord syntax
- [ChordPro](https://www.chordpro.org/) — another music notation format
- Markdown — inspiration for inline formatting
