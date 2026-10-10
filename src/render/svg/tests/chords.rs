use super::{chart_of, left_line, render_default, render_one};
use crate::model::{Chord, ChordQuality, Inline, Letter, Line, LineLevel, Note, TextStyle};
use crate::render::{FontStyle, RenderConfig, SvgGenerator, fonts};

fn b_flat_minor_over_f() -> Chord {
    Chord::new(
        Note::flat(Letter::B),
        Some(ChordQuality::try_new("m7").expect("valid quality")),
        Some(Note::natural(Letter::F)),
    )
}

#[test]
fn chord_is_drawn_in_pieces_in_its_style() {
    let line = left_line(
        LineLevel::Text,
        vec![
            Inline::plain("Key of "),
            Inline::new(b_flat_minor_over_f(), TextStyle::Bold),
        ],
    );
    let svg = render_default(&chart_of(vec![line]));

    assert!(
        svg.contains(r#"<tspan>Key of </tspan><tspan font-weight="bold">B</tspan>"#),
        "{svg}"
    );
    let tspan_of = |text: &str| {
        let end = svg.find(&format!(">{text}</tspan>")).expect(text);
        let start = svg[..end].rfind("<tspan").expect("an opening tag");
        svg[start..end].to_string()
    };
    let flat = tspan_of("♭");
    assert!(flat.contains(r#"font-family="Noto Music""#), "{flat}");
    assert!(flat.contains("baseline-shift="), "{flat}");
    for text in ["m7", "/F"] {
        let tspan = tspan_of(text);
        assert!(tspan.contains(r#"font-weight="bold""#), "{tspan}");
        assert!(tspan.contains("font-size="), "{tspan}");
    }
    assert!(tspan_of("m7").contains("baseline-shift=\"-"));
    assert!(!tspan_of("/F").contains("baseline-shift"));
}

/// The `dx` moves of a text line holding `[Bbi]` in `style`, drawn with the
/// text level at `weight`.
///
/// The quality `i` is narrower than the ♭ above it, so the ♭ is drawn after it
/// and moves back by the width of `i`, which differs between the faces.
fn chord_moves(weight: &str, style: TextStyle) -> Vec<String> {
    let defaults = RenderConfig::default();
    let config = RenderConfig {
        text: FontStyle {
            weight: weight.to_string(),
            ..defaults.text.clone()
        },
        ..defaults
    };
    let chord = Chord::new(
        Note::flat(Letter::B),
        Some(ChordQuality::try_new("i").expect("valid quality")),
        None,
    );
    let line = left_line(LineLevel::Text, vec![Inline::new(chord, style)]);
    let svg = render_one(&SvgGenerator::new(config), &chart_of(vec![line]));
    svg.split(" dx=\"")
        .skip(1)
        .map(|rest| rest.split('"').next().expect("closing quote").to_string())
        .collect()
}

#[test]
fn chords_in_a_bold_level_are_measured_in_the_bold_face() {
    let bold_level = chord_moves("700", TextStyle::Normal);
    assert!(!bold_level.is_empty());
    assert_eq!(bold_level, chord_moves("normal", TextStyle::Bold));
    assert_ne!(bold_level, chord_moves("normal", TextStyle::Normal));
}

/// The left and right edges of the ink of the line, as usvg draws it.
fn ink_edges(line: Line) -> (f32, f32) {
    let options = usvg::Options {
        fontdb: fonts::database(),
        ..usvg::Options::default()
    };
    let svg = render_default(&chart_of(vec![line]));
    let tree = usvg::Tree::from_str(&svg, &options).expect("generated SVG parses");
    let bounds = tree.root().abs_bounding_box();
    (bounds.left(), bounds.right())
}

/// `Key [Bbm7/F] end`, or `Key  end` without the chord.
fn column(with_chord: bool) -> Vec<Inline> {
    let mut inlines = vec![Inline::plain("Key ")];
    if with_chord {
        inlines.push(Inline::new(b_flat_minor_over_f(), TextStyle::Normal));
    }
    inlines.push(Inline::plain(" end"));
    inlines
}

#[test]
fn chords_keep_right_aligned_columns_on_the_margin() {
    let right = |with_chord| Line::new(LineLevel::Text, vec![], vec![], column(with_chord));
    let (_, with_chord) = ink_edges(right(true));
    let (_, without) = ink_edges(right(false));
    assert!(
        (with_chord - without).abs() < 0.01,
        "{with_chord} vs {without}"
    );
}

#[test]
fn chords_keep_centered_columns_centered() {
    let centered = |with_chord| Line::new(LineLevel::Text, vec![], column(with_chord), vec![]);
    let (left, right) = ink_edges(centered(true));
    let (plain_left, plain_right) = ink_edges(centered(false));
    let chord_width = (right - left) - (plain_right - plain_left);
    assert!(chord_width > 0.0);
    assert!(
        (right - plain_right - chord_width / 2.0).abs() < 0.01,
        "{right} vs {plain_right}"
    );
}
