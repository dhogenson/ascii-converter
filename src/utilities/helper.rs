use ab_glyph::{Font, FontRef, ScaleFont, point};

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

#[cfg(test)]
mod tests {
    use super::*;

    fn load_test_font() -> FontRef<'static> {
        let font_data = include_bytes!("../../assets/RobotoMono-Regular.ttf");
        FontRef::try_from_slice(font_data).unwrap()
    }

    #[test]
    fn test_get_font_metrics_returns_positive_values() {
        let font = load_test_font();
        let (advance, glyph_height, aspect) = get_font_metrics(&font, 10.0);

        assert!(advance > 0.0, "advance should be positive");
        assert!(glyph_height > 0.0, "glyph_height should be positive");
        assert!(aspect > 0.0, "aspect should be positive");
    }

    #[test]
    fn test_get_font_metrics_aspect_ratio_calculation() {
        let font = load_test_font();
        let (advance, glyph_height, aspect) = get_font_metrics(&font, 10.0);

        let expected_aspect = advance / glyph_height;
        assert!(
            (aspect - expected_aspect).abs() < f32::EPSILON,
            "aspect ratio should equal advance / glyph_height"
        );
    }

    #[test]
    fn test_get_font_metrics_different_heights() {
        let font = load_test_font();

        let (advance_small, height_small, _) = get_font_metrics(&font, 8.0);
        let (advance_large, height_large, _) = get_font_metrics(&font, 16.0);

        // Larger font size should result in larger metrics
        assert!(
            height_large > height_small,
            "larger char_height should produce larger glyph_height"
        );
        assert!(
            advance_large > advance_small,
            "larger char_height should produce larger advance"
        );
    }

    #[test]
    fn test_get_font_metrics_proportional_scaling() {
        let font = load_test_font();

        let (_, height_10, aspect_10) = get_font_metrics(&font, 10.0);
        let (_, height_20, aspect_20) = get_font_metrics(&font, 20.0);

        // Aspect ratio should remain roughly constant when scaling
        assert!(
            (aspect_10 - aspect_20).abs() < 0.2,
            "aspect ratio should remain consistent when scaling font size"
        );

        // Height should scale proportionally (within reasonable bounds)
        let height_ratio = height_20 / height_10;
        assert!(
            (height_ratio - 2.0).abs() < 0.5,
            "doubling char_height should roughly double glyph_height"
        );
    }
}
