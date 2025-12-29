use crate::chart::{Chart, Section, Measure, Chord};
use anyhow::Result;
use svg::node::element::{Rectangle, Text, Line, Group};
use svg::Document;

const PAGE_WIDTH: f32 = 800.0;
const PAGE_HEIGHT: f32 = 1000.0;
const MARGIN: f32 = 40.0;
const MEASURE_WIDTH: f32 = 180.0;
const MEASURE_HEIGHT: f32 = 100.0;
const MEASURES_PER_ROW: usize = 4;

pub fn render_to_svg(chart: &Chart) -> Result<String> {
    let mut document = Document::new()
        .set("width", PAGE_WIDTH)
        .set("height", PAGE_HEIGHT)
        .set("viewBox", (0, 0, PAGE_WIDTH as i32, PAGE_HEIGHT as i32));

    // Add white background
    let background = Rectangle::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("fill", "white");
    document = document.add(background);

    let mut y_offset = MARGIN;

    // Render title
    let title_text = Text::new(&chart.title)
        .set("x", PAGE_WIDTH / 2.0)
        .set("y", y_offset)
        .set("text-anchor", "middle")
        .set("font-size", 28)
        .set("font-weight", "bold")
        .set("font-family", "Arial, sans-serif");
    document = document.add(title_text);
    y_offset += 40.0;

    // Render metadata
    if let Some(ref composer) = chart.composer {
        let composer_text = Text::new(composer)
            .set("x", PAGE_WIDTH / 2.0)
            .set("y", y_offset)
            .set("text-anchor", "middle")
            .set("font-size", 16)
            .set("font-family", "Arial, sans-serif");
        document = document.add(composer_text);
        y_offset += 25.0;
    }

    // Render key and time signature
    let mut meta_parts = Vec::new();
    if let Some(ref key) = chart.key {
        meta_parts.push(format!("Key: {}", key));
    }
    if let Some(ref time) = chart.time_signature {
        meta_parts.push(format!("Time: {}", time));
    }
    if !meta_parts.is_empty() {
        let meta_text = Text::new(&meta_parts.join(" • "))
            .set("x", PAGE_WIDTH / 2.0)
            .set("y", y_offset)
            .set("text-anchor", "middle")
            .set("font-size", 14)
            .set("font-family", "Arial, sans-serif");
        document = document.add(meta_text);
        y_offset += 35.0;
    }

    // Render sections
    for section in &chart.sections {
        let section_group = render_section(section, &mut y_offset);
        document = document.add(section_group);
    }

    let svg_string = document.to_string();
    Ok(svg_string)
}

fn render_section(section: &Section, y_offset: &mut f32) -> Group {
    let mut group = Group::new();

    // Section header
    let section_text = Text::new(&section.name)
        .set("x", MARGIN)
        .set("y", *y_offset)
        .set("font-size", 18)
        .set("font-weight", "bold")
        .set("font-family", "Arial, sans-serif");
    group = group.add(section_text);
    *y_offset += 30.0;

    // Render measures in rows
    let mut measure_index = 0;
    while measure_index < section.measures.len() {
        let row_y = *y_offset;
        
        for col in 0..MEASURES_PER_ROW {
            if measure_index >= section.measures.len() {
                break;
            }

            let measure = &section.measures[measure_index];
            let x = MARGIN + (col as f32) * MEASURE_WIDTH;
            
            let measure_group = render_measure(measure, x, row_y);
            group = group.add(measure_group);
            
            measure_index += 1;
        }

        *y_offset += MEASURE_HEIGHT + 10.0;
    }

    *y_offset += 20.0; // Space between sections

    group
}

fn render_measure(measure: &Measure, x: f32, y: f32) -> Group {
    let mut group = Group::new();

    // Measure box
    let rect = Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", MEASURE_WIDTH - 5.0)
        .set("height", MEASURE_HEIGHT - 5.0)
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 2);
    group = group.add(rect);

    // Render chords in the measure
    let chord_count = measure.chords.len();
    if chord_count == 0 {
        return group;
    }

    // Layout chords based on their count
    match chord_count {
        1 => {
            // Single chord - center it
            let chord = &measure.chords[0];
            let chord_text = render_chord_text(chord, x + MEASURE_WIDTH / 2.0, y + MEASURE_HEIGHT / 2.0);
            group = group.add(chord_text);
        }
        2 => {
            // Two chords - side by side
            let chord1 = &measure.chords[0];
            let chord2 = &measure.chords[1];
            
            let chord_text1 = render_chord_text(chord1, x + MEASURE_WIDTH / 4.0, y + MEASURE_HEIGHT / 2.0);
            let chord_text2 = render_chord_text(chord2, x + 3.0 * MEASURE_WIDTH / 4.0, y + MEASURE_HEIGHT / 2.0);
            
            // Divider line
            let divider = Line::new()
                .set("x1", x + MEASURE_WIDTH / 2.0)
                .set("y1", y + 10.0)
                .set("x2", x + MEASURE_WIDTH / 2.0)
                .set("y2", y + MEASURE_HEIGHT - 15.0)
                .set("stroke", "black")
                .set("stroke-width", 1)
                .set("stroke-dasharray", "3,3");
            
            group = group.add(divider);
            group = group.add(chord_text1);
            group = group.add(chord_text2);
        }
        3 | 4 => {
            // Four chords - 2x2 grid (or 3 chords with one empty)
            for (i, chord) in measure.chords.iter().enumerate() {
                let col = i % 2;
                let row = i / 2;
                
                let chord_x = x + (col as f32 + 0.5) * MEASURE_WIDTH / 2.0;
                let chord_y = y + (row as f32 + 0.5) * MEASURE_HEIGHT / 2.0;
                
                let chord_text = render_chord_text(chord, chord_x, chord_y);
                group = group.add(chord_text);
            }

            // Grid lines
            let h_line = Line::new()
                .set("x1", x + 10.0)
                .set("y1", y + MEASURE_HEIGHT / 2.0)
                .set("x2", x + MEASURE_WIDTH - 15.0)
                .set("y2", y + MEASURE_HEIGHT / 2.0)
                .set("stroke", "black")
                .set("stroke-width", 1)
                .set("stroke-dasharray", "3,3");
            
            let v_line = Line::new()
                .set("x1", x + MEASURE_WIDTH / 2.0)
                .set("y1", y + 10.0)
                .set("x2", x + MEASURE_WIDTH / 2.0)
                .set("y2", y + MEASURE_HEIGHT - 15.0)
                .set("stroke", "black")
                .set("stroke-width", 1)
                .set("stroke-dasharray", "3,3");
            
            group = group.add(h_line);
            group = group.add(v_line);
        }
        _ => {
            // More than 4 chords - just center the first one
            let chord = &measure.chords[0];
            let chord_text = render_chord_text(chord, x + MEASURE_WIDTH / 2.0, y + MEASURE_HEIGHT / 2.0);
            group = group.add(chord_text);
        }
    }

    group
}

fn render_chord_text(chord: &Chord, x: f32, y: f32) -> Text {
    Text::new(&chord.full_name())
        .set("x", x)
        .set("y", y)
        .set("text-anchor", "middle")
        .set("dominant-baseline", "middle")
        .set("font-size", 20)
        .set("font-weight", "bold")
        .set("font-family", "Arial, sans-serif")
}
