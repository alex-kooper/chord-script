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
| `\|` | Bar row | [Bar Rows](#bar-rows) |
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
Inside a column, a run of spaces renders as a single space, so `a     b` renders
as "a b".

Text may use any script, but unprintable characters (control characters
other than tab) are an error. PDF output draws text only in its bundled font,
which covers Latin, Cyrillic, and Greek; any other character, such as an emoji,
a CJK character, or a typed `♭`, is an error rather than silently missing from
the PDF. Write sharps and flats as [inline chords](#inline-chords) instead, e.g. `[Bb]`.

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
An unescaped `<` or `>` that doesn't form the center is a parse error, and so is
a `]` that closes no `[`.

### Inline Chords

A chord in square brackets can stand anywhere in a text line:

```
= Key of [Bb] minor <> Capo 3
- Vamp on [Am7] [D7/F#] until the cue
```

A note is a chord with no quality, so `[Bb]` serves for a key as well. Inline
chords are written as chords rather than text so that they can be transposed
and drawn with proper symbols.

Chords are drawn in the style of iReal Pro: the root at full size with its
accidental small and raised, the quality smaller and lowered, and the bass a
little smaller than the root, on its baseline. In the quality, `b` and `#` are drawn as ♭ and ♯, `^` as Δ, and a
leading `o` or `h` with no letter after it as ° or ø: `[Bbh7/Ab]` reads
B♭ø7/A♭, while `[Comit3]` keeps its `o`.

The brackets hold exactly one chord symbol and nothing else, not even spaces:

| Part | Syntax | Examples |
|------|--------|----------|
| Root | `A` to `G`, then an optional `b` or `#` | `C`, `Bb`, `F#`, `Cb` |
| Quality (optional) | ASCII letters and digits, `#`, `^`, `-`, `+`, and `( )` groups | `m7`, `^7`, `-7`, `o7`, `h7`, `sus4`, `7(b9#13)` |
| Bass (optional) | `/` and a single note | `C/E`, `D/F#` |

- The quality is kept exactly as written: `[Cmaj7]` and `[C^7]` are both valid
  and stay different text.
- The quality never starts with `b` or `#`: right after the root, those belong
  to the root. `[Cb5]` is C-flat with quality `5`; a flat fifth is `[C(b5)]`,
  or `[C7b5]` for the seventh chord. Double accidentals (`[Cbb]`) are errors.
- Parentheses come in pairs, each with something inside and none nested:
  `[C7(b9)(#11)]`, not `[C7(b9]` or `[C()]`.
- Inline chords take the style around them: `*[Am]*` is a bold chord.
- `[ Am ]`, `[]`, `[Am G]`, `[am]`, `[C/]`, `[C/E7]`, and an unclosed `[Am`
  are errors. Write `\[` and `\]` for literal brackets.

---

## Bar Rows

Bars with chords are the heart of a chart, as in iReal Pro or chordsheet.com.
A bar row starts with `|` and is one row of the chart's grid: every bar stands
between two `|`, and the row is drawn as bars of equal width, as many as it
has.

```
| Am _ _ A7 | Dm | G7 | C |
| % | % | G G7 | C |
```

A bar holds either slots or a repeat:

| Bar | Meaning | Example |
|-----|---------|---------|
| Slots, separated by spaces | The bar is split evenly into one slot per item | `\| Am _ _ A7 \|`: A7 on the last quarter |
| `%` alone | The previous bar's chords again | `\| Am \| % \|` |

| Slot | Meaning | Drawn as |
|------|---------|----------|
| A chord | The chord starts here | The chord |
| `_` | The chord before continues | Nothing |
| `/` | The chord before continues on this beat | A beat slash |

- Chords are written as in [Inline Chords](#inline-chords), without the
  brackets: `| Bb^7 | Eb7(#11) | Am7/G |`.
- Slots are positions, not beats: `| Am Dm |` is two halves of the bar, the
  same as `| Am _ Dm _ |`. `_` is only needed when chords are not evenly
  spaced: `| A _ _ D |` puts D on beat 4 of a 4/4 bar, `| _ B _ _ |` puts B
  on beat 2.
- `/` never moves a chord, it only shows the beats: `| Am / Dm / |` places
  its chords like `| Am Dm |`. Slots after the last chord count too:
  `| Bo / Do7 / |` has Do7 on beat 3, while `| Bo / Do7 |` splits the bar in
  thirds.
- `%` repeats the previous bar in the chart, even in an earlier row: text
  lines and page breaks in between are skipped.
- Spaces around `|` are optional: `|Am|G|` is the same as `| Am | G |`. An
  empty bar needs at least one space, `| |`: `||` is reserved for the planned
  double bar line.

Making musical sense is up to the author: whatever can be drawn is drawn as
written. An empty bar (`| |`), a bar starting with `_` or `/`, and `%` or `/`
with nothing before them to continue are all drawn. These are errors:

- `||`, until double bar lines arrive: write `| |` for an empty bar
- `%` with anything else in its bar, e.g. `| % Am |`
- slots without spaces between them, e.g. `Am_G`: write `| Am G |` for two
  chords in one bar (chordsheet.com's `Am_G`), or `| Am _ G |` for three slots
- anything in a bar that is not a chord, `_`, `/` or `%`, including the
  planned repeat signs `|:` and `:|`
- a row that doesn't end with `|`, and a `|` alone
- unprintable characters, as in text, and spaces other than space and tab

### Drawing

Bar rows are drawn in the style of iReal Pro:

- A row spans the page between its margins, and its bars split that width
  evenly. Rows with the same number of bars line up.
- A chord or `/` starts a little after the start of its slot, so a beat lands
  at the same place in every bar with as many slots. `%` is drawn as the
  one-bar repeat sign, centered in its bar.
- A chord may reach over the `_` slots after it. A chord too wide for the room
  before the next chord or `/` in its bar, or before the bar line, is squeezed
  horizontally to fit; its height stays the same. If a row looks cramped, give
  it fewer bars.
- The bass hangs below the root, a little to the right. Bar lines run from
  the top of the chords to about the middle of the bass.
- Every bar row has the same height, with or without a bass, and a small gap
  above it. For more space between rows, add an empty line such as `==`.

### Planned

In rough order:

1. Spans: invisible bars for alignment, written with a width, e.g.
   `{2} | F | E7 |` puts two bars under the last two of a four-bar row above.
   Spans may later carry text, left, centered, or right aligned. The syntax is
   not settled.
2. Repeat signs (`|:`, `:|`, `:|:`), double (`||`) and final bar lines.
3. Endings (`1.`, `2.`) and repeat counts.
4. Lyrics and other symbols (dynamics, fermatas) in their own rows, aligned
   with the bars and chords.

### Ideas from chordsheet.com

chordsheet.com has no bar lines: every chord is a bar, `_` joins chords into
one bar, `*` holds the previous chord, and `,` is a beat slash. So `Am_Dm`
there is `| Am Dm |` here, `*_B_*_*` is `| _ B _ _ |`, and `Bdim,Ddim7, ,,,,`
is `| Bdim / Ddim7 / | / / / / |`.

Its other features are not decided; they are kept for reference and would
need revising for explicit bars:

| Syntax | Meaning |
|--------|---------|
| `( ) Nx` | Repeat group N times |
| `1.` `2.` | First/second endings |
| `<Chord` | Push (anticipation) |
| `<>` | Accent/stab |
| `?` suffix | Ghost/optional chord |
| `N.C.` | No chord |
| `fermata` | Hold |
| `"text"` | Inline chord annotation |

```
Am,,, ,<Em,, ,<G,, Em, _ G,       (complex rhythm with pushes)
(Am G F F _ G) 4x                 (repeat group 4 times)
(F G Em  1. F  2. E)              (with endings)
Am <> G <> F                      (accented chords)
Am? Dm?                           (ghost/optional chords)
(Am G F F _ G) 4x Am fermata      (ending with fermata)
```

---

## Directives

A directive is a layout instruction, not content. It starts with `#` in column
0, followed by a name and, for directives that take one, a value after a space:

```
#page_break
#key Am                (possible future directive)
```

- One directive per line; the line is only the directive.
- Names are lowercase `snake_case`. The name runs up to the first whitespace,
  so `#page-break` is one invalid name, not `#page` followed by `-break`.
- Each directive defines its own value. A missing, extra, or malformed value
  is an error, e.g. `#page_break now`.
- An unknown name is an error that lists the known directives. When a known
  name differs only in case or punctuation (`#page-break`, `#pagebreak`,
  `#Page_Break`), the error suggests it.
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
right after another one produces an empty page. Spacer lines (a bare `===`,
`==`, `=`, or `-`) right after a `#page_break` are kept, so you can push content down
on purpose.

At an automatic break, spacers that would start the next page are dropped:
that space was only meant to separate content on the previous page.

A single line or bar row taller than the space between a page's margins is an
error; the chart is not rendered.

---

## Comments

A line starting with `//` in column 0 is a comment. It is not rendered.

```
// Transcribed from the album version; the live one has a longer intro.
=== <Rolling in the Deep>
```

Comments are whole lines only: in `= Verse 1 // quiet`, the `// quiet` is part
of the text.

---

## Complete Example

> The chord rows below use the earlier chordsheet.com syntax and will be
> rewritten as [bar rows](#bar-rows).

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

Text lines start with a weight marker, bar rows with `|`, directives with
`#`, and comments with `//`. The first characters alone decide what a line is,
so the kinds can never be confused, and new kinds can be added without
changing the meaning of existing charts. It also makes errors precise: a line
with no known prefix is reported, instead of being read as chords.

### Why explicit bars (`|`)?

In chordsheet.com every chord is a bar, and `_` joins chords into one bar.
That is very concise with one chord per bar, but once bars hold several
chords, holds, and annotations, it gets hard to see where a bar ends:
`*_*_C_* E F G` versus `| _ _ C _ | E | F | G |`. Explicit bar lines make the
form visible in the source, give bar rows an unambiguous prefix, and keep `%`
(repeat previous bar) inside a bar, where it can't be confused with anything
at the start of a line.

### Why `_` and `/` inside bars?

`_` holds the chord before it without drawing anything; it places chords that
are not evenly spaced, like chordsheet.com's `*`. `/` holds it too but draws
a beat slash, like chordsheet.com's `,` and the slashes on jazz and Nashville
charts. Spaces separate slots, so `_` is free for the hold, and `/` can't be
confused with a slash chord: `Am/E` is one chord, `Am /` a chord and a slash.

### Why does the model accept musical nonsense?

The model holds what the author wrote, as text lines hold "Verse" as plain
text rather than as a section. Only what can't be drawn, or would be drawn
ambiguously, is an error. A `%` in the first bar or a bar of slashes with no
chord before it is drawn as written, and the mistake is visible on the page.

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
  means "repeat previous bar" in bar rows.
- LaTeX's `\newpage`: brace arguments are heavy, and `\` is already the escape
  character in text.

A Markdown reader might take `#` for a heading, but headings here are `=`, and a
directive has no space after the `#`.

---

## Grammar (Informal)

```
document     = line (NEWLINE line)*
line         = text_line | bar_row | directive | comment | blank_line
blank_line   = (SP | TAB)*

directive    = "#" name ( (SP | TAB)+ value )?
name         = [a-z] [a-z0-9_]*               # read up to whitespace, then checked
value        = any characters except NEWLINE    # trimmed; format set per directive

comment      = "//" (any character except NEWLINE)*

text_line    = weight ( (SP | TAB)+ columns | &(NEWLINE | EOF) )
weight       = "===" | "==" | "=" | "-"

columns      = text_content ( "<" text_content ">" text_content )?
text_content = (bold | italic | inline)*     # edges trimmed

bold         = "*" (inline | "_" inline+ "_")+ "*"
italic       = "_" (inline | "*" inline+ "*")+ "_"
inline       = "[" chord "]" | text

chord        = note quality? ("/" note)?
note         = ( "A" | "B" | "C" | "D" | "E" | "F" | "G" ) ( "b" | "#" )?
quality      = ( quality_char | "(" quality_char+ ")" )+   # not starting with "b" or "#"
quality_char = [A-Za-z0-9] | "#" | "^" | "-" | "+"

text         = (escape | plain_char)+
escape       = "\" ( "\" | "*" | "_" | "<" | ">" | "[" | "]" )
plain_char   = any character except "*", "_", "<", ">", "[", "]", NEWLINE

NEWLINE      = "\r\n" | "\n" | "\r" | VT | FF | NEL | LS | PS

bar_row      = "|" ( WS* bar WS* "|" )+ WS*
bar          = "%" | slot ( WS+ slot )* | WS    # WS alone: an empty bar
slot         = chord | "_" | "/"
WS           = SP | TAB
```

---

## Future Considerations

- **Metadata:** Key, tempo, time signature — likely as directives (`#key Am`, `#tempo 120`)
- **Form notation:** AABA structure markers?
- **Rendering pipeline:** Parse → Model → SVG pages or one PDF; PNG later
- **Editor support:** Syntax highlighting for `.charts` files

---

## References

- [chordsheet.com](https://chordsheet.com) — inspiration for chord syntax
- [ChordPro](https://www.chordpro.org/) — another music notation format
- [iReal Pro](https://www.irealpro.com/) — chord chart app, inspiration for chord-in-bar layout
- [Typst](https://typst.app/) — modern typesetting system, inspiration for markup syntax
- Markdown — inspiration for the overall plain-text feel
