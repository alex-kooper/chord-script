use crate::chart::{Chart, Section, Measure, Chord, Duration};
use anyhow::{Result, bail};

/// Parse a .cchart file into a Chart structure
pub fn parse_chart(content: &str) -> Result<Chart> {
    let mut lines = content.lines().peekable();
    let mut chart = None;
    let mut current_section: Option<Section> = None;

    while let Some(line) = lines.next() {
        let line = line.trim();
        
        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Parse metadata or directives
        if line.starts_with('#') {
            // Comment - skip
            continue;
        } else if line.starts_with("Title:") {
            let title = line.strip_prefix("Title:").unwrap().trim().to_string();
            chart = Some(Chart::new(title));
        } else if line.starts_with("Composer:") {
            let composer = line.strip_prefix("Composer:").unwrap().trim().to_string();
            if let Some(ref mut c) = chart {
                c.composer = Some(composer);
            }
        } else if line.starts_with("Key:") {
            let key = line.strip_prefix("Key:").unwrap().trim().to_string();
            if let Some(ref mut c) = chart {
                c.key = Some(key);
            }
        } else if line.starts_with("Time:") {
            let time = line.strip_prefix("Time:").unwrap().trim().to_string();
            if let Some(ref mut c) = chart {
                c.time_signature = Some(time);
            }
        } else if line.starts_with('[') && line.ends_with(']') {
            // Section header (e.g., [Verse], [Chorus])
            // Save previous section if exists
            if let Some(section) = current_section.take() {
                if let Some(ref mut c) = chart {
                    c.sections.push(section);
                }
            }
            
            let section_name = line.trim_matches(|c| c == '[' || c == ']').to_string();
            current_section = Some(Section {
                name: section_name,
                measures: Vec::new(),
            });
        } else if line.starts_with('|') || line.contains('|') {
            // Measure line
            if current_section.is_none() {
                // Create a default section if none exists
                current_section = Some(Section {
                    name: "Main".to_string(),
                    measures: Vec::new(),
                });
            }

            let measures = parse_measure_line(line)?;
            if let Some(ref mut section) = current_section {
                section.measures.extend(measures);
            }
        }
    }

    // Save last section
    if let Some(section) = current_section {
        if let Some(ref mut c) = chart {
            c.sections.push(section);
        }
    }

    chart.ok_or_else(|| anyhow::anyhow!("No chart title found in file"))
}

/// Parse a line containing measures (e.g., "| Cmaj7 | G7 | Am | Dm7 G7 |")
fn parse_measure_line(line: &str) -> Result<Vec<Measure>> {
    let mut measures = Vec::new();
    
    // Split by | and process each measure
    let parts: Vec<&str> = line.split('|').collect();
    
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Parse chords in this measure
        let chord_strs: Vec<&str> = part.split_whitespace().collect();
        if chord_strs.is_empty() {
            continue;
        }

        let mut chords = Vec::new();
        let duration = match chord_strs.len() {
            1 => Duration::Whole,
            2 => Duration::Half,
            4 => Duration::Quarter,
            _ => Duration::Whole, // Default fallback
        };

        for chord_str in chord_strs {
            if chord_str == "-" || chord_str == "%" {
                // Note: Repeat symbols ("-" and "%") are currently skipped.
                // This is a known limitation - they are not yet implemented.
                // Empty measures will be rendered with a "/" placeholder.
                continue;
            }
            
            let chord = parse_chord(chord_str, duration)?;
            chords.push(chord);
        }

        measures.push(Measure::with_chords(chords));
    }

    Ok(measures)
}

/// Parse a single chord string (e.g., "Cmaj7", "G7", "Am", "Dm")
fn parse_chord(chord_str: &str, duration: Duration) -> Result<Chord> {
    if chord_str.is_empty() {
        bail!("Empty chord string");
    }

    // Extract root note (first character, possibly with accidental)
    let mut chars = chord_str.chars();
    let first = chars.next().unwrap();
    
    if !first.is_ascii_alphabetic() {
        bail!("Chord must start with a letter: {}", chord_str);
    }

    let mut root = first.to_string();
    
    // Check for accidental (# or b)
    if let Some(&next_char) = chord_str.as_bytes().get(1) {
        if next_char == b'#' || next_char == b'b' {
            root.push(next_char as char);
        }
    }

    // Rest is the quality
    let quality_start = root.len();
    let quality = if quality_start < chord_str.len() {
        Some(chord_str[quality_start..].to_string())
    } else {
        None
    };

    let mut chord = Chord::new(root);
    if let Some(q) = quality {
        if !q.is_empty() {
            chord = chord.with_quality(q);
        }
    }
    chord = chord.with_duration(duration);

    Ok(chord)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_chart() {
        let content = r#"
Title: Test Song
Composer: Test Artist
Key: C
Time: 4/4

[Verse]
| Cmaj7 | G7 | Am | Dm7 G7 |

[Chorus]
| F | C | G | C |
"#;
        let chart = parse_chart(content).unwrap();
        assert_eq!(chart.title, "Test Song");
        assert_eq!(chart.composer, Some("Test Artist".to_string()));
        assert_eq!(chart.key, Some("C".to_string()));
        assert_eq!(chart.sections.len(), 2);
        assert_eq!(chart.sections[0].name, "Verse");
        assert_eq!(chart.sections[0].measures.len(), 4);
    }

    #[test]
    fn test_parse_chord() {
        let chord = parse_chord("Cmaj7", Duration::Whole).unwrap();
        assert_eq!(chord.root, "C");
        assert_eq!(chord.quality, Some("maj7".to_string()));

        let chord2 = parse_chord("G#m", Duration::Half).unwrap();
        assert_eq!(chord2.root, "G#");
        assert_eq!(chord2.quality, Some("m".to_string()));
    }
}
