//! The `chords` binary: arguments, output files, diagnostics, and exit codes.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

const ONE_PAGE: &str = "=== Song\n= Verse <> *Am*\n";
const TWO_PAGES: &str = "= Verse\n#page_break\n= Chorus\n";

/// A temporary directory holding the given chart files.
fn workspace(files: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().expect("temporary directory");
    for (name, contents) in files {
        fs::write(dir.path().join(name), contents).expect("write chart");
    }
    dir
}

fn chords(dir: &TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_chords"))
        .args(args)
        .current_dir(dir.path())
        .output()
        .expect("run chords")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn exists(dir: &TempDir, name: &str) -> bool {
    dir.path().join(name).exists()
}

fn read(dir: &TempDir, name: &str) -> Vec<u8> {
    fs::read(dir.path().join(name)).unwrap_or_else(|e| panic!("read {name}: {e}"))
}

#[test]
fn default_output_is_a_pdf_next_to_the_input() {
    let dir = workspace(&[("song.chords", ONE_PAGE)]);
    let output = chords(&dir, &["song.chords"]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(output.stdout.is_empty(), "success is silent");
    assert!(read(&dir, "song.pdf").starts_with(b"%PDF-"));
}

#[test]
fn single_page_svg_keeps_the_output_name() {
    let dir = workspace(&[("song.chords", ONE_PAGE)]);
    let output = chords(&dir, &["song.chords", "-o", "song.svg"]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(read(&dir, "song.svg").starts_with(b"<svg"));
    assert!(!exists(&dir, "song-1.svg"));
}

#[test]
fn multi_page_svg_is_numbered_from_one() {
    let dir = workspace(&[("song.chords", TWO_PAGES)]);
    let output = chords(&dir, &["song.chords", "--output", "song.svg"]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(String::from_utf8_lossy(&read(&dir, "song-1.svg")).contains("Verse"));
    assert!(String::from_utf8_lossy(&read(&dir, "song-2.svg")).contains("Chorus"));
    assert!(!exists(&dir, "song.svg"));
}

#[test]
fn parse_errors_are_reported_against_the_source() {
    let dir = workspace(&[("bad.chords", "= *unclosed\n")]);
    let output = chords(&dir, &["bad.chords"]);
    let stderr = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("bad.chords:1:"), "{stderr}");
    assert!(
        !stderr.contains('\u{1B}'),
        "no colour when stderr is not a terminal"
    );
    assert!(!exists(&dir, "bad.pdf"));
}

#[test]
fn batch_continues_past_a_failing_chart() {
    let dir = workspace(&[("bad.chords", "= *unclosed\n"), ("good.chords", ONE_PAGE)]);
    let output = chords(&dir, &["bad.chords", "good.chords"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("bad.chords"));
    assert!(exists(&dir, "good.pdf"));
}

#[test]
fn output_with_several_inputs_is_a_usage_error() {
    let dir = workspace(&[("a.chords", ONE_PAGE), ("b.chords", ONE_PAGE)]);
    let output = chords(&dir, &["a.chords", "b.chords", "-o", "out.pdf"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("single input"));
    assert!(!exists(&dir, "out.pdf"));
}

#[test]
fn missing_input_is_a_usage_error() {
    let output = chords(&workspace(&[]), &[]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn unsupported_output_format_is_an_error() {
    let dir = workspace(&[("song.chords", ONE_PAGE)]);
    let output = chords(&dir, &["song.chords", "-o", "song.png"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("unsupported output format '.png'"));
}

#[test]
fn unreadable_input_names_the_file() {
    let dir = workspace(&[]);
    let output = chords(&dir, &["missing.chords"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("reading 'missing.chords'"));
}

#[test]
fn output_never_overwrites_the_input() {
    let dir = workspace(&[("song.pdf", ONE_PAGE)]);
    let output = chords(&dir, &["song.pdf"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("would overwrite the input"));
    assert_eq!(read(&dir, "song.pdf"), ONE_PAGE.as_bytes());
}

#[test]
fn output_directory_is_respected() {
    let dir = workspace(&[("song.chords", ONE_PAGE)]);
    fs::create_dir(dir.path().join("out")).expect("create output directory");
    let output = chords(&dir, &["song.chords", "-o", "out/song.pdf"]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(Path::new(&dir.path().join("out/song.pdf")).exists());
}
