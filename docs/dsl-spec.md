# Chord Script DSL Specification

**Status:** Draft v0.1  
**Date:** December 29, 2025

---

## Overview

Chord Script uses a custom plain-text DSL (`.cchart` files) for defining music charts. The format is inspired by [chordsheet.com](https://chordsheet.com) but adds Typst-like text formatting with flexible alignment.

### Design Goals

1. **Expressive text formatting** — not fixed fields, users design their own layout
2. **Markdown/Typst-inspired** — familiar feel, optimized for charts
3. **Control over:** size/weight, alignment (left/center/right)
4. **Concise chord syntax** — borrowed from chordsheet.com

---

## Text Lines

Text lines start with a weight marker: `===`, `==`, `=`, or `-`. The more `=`, the larger the text; `-` is plain text.

### Weight (Line Prefix)

| Syntax | Weight | Typical Use |
|--------|--------|-------------|
| `===` | H1 (largest) | Song title |
| `==` | H2 (medium) | Artist, subtitle |
| `=` | H3 (smallest) | Section names, annotations |
| `-` | Text | Comments, annotations |

### Line Rules

- Every line ends at the line break; nothing continues onto the next line.
  `\n` and `\r\n` are the usual breaks; a lone `\r`, form feed, vertical tab,
  and the Unicode line/paragraph separators also end a line.
- The weight marker must start in column 0 and be followed by a space.
  `===Title` and indented markers are errors; `==` in the middle of a line is plain text.
- A bare marker (`===`, `==`, `=`, `-`) is an empty line of that weight's height,
  so `===` leaves more space than `=`.
- Blank lines (empty or whitespace-only) are ignored, like whitespace in code.

### Alignment: `LEFT <CENTER> RIGHT`

After the weight prefix, a line has up to three columns. The centered text is
wrapped in `<` `>`; text before it is left-aligned and text after it is
right-aligned. The center is optional and appears at most once. `<>` is an
empty center: it centers nothing but still separates left from right.

| Want | Syntax |
|------|--------|
| Left only | `Left text` |
| Center only | `<Centered text>` |
| Right only | `<> Right text` |
| Left + right | `Left <> Right` |
| Left + center | `Left <Center>` |
| Center + right | `<Center> Right` |
| All three | `Left <Center> Right` |

Spaces around `<` and `>` are optional and trimmed.

### Examples

```
=== <Rolling in the Deep>     (H1, centered)
== <Adele>                    (H2, centered)
= <2011 • _21_>               (H3, centered)

= Verse 1                     (H3, left - default)
= <> page 1 of 2              (H3, right)
```

### Three-Column Lines

```
= Left Text <Centered> Right
```

Renders as:
```
Left Text           Centered           Right
```

Useful for headers/footers:
```
= transcribed by @alex <> page 1
```

### Rule: One Weight Per Line

You cannot mix weights on a single line. Use bold/italic for inline emphasis instead:

```
= Key: Am <> *Adele*          (same weight, bold for emphasis)
```

---

## Inline Formatting

Typst-style inline formatting within text lines:

| Syntax | Result |
|--------|--------|
| `*bold*` | **bold** |
| `_italic_` | *italic* |
| `*_bold italic_*` or `_*bold italic*_` | ***bold italic*** |

Styles nest one level into each other and combine, so the order doesn't matter:

| Syntax | Result |
|--------|--------|
| `*bold _both_ bold*` | **bold *both* bold** |
| `_it *both* it_` | *it **both** it* |
| `un_believ_able` | un*believ*able |

Spaces around styled text are kept as written: `a *b* c` renders as "a **b** c".
Only the spaces at the edges of each column (e.g. around `<` and `>`) are trimmed.

A style cannot nest into itself: inside `*…*`, the next `*` closes the bold.
Unclosed or crossed markers (`*bold`, `*_x*_`) are parse errors, and so is
Markdown's `**bold**` (an empty bold followed by text).

### Escaping

`*`, `_`, `<`, `>`, `[` and `]` are structural in text lines. Use backslash to write them literally:

| Literal | Escape |
|---------|--------|
| `*` | `\*` |
| `_` | `\_` |
| `<` | `\<` |
| `>` | `\>` |
| `[` | `\[` |
| `]` | `\]` |
| `\` | `\\` |

A backslash before any other character is kept as-is (`C:\path`), as in Markdown.
An unescaped `<` or `>` that doesn't form the center is a parse error.

`[` and `]` are reserved for transposable notes (e.g. `Key of [A] minor`), a
planned feature. Until then, an unescaped `[` or `]` is a parse error, so
existing charts won't change meaning when notes arrive.

---

## Chord Lines

Any line that does **not** start with a text weight marker (`===`, `==`, `=`, or `-`,
followed by a space or the end of the line) is treated as a chord line.

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
=== <Rolling in the Deep>
== <Adele>
= <2011 • _21_>

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

= Verse 3 <> N.C.
(Am?,,, ,<Em?,, ,<G?,, Em?, _ G?,) 2x

= Chorus <> *build up*, no drums
(Am <> G <> F <> F _ G) 2x

= Chorus 3
(Am G F F _ G) 4x Am fermata

= transcribed by @alex <> page 1
```

---

## Design Decisions Log

This section captures the reasoning behind key decisions made during DSL design.

### Why not standard Markdown headings?

Standard Markdown uses `#` for largest, `###` for smaller. For music charts, you'd mostly use the smallest level (annotations), requiring `###` everywhere. That's verbose.

We inverted the intuition: fewer `=` = smaller text, which matches frequency of use.

### Why allow multiple alignment zones per line?

Common use cases:
- `= Left Info <> Right Info` — header/footer layouts
- `= Key: Am <Title> Page 1` — three-column headers

### Why `LEFT <CENTER> RIGHT`?

The earlier syntax used prefix markers (`<Left <>Center >Right`). The current form
is shorter where it matters: left text needs no marker, and a centered title is
`<Title>`. It also looks like the output, with the center in the middle.

Wrapping the whole line (`<Left <Center> Right>`) was rejected as ambiguous:
`<A B>` could mean "center A B" or "left A, right B".

### Why one weight per line?

Simplicity. Mixed weights would complicate:
- Parsing
- Rendering (vertical alignment of different sizes)
- Source readability

Use `*bold*` for inline emphasis instead.

### Why Typst-style `*bold*` / `_italic_`?

Markdown uses `*italic*`, `**bold**` and `***bold italic***`. The run of stars is
ambiguous once styles nest (`***a** b*`), which is why CommonMark needs complex
delimiter rules. With one character per style, nesting is unambiguous and
composable: `*_x_*` and `_*x*_` are both bold italic. It is also the convention
of Typst, AsciiDoc and Slack/WhatsApp, and gives bold — likely the more common
emphasis on a chart — the single `*`.

Other candidates were rejected: `/italic/` (Org-mode) clashes with slash chords
like `C/G`, `''italic''` (MediaWiki) with apostrophes, and `<b>`/`<i>` tags with
the `< >` center.

### Text lines vs chord lines

Any line starting with a weight marker (`===`, `==`, `=`, or `-`) is a text line. Everything else is chords. Simple, unambiguous.

---

## Grammar (Informal)

```
document     = line (NEWLINE line)*
line         = text_line | chord_line | blank_line
blank_line   = (SP | TAB)*

text_line    = weight ( (SP | TAB)+ columns | &(NEWLINE | EOF) )
weight       = "===" | "==" | "=" | "-"

columns      = text_content ( "<" text_content ">" text_content )?
text_content = (bold | italic | text)*       # edges trimmed

bold         = "*" (text | "_" text "_")+ "*"
italic       = "_" (text | "*" text "*")+ "_"

text         = (escape | reserved | plain_char)+
escape       = "\" ( "\" | "*" | "_" | "<" | ">" | "[" | "]" )
plain_char   = any character except "*", "_", "<", ">", "[", "]", NEWLINE
reserved     = "[" | "]"                   # error: reserved for notes; kept as text

NEWLINE      = "\r\n" | "\n" | "\r" | VT | FF | NEL | LS | PS

chord_line   = chord_element (SP chord_element)*
               (repeat_marker)?
               
# Chord syntax TBD - largely follows chordsheet.com
```

---

## Future Considerations

- **Metadata:** Key, tempo, time signature — as text lines or special syntax?
- **Form notation:** AABA structure markers?
- **Rendering pipeline:** Parse → Model → SVG → PNG/PDF
- **Editor support:** Syntax highlighting for `.charts` files

---

## References

- [chordsheet.com](https://chordsheet.com) — inspiration for chord syntax
- [ChordPro](https://www.chordpro.org/) — another music notation format
- [iReal Pro](https://www.irealpro.com/) — chord chart app, inspiration for chord-in-bar layout
- [Typst](https://typst.app/) — modern typesetting system, inspiration for markup syntax
- Markdown — inspiration for the overall plain-text feel
