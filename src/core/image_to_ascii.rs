use crate::utilities::helper::get_font_metrics;
use crate::utilities::progress_bar::ProgressBar;
use ab_glyph::{Font, FontRef, ScaleFont, point};
use anyhow::Result;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Luma, Rgba};

const CHAR_HEIGHT: u32 = 10;
const ASCII_SCALE: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

pub fn render_image_to_ascii(input_path: &str, output_path: &str) -> Result<()> {
    let font_data = include_bytes!("../../assets/RobotoMono-Regular.ttf");
    let font = FontRef::try_from_slice(font_data)?;
    let (advance, glyph_height, _) = get_font_metrics(&font, CHAR_HEIGHT as f32);

    let img = ImageReader::open(input_path)?.decode()?;
    let (width, height) = img.dimensions();

    // Calculate cols based on width, then rows to maintain aspect ratio
    // Aspect ratio: width/height = (cols * advance) / (rows * glyph_height)
    // Therefore: rows = cols * advance * height / (width * glyph_height)
    let cols = (width as f32 / advance).round().max(1.0) as u32;
    let rows = ((cols as f32 * advance * height as f32) / (width as f32 * glyph_height))
        .round()
        .max(1.0) as u32;

    // Calculate block size based on character counts
    let block_w = width as f32 / cols as f32;
    let block_h = height as f32 / rows as f32;

    let ascii = image_to_ascii(&img, block_w, block_h, cols, rows);

    render_image_to_ascii_core(&ascii, &font, output_path, width, height)?;
    Ok(())
}

pub fn render_image_to_ascii_core(
    ascii: &str,
    font: &FontRef,
    output_path: &str,
    orig_width: u32,
    orig_height: u32,
) -> Result<()> {
    let char_height = CHAR_HEIGHT as f32;
    let (advance, _glyph_height, _) = get_font_metrics(font, char_height);

    // Use original image dimensions for output
    let raw_width = orig_width;
    let raw_height = orig_height;

    let mut img_buf: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(raw_width.max(1), raw_height.max(1), Luma([0]));

    let lines: Vec<&str> = ascii.lines().collect();

    // Scale glyph positions to fit within the output dimensions
    let num_lines = lines.len().max(1) as f32;
    let scaled_glyph_height = raw_height as f32 / num_lines;

    for (row, line) in lines.iter().enumerate() {
        let row_y = row as f32 * scaled_glyph_height;
        let mut caret_x = 0.0f32;

        for ch in line.chars() {
            let mut glyph = font.as_scaled(char_height).scaled_glyph(ch);
            glyph.position = point(caret_x, row_y + scaled_glyph_height);

            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|x, y, coverage| {
                    let px = (bounds.min.x + x as f32) as u32;
                    let py = (bounds.min.y + y as f32) as u32;
                    if px < raw_width && py < raw_height {
                        img_buf.put_pixel(px, py, Luma([(coverage * 255.0) as u8]));
                    }
                });
            }

            caret_x += advance;
        }
    }

    img_buf.save(output_path)?;

    Ok(())
}

pub fn image_to_ascii(
    img: &DynamicImage,
    block_w: f32,
    block_h: f32,
    cols: u32,
    rows: u32,
) -> String {
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();

    // Use the pre-calculated dimensions
    let new_width = cols;
    let new_height = rows;

    let total_blocks = (new_width * new_height) as usize;
    let mut progress_bar = ProgressBar::new(total_blocks).with_message("Processing image blocks");

    let mut buffer = String::with_capacity((new_width * new_height) as usize + new_height as usize);

    for block_y in 0..new_height {
        for block_x in 0..new_width {
            let avg_luma =
                sample_block_luma(&rgba_img, block_x, block_y, block_w, block_h, width, height);
            let ch = match avg_luma {
                Some(luma) => luma_to_char(luma),
                None => ' ',
            };
            buffer.push(ch);
            progress_bar.increment();
        }
        buffer.push('\n');
    }

    progress_bar.finish();
    buffer
}

