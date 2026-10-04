use super::*;

const LETTERS: [Letter; 7] = [
    Letter::C,
    Letter::D,
    Letter::E,
    Letter::F,
    Letter::G,
    Letter::A,
    Letter::B,
];

/// A note from its ASCII spelling, e.g. `"Bb"`.
fn note(spelling: &str) -> Note {
    let mut chars = spelling.chars();
    let letter = match chars.next() {
        Some('C') => Letter::C,
        Some('D') => Letter::D,
        Some('E') => Letter::E,
        Some('F') => Letter::F,
        Some('G') => Letter::G,
        Some('A') => Letter::A,
        Some('B') => Letter::B,
        _ => panic!("{spelling:?} should start with a note letter"),
    };
    let accidental = match chars.as_str() {
        "" => None,
        "b" => Some(Accidental::Flat),
        "#" => Some(Accidental::Sharp),
        rest => panic!("{rest:?} in {spelling:?} is not an accidental"),
    };
    Note { letter, accidental }
}

fn semitones(value: u8) -> Semitones {
    Semitones::try_new(value).expect("test semitones are within 0..=11")
}

fn all_notes() -> impl Iterator<Item = Note> {
    LETTERS.into_iter().flat_map(|letter| {
        [
            Note::natural(letter),
            Note::flat(letter),
            Note::sharp(letter),
        ]
    })
}

fn transposed(spellings: &[&str], transposition: Transposition) -> Vec<String> {
    spellings
        .iter()
        .map(|spelling| note(spelling).transpose(transposition).to_string())
        .collect()
}

#[test]
fn semitones_are_at_most_eleven() {
    assert!(Semitones::try_new(11).is_ok());
    assert!(Semitones::try_new(12).is_err());
}

#[test]
fn semitones_add_modulo_twelve() {
    for (a, b, sum) in [(11, 1, 0), (10, 4, 2), (0, 0, 0), (5, 6, 11), (11, 11, 10)] {
        assert_eq!(semitones(a) + semitones(b), semitones(sum), "{a} + {b}");
    }
}

#[test]
fn display_is_ascii_spelling() {
    for spelling in ["C", "Bb", "F#", "Cb", "E#"] {
        assert_eq!(note(spelling).to_string(), spelling);
    }
}

#[test]
fn position_wraps_around_c() {
    assert_eq!(note("Bb").semitones_above_c(), semitones(10));
    assert_eq!(note("Cb").semitones_above_c(), semitones(11));
    assert_eq!(note("B#").semitones_above_c(), semitones(0));
}

#[test]
fn difference_is_the_upward_distance() {
    assert_eq!(note("D") - note("C"), semitones(2));
    assert_eq!(note("C") - note("D"), semitones(10));
    assert_eq!(note("G") - note("G"), semitones(0));
    assert_eq!(note("C") - note("B"), semitones(1));
}

#[test]
fn enharmonic_notes_are_zero_apart() {
    for (a, b) in [
        ("A#", "Bb"),
        ("Cb", "B"),
        ("B#", "C"),
        ("E#", "F"),
        ("Fb", "E"),
    ] {
        assert_eq!(note(a) - note(b), semitones(0), "{a} - {b}");
        assert_ne!(note(a), note(b), "{a} and {b} are spelled differently");
    }
}

#[test]
fn direction_picks_the_default_spelling() {
    let shift = |up, spelling| Transposition::Shift {
        up: semitones(up),
        spelling,
    };
    assert_eq!(Transposition::new(2, None), shift(2, Spelling::Sharps));
    assert_eq!(Transposition::new(-10, None), shift(2, Spelling::Flats));
    assert_eq!(Transposition::new(-2, None), shift(10, Spelling::Flats));
    assert_eq!(Transposition::new(14, None), shift(2, Spelling::Sharps));
    assert_eq!(Transposition::new(0, None), Transposition::Unchanged);
}

#[test]
fn explicit_spelling_wins() {
    let shift = |up, spelling| Transposition::Shift {
        up: semitones(up),
        spelling,
    };
    assert_eq!(
        Transposition::new(2, Some(Spelling::Flats)),
        shift(2, Spelling::Flats)
    );
    assert_eq!(
        Transposition::new(0, Some(Spelling::Flats)),
        shift(0, Spelling::Flats)
    );
    assert_eq!(
        Transposition::new(12, Some(Spelling::Flats)),
        shift(0, Spelling::Flats)
    );
}

#[test]
fn whole_octaves_without_spelling_are_unchanged() {
    for octaves in [12, -12, 24, -24] {
        assert_eq!(
            Transposition::new(octaves, None),
            Transposition::Unchanged,
            "{octaves}"
        );
    }
}

#[test]
fn extreme_amounts_wrap_within_the_octave() {
    let shift = |up, spelling| Transposition::Shift {
        up: semitones(up),
        spelling,
    };
    assert_eq!(
        Transposition::new(i32::MAX, None),
        shift(7, Spelling::Sharps)
    );
    assert_eq!(
        Transposition::new(i32::MIN, None),
        shift(4, Spelling::Flats)
    );
}

#[test]
fn chordpro_examples() {
    let notes = ["C", "D", "E", "F"];
    assert_eq!(
        transposed(&notes, Transposition::new(2, None)),
        ["D", "E", "F#", "G"]
    );
    assert_eq!(
        transposed(&notes, Transposition::new(-10, None)),
        ["D", "E", "Gb", "G"]
    );
    assert_eq!(
        transposed(&notes, Transposition::new(2, Some(Spelling::Flats))),
        ["D", "E", "Gb", "G"]
    );
}

#[test]
fn naturals_are_preferred_over_accidentals() {
    assert_eq!(
        transposed(&["E", "B"], Transposition::new(1, None)),
        ["F", "C"]
    );
    assert_eq!(
        transposed(&["F", "C"], Transposition::new(-1, None)),
        ["E", "B"]
    );
}

#[test]
fn zero_shift_with_spelling_only_respells() {
    let to_flats = Transposition::new(0, Some(Spelling::Flats));
    assert_eq!(
        transposed(&["A#", "C", "Cb", "E#"], to_flats),
        ["Bb", "C", "B", "F"]
    );
    let to_sharps = Transposition::new(0, Some(Spelling::Sharps));
    assert_eq!(transposed(&["Bb", "Gb"], to_sharps), ["A#", "F#"]);
}

#[test]
fn unchanged_keeps_rare_spellings() {
    let notes = ["Cb", "Fb", "E#", "B#", "Bb"];
    assert_eq!(transposed(&notes, Transposition::Unchanged), notes);
}

#[test]
fn every_shift_moves_by_its_distance_and_avoids_rare_spellings() {
    let rare = [note("Cb"), note("Fb"), note("E#"), note("B#")];
    for start in all_notes() {
        for up in 0..12 {
            for spelling in [Spelling::Sharps, Spelling::Flats] {
                let transposition = Transposition::new(i32::from(up), Some(spelling));
                let result = start.transpose(transposition);
                assert_eq!(result - start, semitones(up), "{start} up {up}");
                assert!(!rare.contains(&result), "{start} up {up} gave {result}");
            }
        }
    }
}

#[test]
fn twelve_steps_return_to_the_same_pitch() {
    let step = Transposition::new(1, None);
    for start in all_notes() {
        let end = (0..12).fold(start, |current, _| current.transpose(step));
        assert_eq!(end - start, semitones(0), "{start}");
    }
}
