use super::{polygons, render_default};
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

/// The tight ink of a one-bar row holding `chord`, its glyphs and strokes, in
/// the SVG viewBox.
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
    let (mut top, mut bottom) = (f32::MAX, f32::MIN);
    ink(tree.root(), &mut top, &mut bottom);
    assert!(top < bottom, "the chord should leave some ink");
    (f64::from(top), f64::from(bottom))
}

/// Widens `top..bottom` to the ink of every text and stroke under `group`.
fn ink(group: &usvg::Group, top: &mut f32, bottom: &mut f32) {
    for child in group.children() {
        let bounds = match child {
            usvg::Node::Text(text) => text.flattened().abs_bounding_box(),
            usvg::Node::Path(path) => path.abs_bounding_box(),
            usvg::Node::Group(group) => {
                ink(group, top, bottom);
                continue;
            }
            usvg::Node::Image(_) => continue,
        };
        *top = top.min(bounds.top());
        *bottom = bottom.max(bounds.bottom());
    }
}

fn c_over(bass: Letter, quality_text: &str) -> Chord {
    Chord::new(
        Note::natural(Letter::C),
        quality(quality_text),
        Some(Note::natural(bass)),
    )
}

#[test]
fn a_hanging_bass_slash_is_a_stroke_drawn_with_its_chord() {
    let svg = render_row(vec1![Bar::Slots(vec![Slot::Chord(c_over(Letter::E, "7"))])]);
    assert_eq!(polygons(&svg).len(), 2 + 1, "bar lines and the slash");
    assert!(svg.contains("<g>"), "{svg}");
    assert!(!svg.contains(">/</tspan>"), "{svg}");
}

#[test]
fn a_squeezed_chord_squeezes_its_hanging_slash_with_it() {
    let bar = || {
        Bar::Slots(vec![
            Slot::Chord(c_over(Letter::E, "7(b9#13)")),
            Slot::Slash,
        ])
    };
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
    let squeezed: Vec<&str> = svg
        .split("<g transform=\"translate(")
        .skip(1)
        .map(|rest| rest.split("</g>").next().expect("closed group"))
        .collect();
    assert_eq!(squeezed.len(), 8, "{svg}");
    for group in squeezed {
        assert_eq!(group.matches("<polygon").count(), 1, "{group}");
    }
}

#[test]
fn a_beat_slash_stands_level_on_the_chord_baseline() {
    let svg = render_row(vec1![Bar::Slots(vec![chord(Letter::C, None), Slot::Slash])]);
    let baseline: f64 = svg
        .split("<text")
        .nth(1)
        .and_then(|text| text.split(" y=\"").nth(1))
        .and_then(|rest| rest.split('"').next())
        .expect("the chord has a baseline")
        .parse()
        .expect("a number");
    let leaning: Vec<_> = polygons(&svg)
        .into_iter()
        .filter(|corners| corners[0].0 != corners[3].0)
        .collect();
    assert_eq!(leaning.len(), 1, "{svg}");
    let corners = &leaning[0];
    assert_eq!((corners[0].1, corners[1].1), (baseline, baseline));
    assert_eq!(corners[2].1, corners[3].1, "a level top");
}

#[test]
fn a_row_is_drawn_as_bar_lines_and_chords() {
    let svg = render_row(vec1![
        Bar::Slots(vec![chord(Letter::A, Some("m7"))]),
        Bar::Slots(vec![chord(Letter::D, None)]),
    ]);
    // Defaults: 595pt page with 28pt margins, so bars split 539pt in two.
    let bar_lines: Vec<f64> = polygons(&svg)
        .iter()
        .filter(|corners| corners[0].0 == corners[3].0)
        .map(|corners| (corners[0].0 + corners[1].0) / 2.0)
        .collect();
    assert_eq!(bar_lines, [28.0, 297.5, 567.0], "{svg}");
    assert!(svg.contains(r#"font-size="38""#), "{svg}");
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
fn slashes_and_repeat_signs_are_drawn_as_strokes_and_dots() {
    let svg = render_row(vec1![
        Bar::Slots(vec![Slot::Slash, Slot::Hold]),
        Bar::Repeat
    ]);
    assert_eq!(
        polygons(&svg).len(),
        3 + 1 + 1,
        "bar lines, a beat slash and the repeat sign's slash: {svg}"
    );
    assert_eq!(svg.matches("<circle").count(), 2, "{svg}");
    assert!(!svg.contains("<text"), "nothing is drawn in a font: {svg}");
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

fn with_bar_row(chord_size: f64, gap: f64) -> SvgGenerator {
    SvgGenerator::new(RenderConfig {
        bar_row: BarRowStyle { chord_size, gap },
        ..RenderConfig::default()
    })
}

#[test]
fn a_chord_size_or_gap_that_is_not_a_fitting_number_is_an_error() {
    let chart = Chart::new(vec![]);
    let cases = [
        (0.0, 6.0),
        (-1.0, 6.0),
        (20.0, -0.1),
        (f64::NAN, 6.0),
        (f64::INFINITY, 6.0),
        (20.0, f64::NAN),
        (20.0, f64::INFINITY),
    ];
    for (chord_size, gap) in cases {
        let error = with_bar_row(chord_size, gap)
            .render(&chart)
            .expect_err("the row cannot be drawn");
        // NaN never equals itself, so the fields are compared by their bits.
        let RenderError::InvalidBarRowStyle {
            chord_size: reported_size,
            gap: reported_gap,
        } = error
        else {
            panic!("{error:?} for {chord_size}, {gap}");
        };
        assert_eq!(reported_size.to_bits(), chord_size.to_bits());
        assert_eq!(reported_gap.to_bits(), gap.to_bits());
    }
}

#[test]
fn a_tiny_chord_size_still_renders() {
    let bars = vec1![
        Bar::Slots(vec![chord(Letter::C, None), Slot::Slash]),
        Bar::Repeat
    ];
    let chart = Chart::new(vec![BarRow::new(bars).into()]);
    for chord_size in [1e-6, 1e-14, 1e-20] {
        let pages = with_bar_row(chord_size, 6.0)
            .render(&chart)
            .expect("a tiny chord size is still a size");
        let svg: String = pages.into_iter().next().expect("a page").into();
        assert!(!svg.contains("NaN") && !svg.contains("inf"), "{svg}");
    }
}
