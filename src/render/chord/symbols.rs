//! How the ASCII of a chord symbol is written on paper: `b` and `#` become
//! ♭ and ♯, `^` becomes Δ, and a leading `o` or `h` becomes ° or ø.

use crate::model::Accidental;

/// A stretch of a chord's quality drawn in one font.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Run {
    pub text: String,
    pub kind: RunKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RunKind {
    /// Letters, digits and signs from the text font
    Text,
    /// ♭ and ♯ from the music font
    Accidental,
}

pub(super) fn accidental_sign(accidental: Accidental) -> char {
    match accidental {
        Accidental::Flat => '♭',
        Accidental::Sharp => '♯',
    }
}

/// The quality as drawn, split where the font changes: `m7b5` is `m7`, `♭`,
/// `5`.
pub(super) fn quality_runs(quality: &str) -> Vec<Run> {
    let word_follows_first = quality.chars().nth(1).is_some_and(is_letter);
    let mut runs: Vec<Run> = Vec::new();
    for (index, c) in quality.chars().enumerate() {
        let (sign, kind) = drawn(c, index == 0 && !word_follows_first);
        match runs.last_mut() {
            Some(run) if run.kind == kind => run.text.push(sign),
            _ => runs.push(Run {
                text: sign.to_string(),
                kind,
            }),
        }
    }
    runs
}

/// How `c` is drawn; `leads` tells whether it starts the quality and no letter
/// follows it, which makes `o7` diminished but leaves `omit3` as typed.
fn drawn(c: char, leads: bool) -> (char, RunKind) {
    match c {
        'b' => ('♭', RunKind::Accidental),
        '#' => ('♯', RunKind::Accidental),
        '^' => ('Δ', RunKind::Text),
        'o' if leads => ('°', RunKind::Text),
        'h' if leads => ('ø', RunKind::Text),
        c => (c, RunKind::Text),
    }
}

fn is_letter(c: char) -> bool {
    c.is_ascii_alphabetic()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drawn_runs(quality: &str) -> Vec<(&'static str, String)> {
        quality_runs(quality)
            .into_iter()
            .map(|run| {
                let kind = match run.kind {
                    RunKind::Text => "text",
                    RunKind::Accidental => "music",
                };
                (kind, run.text)
            })
            .collect()
    }

    fn text(text: &str) -> (&'static str, String) {
        ("text", text.to_string())
    }

    fn music(text: &str) -> (&'static str, String) {
        ("music", text.to_string())
    }

    #[test]
    fn flats_and_sharps_switch_to_the_music_font() {
        assert_eq!(drawn_runs("m7b5"), [text("m7"), music("♭"), text("5")]);
        assert_eq!(
            drawn_runs("7(b9#13)"),
            [text("7("), music("♭"), text("9"), music("♯"), text("13)")]
        );
        assert_eq!(
            drawn_runs("7b9#11"),
            [text("7"), music("♭"), text("9"), music("♯"), text("11")]
        );
    }

    #[test]
    fn caret_is_a_triangle_anywhere() {
        assert_eq!(drawn_runs("^7"), [text("Δ7")]);
        assert_eq!(drawn_runs("-^7"), [text("-Δ7")]);
    }

    #[test]
    fn leading_o_and_h_are_diminished_signs_unless_a_word_follows() {
        assert_eq!(drawn_runs("o7"), [text("°7")]);
        assert_eq!(drawn_runs("o"), [text("°")]);
        assert_eq!(drawn_runs("h7"), [text("ø7")]);
        assert_eq!(drawn_runs("omit3"), [text("omit3")]);
        assert_eq!(drawn_runs("7sus4"), [text("7sus4")]);
        assert_eq!(drawn_runs("m7h"), [text("m7h")]);
    }

    #[test]
    fn accidental_signs() {
        assert_eq!(accidental_sign(Accidental::Flat), '♭');
        assert_eq!(accidental_sign(Accidental::Sharp), '♯');
    }
}
