use crate::font::glyph::*;
use crate::Image;
use std::collections::HashMap;

pub struct SvgFont {
    chars: HashMap<char, (nsvg::SvgImage, f32)>,
}
// TODO Determine if it is actually okay for SvgImage to implement Send+Sync
unsafe impl Send for SvgFont {}
unsafe impl Sync for SvgFont {}
impl SvgFont {
    pub fn new(chars: HashMap<char, (nsvg::SvgImage, f32)>) -> Self {
        Self { chars }
    }

    pub fn line_height(&self, font_size: u32) -> u32 {
        font_size
    }

    pub fn line_advance_height(&self, font_size: u32) -> u32 {
        (5 * font_size) / 4
    }

    pub fn dist_to_baseline(&self, font_size: u32) -> u32 {
        (3 * font_size) / 4
    }

    pub fn create_glyph(&self, font_size: u32, ch: char) -> Glyph {
        if let Some((svg_img, factor)) = self.chars.get(&ch) {
            let scale = (font_size as f32) * (*factor);

            let (width, height, raw_rgba) = svg_img.rasterize_to_raw_rgba(scale).unwrap();
            let img = Image::from_raw_rgba_bytes([width, height], &raw_rgba);

            let offset = [0, 0];

            let advance_width = width;

            Glyph::from_image(ch, &img, offset, None, advance_width as i32)
        } else {
            let advance_width = 0;
            Glyph::empty(ch, None, advance_width)
        }
    }

    pub fn kerning_for(&self, font_size: u32, first: &Glyph, second: &Glyph) -> i32 {
        // Kerning for SVG fonts is not provided at this time
        let _ = (font_size, first, second);
        0
    }
}