pub fn sample_block_luma(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    block_x: u32,
    block_y: u32,
    block_w: f32,
    block_h: f32,
    img_width: u32,
    img_height: u32,
) -> Option<u8> {
    let mut total_luma: u32 = 0;
    let mut pixel_count: u32 = 0;
    let mut transparent_pixels: u32 = 0;

    for dy in 0..block_h.ceil() as i32 {
        for dx in 0..block_w.ceil() as i32 {
            let x = (block_x as f32 * block_w + dx as f32) as u32;
            let y = (block_y as f32 * block_h + dy as f32) as u32;

            if x < img_width && y < img_height {
                let pixel = img.get_pixel(x, y);
                let alpha = pixel[3];

                if alpha < 128 {
                    transparent_pixels += 1;
                } else {
                    let luma =
                        0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32;
                    total_luma += luma as u32;
                    pixel_count += 1;
                }
            }
        }
    }

    if transparent_pixels > pixel_count {
        return None;
    }

    if pixel_count == 0 {
        return Some(0);
    }

    Some((total_luma / pixel_count) as u8)
}

pub fn luma_to_char(luma: u8) -> char {
    let threshold_step = 255 / ASCII_SCALE.len() as u8;
    let index = (luma / threshold_step).min((ASCII_SCALE.len() - 1) as u8) as usize;
    ASCII_SCALE[index]
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};
    use tempfile::TempDir;

    fn load_test_font() -> FontRef<'static> {
        let font_data = include_bytes!("../../assets/RobotoMono-Regular.ttf");
        FontRef::try_from_slice(font_data).unwrap()
    }

    fn create_test_image(
        width: u32,
        height: u32,
        color: Rgba<u8>,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        ImageBuffer::from_pixel(width, height, color)
    }

    #[test]
    fn test_luma_to_char_maps_to_correct_characters() {
        // With 10 characters in ASCII_SCALE and threshold_step = 255 / 10 = 25
        // Index 0: 0-24, Index 1: 25-49, Index 2: 50-74, etc.
        assert_eq!(luma_to_char(0), ' ');
        assert_eq!(luma_to_char(24), ' ');
        assert_eq!(luma_to_char(25), '.');
        assert_eq!(luma_to_char(49), '.');
        assert_eq!(luma_to_char(50), ':');
        assert_eq!(luma_to_char(74), ':');
        assert_eq!(luma_to_char(75), '-');
        assert_eq!(luma_to_char(99), '-');
        assert_eq!(luma_to_char(100), '=');
        assert_eq!(luma_to_char(124), '=');
        assert_eq!(luma_to_char(125), '+');
        assert_eq!(luma_to_char(149), '+');
        assert_eq!(luma_to_char(150), '*');
        assert_eq!(luma_to_char(174), '*');
        assert_eq!(luma_to_char(175), '#');
        assert_eq!(luma_to_char(199), '#');
        assert_eq!(luma_to_char(200), '%');
        assert_eq!(luma_to_char(224), '%');
        assert_eq!(luma_to_char(225), '@');
        assert_eq!(luma_to_char(255), '@');
    }

    #[test]
    fn test_sample_block_luma_white_pixels() {
        let white = Rgba([255, 255, 255, 255]);
        let img = create_test_image(100, 100, white);
        let luma = sample_block_luma(&img, 0, 0, 10.0, 10.0, 100, 100);

        assert!(luma.is_some());
        assert!(luma.unwrap() > 250);
    }

    #[test]
    fn test_sample_block_luma_black_pixels() {
        let black = Rgba([0, 0, 0, 255]);
        let img = create_test_image(100, 100, black);
        let luma = sample_block_luma(&img, 0, 0, 10.0, 10.0, 100, 100);

        assert!(luma.is_some());
        assert!(luma.unwrap() < 10);
    }

    #[test]
    fn test_sample_block_luma_gray_pixels() {
        let gray = Rgba([128, 128, 128, 255]);
        let img = create_test_image(100, 100, gray);
        let luma = sample_block_luma(&img, 0, 0, 10.0, 10.0, 100, 100);

        assert!(luma.is_some());
        let value = luma.unwrap();
        assert!(value > 120 && value < 136);
    }

    #[test]
    fn test_sample_block_luma_transparent_pixels() {
        let transparent = Rgba([255, 255, 255, 0]);
        let img = create_test_image(100, 100, transparent);
        let luma = sample_block_luma(&img, 0, 0, 10.0, 10.0, 100, 100);

        assert!(luma.is_none());
    }

    #[test]
    fn test_sample_block_luma_out_of_bounds() {
        let white = Rgba([255, 255, 255, 255]);
        let img = create_test_image(10, 10, white);
        let luma = sample_block_luma(&img, 10, 10, 10.0, 10.0, 10, 10);

        assert!(luma.is_some());
        assert_eq!(luma.unwrap(), 0);
    }

    #[test]
    fn test_sample_block_luma_mixed_pixels() {
        let mut img = create_test_image(100, 100, Rgba([0, 0, 0, 255]));
        // Fill half the image with white
        for y in 0..50 {
            for x in 0..100 {
                img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }

        let luma = sample_block_luma(&img, 0, 0, 100.0, 100.0, 100, 100);

        assert!(luma.is_some());
        let value = luma.unwrap();
        assert!(
            value > 120 && value < 136,
            "Expected mid-range luma, got {}",
            value
        );
    }

    #[test]
    fn test_image_to_ascii_simple() {
        let white = Rgba([255, 255, 255, 255]);
        let img = create_test_image(20, 20, white);
        let dynamic_img = DynamicImage::ImageRgba8(img);

        let ascii = image_to_ascii(&dynamic_img, 10.0, 10.0, 2, 2);

        assert_eq!(ascii.lines().count(), 2);
        assert!(ascii.contains('@'));
    }

    #[test]
    fn test_image_to_ascii_black_image() {
        let black = Rgba([0, 0, 0, 255]);
        let img = create_test_image(20, 20, black);
        let dynamic_img = DynamicImage::ImageRgba8(img);

        let ascii = image_to_ascii(&dynamic_img, 10.0, 10.0, 2, 2);

        assert_eq!(ascii.lines().count(), 2);
        assert!(ascii.contains(' '));
    }

    #[test]
    fn test_image_to_ascii_preserves_dimensions() {
        let white = Rgba([255, 255, 255, 255]);
        let img = create_test_image(100, 100, white);
        let dynamic_img = DynamicImage::ImageRgba8(img);

        let cols = 10;
        let rows = 10;
        let ascii = image_to_ascii(&dynamic_img, 10.0, 10.0, cols, rows);

        let lines: Vec<&str> = ascii.lines().collect();
        assert_eq!(lines.len(), rows as usize);
        for line in &lines {
            assert_eq!(line.len(), cols as usize);
        }
    }

    #[test]
    fn test_render_image_to_ascii_core() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_output.png");
        let font = load_test_font();

        let ascii = "@@@@\n@@@@\n";
        let result =
            render_image_to_ascii_core(ascii, &font, output_path.to_str().unwrap(), 100, 100);

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_render_image_to_ascii_core_with_newlines() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_output.png");
        let font = load_test_font();

        let ascii = "@@@\n@@@\n@@@\n";
        let result =
            render_image_to_ascii_core(ascii, &font, output_path.to_str().unwrap(), 100, 100);

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_ascii_scale_length() {
        assert_eq!(ASCII_SCALE.len(), 10);
        assert_eq!(ASCII_SCALE[0], ' ');
        assert_eq!(ASCII_SCALE[ASCII_SCALE.len() - 1], '@');
    }
}
