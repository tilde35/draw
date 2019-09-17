use crate::font::glyph::Glyph;
use crate::{Image, Rgba};
use rusttype::{point, Font, FontCollection, Scale};
use std::cmp::max;

pub struct TtfFont {
    font: Font<'static>,
}
impl TtfFont {
    pub fn from_static(font_data: &'static [u8]) -> Result<Self, rusttype::Error> {
        Self::create(FontCollection::from_bytes(font_data)?)
    }
    pub fn from_vec(font_data: Vec<u8>) -> Result<Self, rusttype::Error> {
        Self::create(FontCollection::from_bytes(font_data)?)
    }
    pub fn from_font(font: Font<'static>) -> Self {
        Self { font }
    }
    fn create(fc: FontCollection<'static>) -> Result<Self, rusttype::Error> {
        let font = fc.into_font()?;
        Ok(Self { font })
    }

    pub fn line_height(&self, font_size: u32) -> u32 {
        let scale = Scale::uniform(font_size as f32);
        let v_metrics = self.font.v_metrics(scale);
        // Note: descent is a negative number
        let line_height = v_metrics.ascent - v_metrics.descent;

        line_height.ceil() as u32
    }

    pub fn line_advance_height(&self, font_size: u32) -> u32 {
        let scale = Scale::uniform(font_size as f32);
        let v_metrics = self.font.v_metrics(scale);
        // Note: descent is a negative number
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        advance_height.ceil() as u32
    }

    pub fn dist_to_baseline(&self, font_size: u32) -> u32 {
        let scale = Scale::uniform(font_size as f32);
        let v_metrics = self.font.v_metrics(scale);

        v_metrics.ascent as u32
    }

    pub fn create_glyph(&self, font_size: u32, ch: char) -> Glyph {
        let scale = Scale::uniform(font_size as f32);
        let v_metrics = self.font.v_metrics(scale);
        let base_glyph = self.font.glyph(ch);
        let glyph = base_glyph.scaled(scale);
        let advance_width = glyph.h_metrics().advance_width.ceil() as i32;
        let glyph_id = glyph.id();
        let pos_glyph = glyph.positioned(point(0.0, v_metrics.ascent));
        if let Some(bb) = pos_glyph.pixel_bounding_box() {
            // Draw the glyph onto an image
            let (width, height) = (bb.width() as u32, bb.height() as u32);
            let mut img = Image::new([width, height]);
            pos_glyph.draw(|x, y, v| {
                img.set([x, y], Rgba::from_f32([0., 0., 0., v]));
            });

            // Note: Some letters have negative offsets - ignore this for now
            let x_offset = max(bb.min.x, 0) as u32;
            let y_offset = max(bb.min.y, 0) as u32;

            Glyph::from_image(
                ch,
                &img,
                [x_offset, y_offset],
                Some(glyph_id),
                advance_width,
            )
        } else {
            Glyph::empty(ch, Some(glyph_id), advance_width)
        }
    }

    pub fn kerning_for(&self, font_size: u32, first: &Glyph, second: &Glyph) -> i32 {
        let scale = Scale::uniform(font_size as f32);
        if let (Some(first_id), Some(second_id)) = (first.id, second.id) {
            self.font.pair_kerning(scale, first_id, second_id).ceil() as i32
        } else {
            0
        }
    }
}
