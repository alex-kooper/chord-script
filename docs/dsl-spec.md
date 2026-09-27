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

## Line Kinds

Every line's kind is decided by how it starts, in column 0:

| Line starts with | Kind | Section |
|------------------|------|---------|
| `===`, `==`, `=`, `-` (then a space or end of line) | Text line | [Text Lines](#text-lines) |
| `\|` | Chord line | [Chord Lines](#chord-lines) |
| `#` | Directive | [Directives](#directives) |
| `//` | Comment | [Comments](#comments) |
| nothing, or only whitespace | Blank line (ignored) | [Line Rules](#line-rules) |

A line that starts any other way, or starts with an indented prefix, is an error.

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

> **Status:** draft, not implemented yet. The syntax below comes from
> chordsheet.com and will be revised for explicit bars.

A chord line starts with `|`. Unlike chordsheet.com, bars are explicit: every
bar is delimited by `|`, and `%` (repeat previous bar) stands inside a bar:

```
| Am | % | G | % |
```

### Basic Chord Syntax (from chordsheet.com)

| Syntax | Meaning |
|--------|---------|
| `Am`, `Cmaj7`, `F#m7b5` | Chord names |
| `_` | Beat/subdivision separator |
| `,` | Empty beat / rest |
| `*` | Repeat previous chord |
| `%` | Repeat previous bar (inside a bar: `\| % \|`) |
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

These examples predate explicit bars and will be rewritten with `|`.

---

## Directives

> **Status:** planned, not implemented yet.

A directive is a layout instruction, not content. It starts with `#` in column
0, followed by a name and, for directives that take one, a value after a space:

```
#page_break
#key Am                (possible future directive)
```

- One directive per line; the line is only the directive.
- Names are lowercase `snake_case`.
- Each directive defines its own value. A missing, extra, or malformed value
  is an error, e.g. `#page_break now`.
- An unknown name is an error that lists the known directives.
- Like every line prefix, `#` must be in column 0; an indented directive is an
  error. Inside a text line, `#` is plain text.

### Known Directives

| Directive | Value | Effect |
|-----------|-------|--------|
| `#page_break` | none | Starts a new page |

### Page Breaks

Content flows onto a new page automatically when the current page is full.
`#page_break` forces a new page at that point. There is no "smart" layout
(such as keeping a section title with the content below it): when the
automatic break lands badly, add a `#page_break` where you want it.

Breaks are literal: a `#page_break` at the start of the chart, at the end, or
right after another one produces an empty page.

---

## Comments

> **Status:** planned, not implemented yet.

A line starting with `//` in column 0 is a comment. It is not rendered.

```
// Transcribed from the album version; the live one has a longer intro.
=== <Rolling in the Deep>
```

Comments are whole lines only: in `= Verse 1 // quiet`, the `// quiet` is part
of the text.

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

### Why does every line kind have its own prefix?

Text lines start with a weight marker, chord lines with `|`, directives with
`#`, and comments with `//`. The first characters alone decide what a line is,
so the kinds can never be confused, and new kinds can be added without
changing the meaning of existing charts. It also makes errors precise: a line
with no known prefix is reported, instead of being read as chords.

### Why explicit bars (`|`)?

chordsheet.com infers bars from spacing. Explicit bar lines make the form
visible in the source, give chord lines an unambiguous prefix, and keep `%`
(repeat previous bar) inside a bar, where it can't be confused with anything
at the start of a line.

### Why `#` directives and `//` comments?

The DSL already borrows from Typst (`=` headings, `*bold*`, `_italic_`), and
Typst uses `#` for "this is an instruction, not content" and `//` for comments.
One shared directive syntax, `#name value`, covers future layout and metadata
instructions (key, bars per line, columns), so each one doesn't need a new
symbol.

Alternatives considered:
- ChordPro's `{new_page}` / `{key: Am}`: the standard in songbook tools, but
  heavier to type and read.
- ABC's `%%newpage` with `%` comments: good music precedent, but `%` already
  means "repeat previous bar" in chord lines.
- LaTeX's `\newpage`: brace arguments are heavy, and `\` is already the escape
  character in text.

A Markdown reader might take `#` for a heading, but headings here are `=`, and a
directive has no space after the `#`.

---

## Grammar (Informal)

```
document     = line (NEWLINE line)*
line         = text_line | chord_line | directive | comment | blank_line
blank_line   = (SP | TAB)*

directive    = "#" name ( (SP | TAB)+ value )?
name         = [a-z] [a-z0-9_]*
value        = any characters except NEWLINE    # trimmed; format set per directive

comment      = "//" (any character except NEWLINE)*

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

chord_line   = "|" (bar "|")+
bar          = TBD                          # chords, %, pushes, …

# Chord syntax TBD - largely follows chordsheet.com, with explicit bars
```

---

## Future Considerations

- **Metadata:** Key, tempo, time signature — likely as directives (`#key Am`, `#tempo 120`)
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
