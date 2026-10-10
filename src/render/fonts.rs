//! Fonts bundled into the binary, so output looks the same on every machine.
//!
//! Noto Sans covers Latin, Cyrillic (including Ukrainian), and Greek. It ships
//! as four static faces, so weights snap to regular (400) or bold (700).
//! Noto Music draws the ♭ and ♯ of chord symbols, which Noto Sans lacks; it
//! has a regular face only.

use crate::model::TextStyle;
use std::sync::{Arc, OnceLock};
use ttf_parser::{Face, GlyphId};
use usvg::fontdb::{Database, Family, Query, Source};

/// The family name of the bundled text font.
pub(super) const FAMILY: &str = "Noto Sans";

/// The family name of the bundled font for ♭ and ♯ in chord symbols.
pub(super) const MUSIC_FAMILY: &str = "Noto Music";

/// The bundled faces: the text faces in [`text_face_index`] order, then the
/// music face.
const FACES: [&[u8]; 5] = [
    include_bytes!("../../assets/fonts/NotoSans-Regular.ttf"),
    include_bytes!("../../assets/fonts/NotoSans-Bold.ttf"),
    include_bytes!("../../assets/fonts/NotoSans-Italic.ttf"),
    include_bytes!("../../assets/fonts/NotoSans-BoldItalic.ttf"),
    include_bytes!("../../assets/fonts/NotoMusic-Regular.ttf"),
];

const MUSIC_FACE_INDEX: usize = 4;

/// A bundled face to measure text in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Font {
    /// Noto Sans in the given style
    Text(TextStyle),
    /// Noto Music
    Music,
}

/// A font database holding only the bundled faces, never system fonts.
///
/// The generic `sans-serif` family also maps to the bundled font; fontdb would
/// otherwise map it to Arial, which may not be loaded.
pub(super) fn database() -> Arc<Database> {
    static DATABASE: OnceLock<Arc<Database>> = OnceLock::new();
    DATABASE
        .get_or_init(|| {
            let mut database = Database::new();
            for face in FACES {
                database.load_font_source(Source::Binary(Arc::new(face)));
            }
            database.set_sans_serif_family(FAMILY);
            Arc::new(database)
        })
        .clone()
}

/// Whether a CSS-style family list, e.g. `Noto Sans, sans-serif`, names at
/// least one bundled font.
pub(super) fn has_family(family_list: &str) -> bool {
    let database = database();
    family_list.split(',').map(parse_family).any(|family| {
        database
            .query(&Query {
                families: &[family],
                ..Query::default()
            })
            .is_some()
    })
}

fn parse_family(name: &str) -> Family<'_> {
    match name.trim().trim_matches(['"', '\'']) {
        "serif" => Family::Serif,
        "sans-serif" => Family::SansSerif,
        "monospace" => Family::Monospace,
        "cursive" => Family::Cursive,
        "fantasy" => Family::Fantasy,
        name => Family::Name(name),
    }
}

/// Whether some bundled text face has a glyph for `c`. The music face does not
/// count: it draws chord symbols, not text.
pub(super) fn can_draw(c: char) -> bool {
    text_faces()
        .iter()
        .any(|face| face.glyph_index(c).is_some())
}

/// The width of `text` in `font` at `size`, from glyph advances alone.
///
/// Kerning is ignored, which is close enough for the short pieces of a chord
/// symbol.
pub(super) fn advance(text: &str, font: Font, size: f64) -> f64 {
    let face = &faces()[face_index(font)];
    let units: u32 = text
        .chars()
        .map(|c| {
            let glyph = face.glyph_index(c).unwrap_or(GlyphId(0));
            u32::from(face.glyph_hor_advance(glyph).unwrap_or(0))
        })
        .sum();
    f64::from(units) * size / f64::from(face.units_per_em())
}

#[cfg(test)]
pub(super) fn has_glyph(font: Font, c: char) -> bool {
    faces()[face_index(font)].glyph_index(c).is_some()
}

fn face_index(font: Font) -> usize {
    match font {
        Font::Text(style) => text_face_index(style),
        Font::Music => MUSIC_FACE_INDEX,
    }
}

