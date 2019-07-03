use crate::font::chars::RenderableCharacters;
use crate::font::glyph::Glyph;
use crate::font::glyph_builder::GlyphInstructionBuilder;
use crate::font::rendered_text::{NextLineReason, RenderedCharInstruction, RenderedChars, RenderedText, RenderedTextInstruction};
use crate::font::scaled_glyph_cache::ScaledFontCache;
use crate::img::Image;
use crate::rgba::Rgba;
use rusttype::{point, Font, FontCollection, Scale};
use std::cmp::max;
use std::collections::HashMap;

#[derive(Clone)]
pub struct FontCache {
    font: Font<'static>,
    scaled_glyph_cache: HashMap<u64, ScaledFontCache>,
}
impl FontCache {
    pub fn from_static(font_data: &'static [u8]) -> Result<FontCache, rusttype::Error> {
        Self::create(FontCollection::from_bytes(font_data)?)
    }
    pub fn from_slice(font_data: &[u8]) -> Result<FontCache, rusttype::Error> {
        let mut data = Vec::with_capacity(font_data.len());
        data.extend(font_data);
        Self::from_vec(data)
    }
    pub fn from_vec(font_data: Vec<u8>) -> Result<FontCache, rusttype::Error> {
        Self::create(FontCollection::from_bytes(font_data)?)
    }
    pub fn from_font(font: Font<'static>) -> FontCache {
        FontCache {
            font: font,
            scaled_glyph_cache: HashMap::new(),
        }
    }
    fn create(fc: FontCollection<'static>) -> Result<FontCache, rusttype::Error> {
        let font = fc.into_font()?;
        Ok(FontCache {
            font: font,
            scaled_glyph_cache: HashMap::new(),
        })
    }

    fn key_for(size: f32) -> u64 {
        (size * 32.) as u64
    }

    pub fn line_advance_height(&self, font_size: f32) -> u32 {
        let scale = Scale::uniform(font_size);
        let v_metrics = self.font.v_metrics(scale);
        // Note: descent is a negative number
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        advance_height.ceil() as u32
    }

    pub fn dist_to_baseline(&self, font_size: f32) -> u32 {
        let scale = Scale::uniform(font_size);
        let v_metrics = self.font.v_metrics(scale);

        v_metrics.ascent as u32
    }

    pub fn render_chars<RChar>(&mut self, txt: RChar, size: f32, width: Option<u32>) -> RenderedChars
    where
        RChar: RenderableCharacters,
    {
        RenderedChars::from_rendered_text(&self.render(txt, size, width))
    }

    pub fn cache_only_render_chars<RChar>(&self, txt: RChar, size: f32, width: Option<u32>) -> Option<RenderedChars>
    where
        RChar: RenderableCharacters,
    {
        if let Some(t) = self.cache_only_render(txt, size, width) {
            Some(RenderedChars::from_rendered_text(&t))
        } else {
            None
        }
    }

