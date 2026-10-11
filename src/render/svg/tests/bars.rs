use super::render_default;
use crate::model::{Bar, BarRow, Chart, Chord, ChordQuality, Letter, Note, Slot};
use crate::render::fonts;
use crate::render::{BarRowStyle, RenderConfig, RenderError, SvgGenerator, bar_row};
use vec1::{Vec1, vec1};

fn chord(letter: Letter, quality: Option<&str>) -> Slot {
    let quality = quality.map(|text| ChordQuality::try_new(text).expect("valid quality"));
    Slot::Chord(Chord::new(Note::natural(letter), quality, None))
}

fn render_row(bars: Vec1<Bar>) -> String {
    render_default(&Chart::new(vec![BarRow::new(bars).into()]))
}

fn quality(text: &str) -> Option<ChordQuality> {
    Some(ChordQuality::try_new(text).expect("valid quality"))
}

/// The tight glyph ink of a one-bar row holding `chord`, in the SVG viewBox.
///
/// `width`/`height` are written in `pt`; usvg would scale those to pixels, so
/// they are stripped first and the ink stays in the same units as the layout.
fn chord_ink(chord: Chord) -> (f64, f64) {
    let svg = render_row(vec1![Bar::Slots(vec![Slot::Chord(chord)])]).replace("pt\"", "\"");
    let options = usvg::Options {
        fontdb: fonts::database(),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(&svg, &options).expect("generated SVG parses");
    let mut top = f32::MAX;
    let mut bottom = f32::MIN;
    for child in tree.root().children() {
        if let usvg::Node::Text(text) = child {
            let bounds = text.flattened().abs_bounding_box();
            top = top.min(bounds.top());
            bottom = bottom.max(bounds.bottom());
        }
    }
    assert!(top < bottom, "the chord should leave some ink");
    (f64::from(top), f64::from(bottom))
}

#[test]
fn a_row_is_drawn_as_bar_lines_and_chords() {
    let svg = render_row(vec1![
        Bar::Slots(vec![chord(Letter::A, Some("m7"))]),
        Bar::Slots(vec![chord(Letter::D, None)]),
    ]);
    // Defaults: 595pt page with 28pt margins, so bars split 539pt in two.
    for x in ["28", "297.5", "567"] {
        assert!(svg.contains(&format!(r#"x1="{x}" x2="{x}""#)), "{x}: {svg}");
    }
    assert!(svg.contains(r#"font-size="20""#), "{svg}");
    assert!(svg.contains(">A</tspan>") && svg.contains(">m7</tspan>"));
    assert!(svg.contains(">D</tspan>"));
    assert!(!svg.contains("transform"), "both chords fit: {svg}");
}

#[test]
fn a_chord_too_wide_for_its_room_is_squeezed_horizontally() {
    let bar = || Bar::Slots(vec![chord(Letter::C, Some("7(b9#13)")), Slot::Slash]);
    let svg = render_row(vec1![
        bar(),
        bar(),
        bar(),
        bar(),
        bar(),
        bar(),
        bar(),
        bar()
    ]);
    assert!(svg.contains("<g transform=\"translate("), "{svg}");
    assert!(svg.contains(") scale(0."), "{svg}");
}

#[test]
fn slashes_are_lines_and_repeats_are_the_music_fonts_sign() {
    let svg = render_row(vec1![
        Bar::Slots(vec![Slot::Slash, Slot::Hold]),
        Bar::Repeat
    ]);
    assert_eq!(svg.matches("<line").count(), 3 + 1, "bar lines and a slash");
    assert!(
        svg.contains(&format!(r#"font-family="{}""#, fonts::MUSIC_FAMILY)),
        "{svg}"
    );
    assert!(svg.contains('\u{1D10E}'), "{svg}");
    assert!(svg.contains(r#"text-anchor="middle""#), "{svg}");
}

#[test]
fn the_tallest_chords_stay_inside_the_row() {
    let config = RenderConfig::default();
    let top = config.layout.margin_vertical;
    let bottom = top + bar_row::height(&config.bar_row);
    let gap_end = top + config.bar_row.gap;

    let cases = [
        Chord::new(
            Note::sharp(Letter::F),
            quality("7"),
            Some(Note::sharp(Letter::C)),
        ),
        Chord::new(
            Note::flat(Letter::B),
            quality("7"),
            Some(Note::flat(Letter::E)),
        ),
        Chord::new(
            Note::natural(Letter::C),
            quality("7(b9#13)"),
            Some(Note::natural(Letter::E)),
        ),
    ];
    for chord in cases {
        let (ink_top, ink_bottom) = chord_ink(chord);
        assert!(
            ink_top >= top && ink_bottom <= bottom,
            "{ink_top}..{ink_bottom} outside {top}..{bottom}"
        );
    }

    // A root ♯ peeks into the gap, so ASCENT is tight against the glyphs.
    let (sharp_top, _) = chord_ink(Chord::new(Note::sharp(Letter::C), None, None));
    assert!(sharp_top < gap_end, "{sharp_top} should sit in the gap");
    // A bass ♯ reaches the bottom of the row, so DESCENT is tight too.
    let (_, bass_bottom) = chord_ink(Chord::new(
        Note::natural(Letter::C),
        None,
        Some(Note::sharp(Letter::F)),
    ));
    assert!(
        bottom - bass_bottom < 2.0,
        "DESCENT leaves {bottom} - {bass_bottom} pt unused"
    );
}

#[test]
fn a_non_positive_chord_size_or_negative_gap_is_an_error() {
    let chart = Chart::new(vec![]);
    for (chord_size, gap) in [(0.0, 6.0), (-1.0, 6.0), (20.0, -0.1)] {
        let config = RenderConfig {
            bar_row: BarRowStyle { chord_size, gap },
            ..RenderConfig::default()
        };
        let error = SvgGenerator::new(config)
            .render(&chart)
            .expect_err("the row cannot be drawn");
        assert_eq!(error, RenderError::InvalidBarRowStyle { chord_size, gap });
    }
}