fn text_face_index(style: TextStyle) -> usize {
    match style {
        TextStyle::Normal => 0,
        TextStyle::Bold => 1,
        TextStyle::Italic => 2,
        TextStyle::BoldItalic => 3,
    }
}

fn text_faces() -> &'static [Face<'static>] {
    &faces()[..MUSIC_FACE_INDEX]
}

fn faces() -> &'static [Face<'static>] {
    static FACES_PARSED: OnceLock<Vec<Face<'static>>> = OnceLock::new();
    FACES_PARSED.get_or_init(|| {
        FACES
            .iter()
            .map(|data| Face::parse(data, 0).expect("bundled fonts are valid"))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use usvg::fontdb::{ID, Style, Weight};

    fn find(family: Family, weight: Weight, style: Style) -> Option<ID> {
        database().query(&Query {
            families: &[family],
            weight,
            style,
            ..Query::default()
        })
    }

    fn post_script_name(id: ID) -> String {
        let database = database();
        let face = database.face(id).expect("queried faces exist");
        face.post_script_name.clone()
    }

    #[test]
    fn every_style_resolves_to_its_own_bundled_face() {
        let family = Family::Name(FAMILY);
        let cases = [
            (Weight::NORMAL, Style::Normal, "NotoSans-Regular"),
            (Weight::BOLD, Style::Normal, "NotoSans-Bold"),
            (Weight::NORMAL, Style::Italic, "NotoSans-Italic"),
            (Weight::BOLD, Style::Italic, "NotoSans-BoldItalic"),
        ];
        for (weight, style, expected) in cases {
            let id = find(family, weight, style).expect("bundled face should resolve");
            assert_eq!(post_script_name(id), expected);
        }
    }

    #[test]
    fn in_between_weights_snap_to_a_bundled_face() {
        let id = find(Family::Name(FAMILY), Weight(500), Style::Normal).expect("resolves");
        assert_eq!(post_script_name(id), "NotoSans-Regular");
    }

    #[test]
    fn generic_sans_serif_is_the_bundled_font() {
        let id = find(Family::SansSerif, Weight::NORMAL, Style::Normal).expect("resolves");
        assert_eq!(post_script_name(id), "NotoSans-Regular");
    }

    #[test]
    fn every_face_covers_latin_ukrainian_and_russian() {
        let required = "AZaz éüñçß ҐґЄєІіЇїʼ ЁёЪъЫыЭэ №«»–—’ °øΔ";
        for face in text_faces() {
            let missing: String = required
                .chars()
                .filter(|c| !c.is_whitespace() && face.glyph_index(*c).is_none())
                .collect();
            assert!(missing.is_empty(), "missing glyphs: {missing}");
        }
    }

    #[test]
    fn symbols_emoji_and_cjk_cannot_be_drawn() {
        for c in ['♭', '♯', '♮', '△', '😀', '你'] {
            assert!(!can_draw(c), "{c} is not expected in the bundled fonts");
        }
    }

    #[test]
    fn music_face_draws_accidentals_and_is_found_by_its_family() {
        let music = &faces()[MUSIC_FACE_INDEX];
        for c in ['♭', '♯'] {
            assert!(music.glyph_index(c).is_some(), "{c}");
        }
        let id = find(Family::Name(MUSIC_FAMILY), Weight::NORMAL, Style::Normal);
        assert_eq!(post_script_name(id.expect("resolves")), "NotoMusic-Regular");
    }

    #[test]
    fn advance_scales_with_size_and_depends_on_the_face() {
        let regular = advance("C7", Font::Text(TextStyle::Normal), 10.0);
        assert!(regular > 0.0);
        assert_eq!(
            advance("C7", Font::Text(TextStyle::Normal), 20.0),
            2.0 * regular
        );
        assert_ne!(advance("C7", Font::Text(TextStyle::Bold), 10.0), regular);
        assert!(advance("♭", Font::Music, 10.0) > 0.0);
    }

    #[test]
    fn family_lists_match_bundled_or_generic_sans_serif_names() {
        for list in [
            "Noto Sans",
            "'Noto Sans'",
            "Times, sans-serif",
            "\"Noto Sans\", serif",
        ] {
            assert!(has_family(list), "{list}");
        }
        for list in ["Times New Roman", "serif", "Arial, monospace", ""] {
            assert!(!has_family(list), "{list}");
        }
    }
}
