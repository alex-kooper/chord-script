//! Chord symbols: `Bb`, `F#m7b5/C#`, `C7(b9)/E`.
//!
//! A chord is a root note, an optional quality, and an optional `/` bass note.
//! The parser stops at the first character that cannot belong to a chord, so
//! the surrounding syntax (`]`, a space, `|`, the `)` of an optional `(Am7)`)
//! decides what may follow.

use super::Extra;
use crate::model::{
    Accidental, Chord, ChordQuality, ChordQualityError, Letter, Note, is_quality_char,
};
use chumsky::prelude::*;

/// A whole chord symbol.
pub(super) fn chord<'a>() -> impl Parser<'a, &'a str, Chord, Extra<'a>> {
    note()
        .then(quality().or_not().map(Option::flatten))
        .then(bass().or_not().map(Option::flatten))
        .map(|((root, quality), bass)| Chord::new(root, quality, bass))
}

/// `/` and a single note. Quality characters glued to the bass, as in `C/E7`,
/// are reported: nothing may follow a chord without a separator. A `/` without
/// a note is reported too, and yields `None`.
fn bass<'a>() -> impl Parser<'a, &'a str, Option<Note>, Extra<'a>> {
    let trailing = quality_text().validate(|text: &str, e, emitter| {
        emitter.emit(Rich::custom(
            e.span(),
            format!("the bass after `/` is a single note, but `{text}` follows it"),
        ));
    });
    let missing = quality_text()
        .or_not()
        .to_slice()
        .validate(|text: &str, e, emitter| {
            let found = if text.is_empty() {
                String::new()
            } else {
                format!(", not `{text}`")
            };
            emitter.emit(Rich::custom(
                e.span(),
                format!("a bass note (A to G) must follow `/`{found}"),
            ));
            None
        });

    just('/').ignore_then(note().then_ignore(trailing.or_not()).map(Some).or(missing))
}

/// An uppercase letter and an optional `b` or `#`, e.g. `C`, `Bb`, `F#`.
///
/// The accidental is read greedily, so `Cb5` is C-flat with quality `5`.
pub(super) fn note<'a>() -> impl Parser<'a, &'a str, Note, Extra<'a>> {
    let letter = choice((
        just('C').to(Letter::C),
        just('D').to(Letter::D),
        just('E').to(Letter::E),
        just('F').to(Letter::F),
        just('G').to(Letter::G),
        just('A').to(Letter::A),
        just('B').to(Letter::B),
    ))
    .labelled("note letter (A to G)");
    let accidental = just('b')
        .to(Accidental::Flat)
        .or(just('#').to(Accidental::Sharp));

    letter
        .then(accidental.or_not())
        .map(|(letter, accidental)| Note { letter, accidental })
}

/// The characters after the root, checked as a [`ChordQuality`]. An invalid
/// quality is reported and yields `None`; the parse fails either way, but the
/// rest of the input is still checked.
fn quality<'a>() -> impl Parser<'a, &'a str, Option<ChordQuality>, Extra<'a>> {
    quality_text().validate(|text: &str, e, emitter| {
        ChordQuality::try_new(text)
            .map_err(|error| emitter.emit(Rich::custom(e.span(), quality_problem(text, &error))))
            .ok()
    })
}

/// Text shaped like a quality, read by structure: plain quality characters
/// and whole `(…)` groups. A `)` without its `(` is not part of it, so
/// `(Am7)` ends the chord at the `)`.
fn quality_text<'a>() -> impl Parser<'a, &'a str, &'a str, Extra<'a>> {
    let unbracketed = || any().filter(|c: &char| is_quality_char(*c) && !matches!(c, '(' | ')'));
    let group = just('(')
        .then(unbracketed().labelled("chord alteration").repeated())
        .then(just(')').labelled("`)` to close `(`"));

    unbracketed()
        .ignored()
        .or(group.ignored())
        .repeated()
        .at_least(1)
        .to_slice()
}

