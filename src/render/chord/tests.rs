use super::*;
use crate::model::{ChordQuality, Letter};

const SIZE: f64 = 10.0;
const PLACEMENTS: [BassPlacement; 2] = [BassPlacement::Below, BassPlacement::After];

fn chord(root: Note, quality: Option<&str>, bass: Option<Note>) -> Chord {
    let quality = quality.map(|text| ChordQuality::try_new(text).expect("valid quality"));
    Chord::new(root, quality, bass)
}

fn plain(chord: &Chord, placement: BassPlacement) -> Vec<Piece> {
    layout(chord, TextStyle::Normal, SIZE, placement)
}

/// Each piece's start and end, measured from the chord's start.
fn extents(pieces: &[Piece]) -> Vec<(f64, f64)> {
    let mut pen = 0.0;
    pieces
        .iter()
        .map(|piece| {
            let start = pen + piece.dx;
            pen = start + piece.width();
            (start, pen)
        })
        .collect()
}

fn find<'a>(pieces: &'a [Piece], text: &str) -> (&'a Piece, (f64, f64)) {
    let index = pieces
        .iter()
        .position(|piece| piece.text == text)
        .expect(text);
    (&pieces[index], extents(pieces)[index])
}

fn c7_over_e() -> Chord {
    chord(
        Note::natural(Letter::C),
        Some("7"),
        Some(Note::natural(Letter::E)),
    )
}

#[test]
fn a_bare_root_is_one_full_size_piece() {
    for placement in PLACEMENTS {
        let pieces = plain(&chord(Note::natural(Letter::C), None, None), placement);
        assert_eq!(
            pieces,
            [Piece::new("C", Font::Text(TextStyle::Normal), SIZE, 0.0)]
        );
        assert_eq!(overlap(&pieces), 0.0);
    }
}

#[test]
fn the_root_accidental_is_raised_and_the_quality_lowered() {
    let b_flat_minor = chord(
        Note::flat(Letter::B),
        Some("m7b5"),
        Some(Note::flat(Letter::A)),
    );
    let pieces = plain(&b_flat_minor, BassPlacement::Below);

    let mut drawn: Vec<&str> = pieces.iter().map(|piece| piece.text.as_str()).collect();
    drawn.sort_unstable();
    assert_eq!(drawn, ["/A", "5", "B", "m7", "♭", "♭", "♭"]);

    let root_accidental = pieces
        .iter()
        .find(|piece| piece.text == "♭" && piece.rise > 0.0)
        .expect("raised root accidental");
    assert_eq!(root_accidental.font, Font::Music);
    let (quality, _) = find(&pieces, "m7");
    assert!(quality.rise < 0.0 && quality.size < SIZE);
}

#[test]
fn the_pen_ends_at_the_chord_right_edge() {
    let cases = [
        chord(
            Note::flat(Letter::B),
            Some("m7"),
            Some(Note::flat(Letter::A)),
        ),
        c7_over_e(),
        chord(Note::sharp(Letter::F), None, None),
        chord(Note::natural(Letter::G), None, Some(Note::sharp(Letter::F))),
        chord(
            Note::natural(Letter::C),
            Some("7(b9#13)"),
            Some(Note::natural(Letter::E)),
        ),
    ];
    for placement in PLACEMENTS {
        for chord in &cases {
            let pieces = plain(chord, placement);
            let extents = extents(&pieces);
            let end = extents.last().expect("a chord has a root").1;
            let widest = extents.iter().map(|&(_, end)| end).fold(0.0, f64::max);
            assert_eq!(end, widest, "{chord}");
            let laid_end_to_end: f64 = pieces.iter().map(Piece::width).sum();
            assert!(
                (laid_end_to_end - overlap(&pieces) - end).abs() < 1e-9,
                "{chord}"
            );
        }
    }
}

#[test]
fn a_bass_below_hangs_under_the_root_and_the_quality() {
    let pieces = plain(&c7_over_e(), BassPlacement::Below);
    let (_, (_, root_end)) = find(&pieces, "C");
    let (quality, (quality_start, _)) = find(&pieces, "7");
    let (bass, (bass_start, _)) = find(&pieces, "/E");
    assert_eq!(quality_start, root_end);
    assert!(bass_start > 0.0 && bass_start < root_end);
    assert!(bass.rise < quality.rise);
}

#[test]
fn a_bass_after_follows_the_quality_on_the_root_baseline() {
    let pieces = plain(&c7_over_e(), BassPlacement::After);
    let (quality, (_, quality_end)) = find(&pieces, "7");
    let (bass, (bass_start, _)) = find(&pieces, "/E");
    assert!((bass_start - quality_end).abs() < 1e-9);
    assert_eq!(bass.rise, 0.0);
    assert!(quality.size < bass.size && bass.size < SIZE);
    assert_eq!(overlap(&pieces), 0.0);
}

#[test]
fn a_bass_after_clears_a_root_accidental_wider_than_the_quality() {
    let b_flat_over_a = chord(Note::flat(Letter::B), None, Some(Note::natural(Letter::A)));
    let pieces = plain(&b_flat_over_a, BassPlacement::After);
    let (_, (_, accidental_end)) = find(&pieces, "♭");
    let (_, (bass_start, _)) = find(&pieces, "/A");
    assert!((bass_start - accidental_end).abs() < 1e-9);
}

#[test]
fn every_piece_is_drawable_in_its_font() {
    let every_quality_character =
        "aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ0123456789#^-+(b)";
    let qualities = [every_quality_character, "o7", "h7", "^7", "o", "h"];
    let letters = [
        Letter::C,
        Letter::D,
        Letter::E,
        Letter::F,
        Letter::G,
        Letter::A,
        Letter::B,
    ];
    let notes = letters.into_iter().flat_map(|letter| {
        [
            Note::natural(letter),
            Note::flat(letter),
            Note::sharp(letter),
        ]
    });
    let styles = [
        TextStyle::Normal,
        TextStyle::Bold,
        TextStyle::Italic,
        TextStyle::BoldItalic,
    ];
    for note in notes {
        for quality in qualities {
            for style in styles {
                let symbol = chord(note, Some(quality), Some(note));
                for piece in layout(&symbol, style, SIZE, BassPlacement::Below) {
                    for c in piece.text.chars() {
                        assert!(fonts::has_glyph(piece.font, c), "{c} in {:?}", piece.font);
                    }
                }
            }
        }
    }
}

#[test]
fn letters_take_the_style_but_signs_stay_in_the_music_font() {
    let symbol = chord(
        Note::sharp(Letter::F),
        Some("m7"),
        Some(Note::flat(Letter::E)),
    );
    for piece in layout(&symbol, TextStyle::BoldItalic, SIZE, BassPlacement::After) {
        let expected = match piece.text.as_str() {
            "♯" | "♭" => Font::Music,
            _ => Font::Text(TextStyle::BoldItalic),
        };
        assert_eq!(piece.font, expected, "{}", piece.text);
    }
}
