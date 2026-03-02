use ab_glyph::{point, Font, FontRef, ScaleFont};

pub fn get_font_metrics(font: &FontRef, char_height: f32) -> (f32, f32, f32) {
    let scaled_font = font.as_scaled(char_height);
    let glyph = scaled_font.scaled_glyph('m');
    let advance = scaled_font.h_advance(glyph.id);

    let mut positioned_glyph = glyph;
    positioned_glyph.position = point(0.0, char_height);

    let bounds = if let Some(outlined) = font.outline_glyph(positioned_glyph.clone()) {
        outlined.px_bounds()
    } else {
        ab_glyph::Rect {
            min: ab_glyph::Point { x: 0.0, y: 0.0 },
            max: ab_glyph::Point {
                x: advance,
                y: char_height,
            },
        }
    };

    let glyph_height = bounds.height();
    let aspect = advance / glyph_height;

    (advance, glyph_height, aspect)
}
