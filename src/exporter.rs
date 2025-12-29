use anyhow::{Context, Result};
use std::path::Path;
use image::{RgbaImage, DynamicImage};

/// Export SVG content to PNG format
pub fn export_png(svg_content: &str, output_path: &Path) -> Result<()> {
    // Parse SVG
    let tree = usvg::Tree::from_str(svg_content, &usvg::Options::default())
        .context("Failed to parse SVG")?;

    // Get SVG dimensions
    let size = tree.size();
    let width = size.width() as u32;
    let height = size.height() as u32;

    // Create a pixmap
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .context("Failed to create pixmap")?;

    // Render SVG to pixmap
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // Save as PNG
    pixmap.save_png(output_path)
        .context("Failed to save PNG file")?;

    Ok(())
}

/// Export SVG content to PDF format
pub fn export_pdf(svg_content: &str, output_path: &Path) -> Result<()> {
    use printpdf::*;

    // Parse SVG to get dimensions
    let tree = usvg::Tree::from_str(svg_content, &usvg::Options::default())
        .context("Failed to parse SVG")?;

    let size = tree.size();
    let width_mm = (size.width() * 0.264583) as f32; // Convert pixels to mm
    let height_mm = (size.height() * 0.264583) as f32;

    // Create PDF document
    let (doc, page1, layer1) = PdfDocument::new(
        "Chord Chart",
        Mm(width_mm),
        Mm(height_mm),
        "Layer 1"
    );

    // First, render SVG to PNG in memory
    let width = size.width() as u32;
    let height = size.height() as u32;
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .context("Failed to create pixmap for PDF")?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // Convert pixmap to image
    let image_data = pixmap.data();
    
    // Create image for PDF using the image crate
    let img = RgbaImage::from_raw(width, height, image_data.to_vec())
        .context("Failed to create image from pixmap")?;
    let dynamic_img = DynamicImage::ImageRgba8(img);

    // Create image for PDF
    let image = Image::from_dynamic_image(&dynamic_img);

    // Add image to PDF
    let current_layer = doc.get_page(page1).get_layer(layer1);
    image.add_to_layer(
        current_layer.clone(),
        ImageTransform {
            translate_x: Some(Mm(0.0)),
            translate_y: Some(Mm(0.0)),
            scale_x: Some(width_mm / width as f32),
            scale_y: Some(height_mm / height as f32),
            ..Default::default()
        }
    );

    // Save PDF
    let file = std::fs::File::create(output_path)
        .context("Failed to create PDF file")?;
    doc.save(&mut std::io::BufWriter::new(file))
        .context("Failed to save PDF")?;

    Ok(())
}