/// Why `text` is not a valid quality. The grammar already guarantees closed,
/// unnested parentheses, so only the model's remaining rules can fail here.
fn quality_problem(text: &str, error: &ChordQualityError) -> String {
    if text.starts_with(['b', '#']) {
        "double accidentals are not supported; an alteration right after the root \
         goes in parentheses, e.g. `Bb(b5)`"
            .to_string()
    } else if text.contains("()") {
        format!("empty parentheses in chord quality `{text}`")
    } else {
        format!("invalid chord quality `{text}`: {error}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<Chord, Vec<String>> {
        chord()
            .then_ignore(end())
            .parse(input)
            .into_result()
            .map_err(|errors| errors.iter().map(ToString::to_string).collect())
    }

    fn error(input: &str) -> String {
        let errors = parse(input).expect_err("chord should be rejected");
        assert_eq!(errors.len(), 1, "{input:?}: {errors:?}");
        errors.into_iter().next().expect("one error")
    }

    /// The chord at the start of `input`, and what follows it unparsed.
    fn split(input: &str) -> (Chord, &str) {
        chord()
            .then(any().repeated().to_slice())
            .parse(input)
            .into_result()
            .unwrap_or_else(|errors| panic!("{input:?} should start with a chord: {errors:?}"))
    }

    fn parse_prefix(input: &str) -> Chord {
        split(input).0
    }

    fn rest_after_chord(input: &str) -> &str {
        split(input).1
    }

    #[test]
    fn chords_round_trip_through_display() {
        for symbol in [
            "C",
            "Bb",
            "F#",
            "Cb",
            "E#",
            "Am",
            "Bbm7b5",
            "F#m7b5/C#",
            "Co7",
            "Ch7",
            "C^7",
            "CmM7",
            "C-7",
            "C+",
            "Csus4",
            "Cadd9",
            "C(b5)",
            "C7(b9#13)/E",
            "D/F#",
        ] {
            let chord = parse(symbol).unwrap_or_else(|e| panic!("{symbol:?} should parse: {e:?}"));
            assert_eq!(chord.to_string(), symbol);
        }
    }

    #[test]
    fn chord_parts_are_separated() {
        let chord = parse("F#m7b5/C#").expect("valid chord");
        assert_eq!(chord.root, Note::sharp(Letter::F));
        assert_eq!(chord.quality.as_ref().map(AsRef::as_ref), Some("m7b5"));
        assert_eq!(chord.bass, Some(Note::sharp(Letter::C)));
    }

    #[test]
    fn accidental_after_the_root_belongs_to_the_root() {
        let chord = parse("Cb5").expect("valid chord");
        assert_eq!(chord.root.accidental, Some(Accidental::Flat));
        assert_eq!(chord.quality.as_ref().map(AsRef::as_ref), Some("5"));
    }

    #[test]
    fn root_must_be_an_uppercase_note_letter() {
        for input in ["am", "H7", "", "7"] {
            assert!(error(input).contains("note letter (A to G)"), "{input:?}");
        }
    }

    #[test]
    fn double_accidentals_are_rejected() {
        for input in ["Cbb5", "F##", "Bb#"] {
            assert!(error(input).contains("double accidentals"), "{input:?}");
        }
    }

    #[test]
    fn unclosed_or_nested_parentheses_are_rejected() {
        for input in ["C7(b9", "C7(b9 G", "C7((b9))"] {
            assert!(error(input).contains("`)` to close `(`"), "{input:?}");
        }
    }

    #[test]
    fn empty_parentheses_are_rejected() {
        assert!(error("C()").contains("empty parentheses"));
        assert!(error("C7(b9)()").contains("empty parentheses"));
    }

    #[test]
    fn slash_needs_a_bass_note() {
        for input in ["C/", "C/h", "C/7"] {
            assert!(error(input).contains("bass note (A to G)"), "{input:?}");
        }
    }

    #[test]
    fn bass_is_a_single_note() {
        for (input, extra) in [
            ("C/E7", "`7`"),
            ("C/Eb7", "`7`"),
            ("C/Gm", "`m`"),
            ("C/E(b9)", "`(b9)`"),
        ] {
            let message = error(input);
            assert!(message.contains("single note"), "{input:?}: {message}");
            assert!(message.contains(extra), "{input:?}: {message}");
        }
    }

    #[test]
    fn closing_parenthesis_ends_the_chord() {
        let chord = parse_prefix("Am7)");
        assert_eq!(chord.to_string(), "Am7");
        assert_eq!(rest_after_chord("C/E)"), ")");
        assert_eq!(rest_after_chord("C7(b9))"), ")");
    }

    #[test]
    fn parser_stops_where_the_chord_ends() {
        for (input, rest) in [
            ("Am7]", "]"),
            ("Am7 G", " G"),
            ("Am7|", "|"),
            ("C/E:|", ":|"),
            ("G7,", ","),
            ("Dm_G", "_G"),
            ("Am7)", ")"),
            ("Am7?", "?"),
            ("C7(b9) G", " G"),
        ] {
            assert_eq!(rest_after_chord(input), rest, "{input:?}");
        }
    }
}
