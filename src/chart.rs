/// Represents a complete chord chart
#[derive(Debug, Clone)]
pub struct Chart {
    pub title: String,
    pub composer: Option<String>,
    pub key: Option<String>,
    pub time_signature: Option<String>,
    pub sections: Vec<Section>,
}

/// Represents a section of the chart (e.g., Verse, Chorus, Bridge)
#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub measures: Vec<Measure>,
}

/// Represents a single measure in the chart
#[derive(Debug, Clone)]
pub struct Measure {
    pub chords: Vec<Chord>,
}

/// Represents a chord
#[derive(Debug, Clone)]
pub struct Chord {
    pub root: String,
    pub quality: Option<String>,
    pub duration: Duration,
}

/// Duration of a chord within a measure
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Duration {
    Whole,
    Half,
    Quarter,
}

impl Chart {
    pub fn new(title: String) -> Self {
        Self {
            title,
            composer: None,
            key: None,
            time_signature: Some("4/4".to_string()),
            sections: Vec::new(),
        }
    }
}

impl Measure {
    pub fn new() -> Self {
        Self {
            chords: Vec::new(),
        }
    }

    pub fn with_chords(chords: Vec<Chord>) -> Self {
        Self { chords }
    }
}

impl Default for Measure {
    fn default() -> Self {
        Self::new()
    }
}

impl Chord {
    pub fn new(root: String) -> Self {
        Self {
            root,
            quality: None,
            duration: Duration::Whole,
        }
    }

    pub fn with_quality(mut self, quality: String) -> Self {
        self.quality = Some(quality);
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Returns the full chord name (e.g., "Cmaj7", "G7", "Am")
    pub fn full_name(&self) -> String {
        if let Some(quality) = &self.quality {
            format!("{}{}", self.root, quality)
        } else {
            self.root.clone()
        }
    }
}
