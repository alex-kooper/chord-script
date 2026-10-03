//! Where a chart's output goes and in which format.

use anyhow::{Result, bail};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Format {
    Pdf,
    Svg,
}

impl Format {
    fn of(path: &Path) -> Result<Self> {
        let Some(extension) = path.extension() else {
            bail!(
                "output '{}' has no extension; use .pdf or .svg",
                path.display()
            );
        };
        match extension.to_string_lossy().to_ascii_lowercase().as_str() {
            "pdf" => Ok(Self::Pdf),
            "svg" => Ok(Self::Svg),
            other => bail!(
                "unsupported output format '.{other}' for '{}'; use .pdf or .svg",
                path.display()
            ),
        }
    }
}

/// The resolved output of one chart: a path and the format it implies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Output {
    path: PathBuf,
    format: Format,
}

impl Output {
    /// The requested path, or a PDF next to the input.
    pub(super) fn resolve(input: &Path, requested: Option<&Path>) -> Result<Self> {
        let path = requested.map_or_else(|| input.with_extension("pdf"), Path::to_path_buf);
        let format = Format::of(&path)?;
        Ok(Self { path, format })
    }

    pub(super) fn format(&self) -> Format {
        self.format
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    /// One path per page: the output path itself for a single page,
    /// `name-1.ext`, `name-2.ext`, ... otherwise.
    pub(super) fn page_paths(&self, count: usize) -> Vec<PathBuf> {
        assert!(count > 0, "a chart always renders at least one page");
        if count == 1 {
            return vec![self.path.clone()];
        }
        (1..=count).map(|page| self.numbered(page)).collect()
    }

    fn numbered(&self, page: usize) -> PathBuf {
        let mut name = OsString::from(self.path.file_stem().unwrap_or_default());
        name.push(format!("-{page}"));
        if let Some(extension) = self.path.extension() {
            name.push(".");
            name.push(extension);
        }
        self.path.with_file_name(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve(input: &str, requested: Option<&str>) -> Result<Output> {
        Output::resolve(Path::new(input), requested.map(Path::new))
    }

    fn error(input: &str, requested: &str) -> String {
        resolve(input, Some(requested))
            .expect_err("output should be rejected")
            .to_string()
    }

    #[test]
    fn default_output_is_a_pdf_next_to_the_input() {
        let output = resolve("songs/song.chords", None).expect("default output");
        assert_eq!(output.path(), Path::new("songs/song.pdf"));
        assert_eq!(output.format(), Format::Pdf);
    }

    #[test]
    fn input_without_extension_gets_a_pdf_extension() {
        let output = resolve("song", None).expect("default output");
        assert_eq!(output.path(), Path::new("song.pdf"));
    }

    #[test]
    fn requested_extension_selects_the_format() {
        for (requested, format) in [
            ("out.pdf", Format::Pdf),
            ("out.svg", Format::Svg),
            ("OUT.SVG", Format::Svg),
        ] {
            let output = resolve("song.chords", Some(requested)).expect("supported format");
            assert_eq!(output.path(), Path::new(requested));
            assert_eq!(output.format(), format, "{requested}");
        }
    }

    #[test]
    fn unsupported_or_missing_extension_is_an_error() {
        assert!(error("song.chords", "out.png").contains("unsupported output format '.png'"));
        assert!(error("song.chords", "out").contains("has no extension"));
    }

    #[test]
    fn single_page_keeps_the_output_name() {
        let output = resolve("song.chords", Some("out/song.svg")).expect("svg output");
        assert_eq!(output.page_paths(1), [PathBuf::from("out/song.svg")]);
    }

    #[test]
    fn multiple_pages_are_numbered_from_one() {
        let output = resolve("song.chords", Some("out/song.svg")).expect("svg output");
        assert_eq!(
            output.page_paths(2),
            [
                PathBuf::from("out/song-1.svg"),
                PathBuf::from("out/song-2.svg")
            ]
        );
    }

    #[test]
    #[should_panic(expected = "at least one page")]
    fn zero_pages_is_a_defect() {
        let output = resolve("song.chords", Some("song.svg")).expect("svg output");
        output.page_paths(0);
    }
}