    pub fn chars_to_text<'b>(&'b mut self, chars: &RenderedChars, size: f32) -> RenderedText<'b> {
        unsafe {
            // Current lifetime limitations cause the cache_only_render to stay alive for the entire function
            // The unsafe code may be removed in the future (note: still not working as of Rust 1.35)
            let ptr = self as *mut Self;
            if let Some(r) = (*ptr).cache_only_chars_to_text(chars, size) {
                return r;
            }
        }

        // Note: This is unlikely to happen since generating RenderedChars would have created the required data.
        let mut chars_only = Vec::with_capacity(chars.get_instructions().len());
        for i in chars.get_instructions() {
            if let RenderedCharInstruction::RenderChar { ch, .. } = *i {
                chars_only.push(ch);
            }
        }

        self.preload(&chars_only[..], size);
        self.cache_only_chars_to_text(chars, size).expect("Text preloading failed for font rendering")
    }

    pub fn cache_only_chars_to_text<'b>(&'b self, chars: &RenderedChars, size: f32) -> Option<RenderedText<'b>> {
        let key = Self::key_for(size);
        if let Some(cache) = self.scaled_glyph_cache.get(&key) {
            let mut instructions = Vec::with_capacity(chars.get_instructions().len());
            for i in chars.get_instructions() {
                let ti = match i {
                    RenderedCharInstruction::RenderChar { ch, .. } => {
                        if let Some(g) = cache.get(*ch) {
                            RenderedTextInstruction::RenderGlyph(g)
                        } else {
                            return None;
                        }
                    }
                    RenderedCharInstruction::Kerning(dx) => RenderedTextInstruction::Kerning(*dx),
                    RenderedCharInstruction::NextLine(dy, r) => RenderedTextInstruction::NextLine(*dy, *r),
                };
                instructions.push(ti);
            }
            Some(RenderedText::new(chars.get_total_width(), chars.get_total_height(), instructions))
        } else {
            None
        }
    }

    pub fn render<'b, RChar>(&'b mut self, txt: RChar, size: f32, width: Option<u32>) -> RenderedText<'b>
    where
        RChar: RenderableCharacters,
    {
        unsafe {
            // Current lifetime limitations cause the cache_only_render to stay alive for the entire function
            // The unsafe code may be removed in the future (note: still not working as of Rust 1.35)
            let ptr = self as *mut Self;
            if let Some(r) = (*ptr).cache_only_render(txt, size, width) {
                return r;
            }
        }
        self.preload(txt, size);
        self.cache_only_render(txt, size, width).expect("Text preloading failed for font rendering")
    }

    pub fn cache_only_render<'b, RChar>(&'b self, txt: RChar, size: f32, width: Option<u32>) -> Option<RenderedText<'b>>
    where
        RChar: RenderableCharacters,
    {
        let font = &self.font;
        let max_width = width.unwrap_or(std::u32::MAX);
        let scale = Scale::uniform(size);

        let next_line_wrap = self.line_advance_height(size);
        let next_line_break = next_line_wrap;

        let inst_capacity = {
            let c = txt.len_hint();
            c + c / 4
        };

        let mut inst = Vec::with_capacity(inst_capacity);
        let mut row_width = 0;
        let mut result_width = 0;
        let mut result_height = next_line_wrap;

        let key = Self::key_for(size);
        if let Some(cache) = self.scaled_glyph_cache.get(&key) {
            let mut last_glyph_id = None;

            let mut extra = 0;
            for c in txt.chars_iter() {
                if c.is_control() {
                    match c {
                        '\r' => {}
                        '\n' => {
                            inst.push(RenderedTextInstruction::NextLine(next_line_break, NextLineReason::WordWrap));
                            result_height += next_line_break;
                            if row_width > result_width {
                                result_width = row_width;
                            }
                            row_width = 0;
                        }
                        _ => {}
                    }
                    continue;
                }

                let existing = cache.get(c);
                if existing.is_none() {
                    // Character is missing from cache - abort
                    return None;
                }
                let existing = existing.unwrap();

                if (existing.advance_width as u32) < existing.render_width {
                    extra = existing.render_width - (existing.advance_width as u32);
                } else {
                    extra = 0;
                }

                if let Some(existing_id) = existing.id {
                    if let Some(prev_id) = last_glyph_id {
                        // Compute the font kerning (often the value is zero)
                        let k = font.pair_kerning(scale, prev_id, existing_id).ceil() as i32;
                        if k > 0 {
                            row_width += k as u32;
                            inst.push(RenderedTextInstruction::Kerning(k));
                        }
                    }
                    // Draw the glyph
                    inst.push(RenderedTextInstruction::RenderGlyph(existing));
                    last_glyph_id = existing.id;

                    if row_width + existing.render_width > max_width {
                        // The resulting width will be the max_width (since the end was reached)
                        result_width = max_width;

                        // Move this word to the next line
                        if let Some(break_idx) = cur_line_break_point(&inst) {
                            inst.insert(break_idx, RenderedTextInstruction::NextLine(next_line_wrap, NextLineReason::WordWrap));

                            // Re-compute the row width
                            row_width = 0;
                            for i in &inst[break_idx + 1..] {
                                match i {
                                    &RenderedTextInstruction::RenderGlyph(g) => {
                                        row_width += g.advance_width as u32;
                                    }
                                    &RenderedTextInstruction::Kerning(len) => {
                                        row_width += len as u32;
                                    }
                                    &RenderedTextInstruction::NextLine(..) => {}
                                }
                            }
                        } else {
                            let most_recent = inst.pop().unwrap();
                            inst.push(RenderedTextInstruction::NextLine(next_line_wrap, NextLineReason::WordWrap));
                            inst.push(most_recent);

                            row_width = existing.advance_width as u32;
                        }
                        result_height += next_line_wrap;
                    } else {
                        row_width += existing.advance_width as u32;
                    }
                }
            }

            row_width += extra;
            if row_width > result_width {
                result_width = row_width;
            }

            Some(RenderedText::new(result_width, result_height, inst))
        } else {
            // No entry for this font size
            None
        }
    }

    pub fn preload<RChar>(&mut self, txt: RChar, size: f32)
    where
        RChar: RenderableCharacters,
    {
        let key = Self::key_for(size);
        let font = &self.font;
        let cache = self.scaled_glyph_cache.entry(key).or_insert_with(|| ScaledFontCache::new(size));

        let scale = Scale::uniform(size);
        let v_metrics = font.v_metrics(scale);

        for c in txt.chars_iter() {
            cache.create_if_missing(c, || {
                let base_glyph = font.glyph(c);
                let glyph = base_glyph.scaled(scale);
                let advance_width = glyph.h_metrics().advance_width;
                let mut render_width = 0;

                let mut inst = Vec::new();
                let glyph_id = glyph.id();

                let pos_glyph = glyph.positioned(point(0.0, v_metrics.ascent));
                if let Some(bb) = pos_glyph.pixel_bounding_box() {
                    // Draw the glyph onto an image
                    let (width, height) = (bb.width() as u32, bb.height() as u32);
                    let mut img = Image::new([width, height]);
                    pos_glyph.draw(|x, y, v| {
                        img.set([x, y], Rgba::from_f32([0., 0., 0., v]));
                    });

                    // Note: Some letters have negative offsets - we are ignoring this
                    let x_offset = max(bb.min.x, 0) as u32;
                    let y_offset = max(bb.min.y, 0) as u32;

                    let ib_capacity = (width + x_offset) * (height + y_offset);
                    let mut ib = GlyphInstructionBuilder::with_capacity(ib_capacity as usize);
                    for _ in 0..y_offset {
                        ib.next_row();
                    }
                    for y in 0..height {
                        for _ in 0..x_offset {
                            ib.next_val(0);
                        }
                        for x in 0..width {
                            let v = img.get([x, y]).alpha();
                            ib.next_val(v);
                            /* Future consideration:
                            // Improve rendering speeds by reducing the amount of partial transparancy
                            if v >= 190 { ib.next_val(255); }
                            else if v <= 35 { ib.next_val(0) }
                            else { ib.next_val(v); }
                            // */
                        }
                        ib.next_row();
                    }
                    inst = ib.into_glyph_instructions();
                    render_width = width + x_offset;
                }

                Glyph {
                    instructions: inst,
                    render_width: render_width,
                    advance_width: advance_width.ceil() as i32,
                    id: Some(glyph_id),
                    ch: c,
                }
            });
        }
    }
}

fn cur_line_break_point<'a>(data: &Vec<RenderedTextInstruction<'a>>) -> Option<usize> {
    // Note: Ignore last character
    if data.len() <= 1 {
        return None;
    }
    let mut cur = data.len() - 2;
    while cur > 0 {
        match &data[cur] {
            &RenderedTextInstruction::RenderGlyph(g) => if g.ch == ' ' || g.ch == '-' {
                return Some(cur + 1);
            },
            &RenderedTextInstruction::Kerning(..) => {
                // Ignore - does not apply to line breaks
            }
            &RenderedTextInstruction::NextLine(..) => {
                // Reached line break - no break point found
                return None;
            }
        }
        cur -= 1;
    }
    None
}
