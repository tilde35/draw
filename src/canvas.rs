use crate::blend::{
    ColorAlphaBlendMode, ColorAlphaBlendTransparent, ColorBlendMode, ColorBlendOverwrite,
    ColorBlendTransparent, ImageBlendMode, ImageBlendTransparent,
};
use crate::font::font_cache::FontCache;
use crate::font::glyph::Glyph;
use crate::font::rendered_text::{RenderedText, RenderedTextInstruction};
use crate::img::Image;
use crate::rect::Rect;
use crate::rgba::Rgba;
use crate::rows::{RowsIter, RowsMutIter};
use std::cmp::{max, min};

pub struct Canvas<'a> {
    img: &'a mut Image,
    pos: [i32; 2],
    dim: [u32; 2],
    idx0: usize,
    stride: usize,
}
impl<'a> Canvas<'a> {
    pub(crate) fn new(img: &'a mut Image, idx0: usize, pos: [i32; 2], dim: [u32; 2]) -> Canvas<'a> {
        let stride = img.stride();
        Canvas {
            img,
            pos,
            dim,
            idx0,
            stride,
        }
    }

    /// Location of the top-left corner of this canvas using the coordinate space of the original image.
    pub fn pos(&self) -> [i32; 2] {
        self.pos
    }
    pub fn dim(&self) -> [u32; 2] {
        self.dim
    }
    pub fn width(&self) -> u32 {
        self.dim[0]
    }
    pub fn height(&self) -> u32 {
        self.dim[1]
    }

    pub fn nth_row(&self, n: u32) -> &[Rgba] {
        if n >= self.dim[1] {
            panic!("Row does not exist");
        } else {
            let idx = self.idx0 + self.stride * (n as usize);
            let end = idx + (self.dim[0] as usize);
            &self.img.buffer()[idx..end]
        }
    }

    pub fn nth_row_mut(&mut self, n: u32) -> &mut [Rgba] {
        if n >= self.dim[1] {
            panic!("Row does not exist");
        } else {
            let idx = self.idx0 + self.stride * (n as usize);
            let end = idx + (self.dim[0] as usize);
            &mut self.img.buffer_mut()[idx..end]
        }
    }

    pub fn rows_iter<'b>(&'b self) -> RowsIter<'b> {
        let idx0 = self.idx0;
        let pos0 = self.pos();
        let width = self.dim[0] as usize;
        let stride = self.stride;
        let max_idx = idx0 + (self.dim[1] as usize) * stride;
        unsafe {
            RowsIter::unchecked_from_index(self.img.buffer(), idx0, pos0, width, stride, max_idx)
        }
    }

    pub fn rows_iter_mut<'b>(&'b mut self) -> RowsMutIter<'b> {
        let idx0 = self.idx0;
        let pos0 = self.pos();
        let width = self.dim[0] as usize;
        let stride = self.stride;
        let max_idx = idx0 + (self.dim[1] as usize) * stride;
        unsafe {
            RowsMutIter::unchecked_from_index(
                self.img.buffer_mut(),
                idx0,
                pos0,
                width,
                stride,
                max_idx,
            )
        }
    }

    /// Returns a sub-section of this canvas that overlaps with the specified rectangle. The sub-canvas retains
    /// the same coordinate space as original canvas. If there is no overlap, then None is returned.
    pub fn sub_canvas<'b>(&'b mut self, pos: [i32; 2], dim: [u32; 2]) -> Option<Canvas<'b>> {
        let [x, y] = pos;
        let [width, height] = dim;

        let end_x = x + (width as i32);
        let end_y = y + (height as i32);

        let cur_start_x = self.pos[0];
        let cur_start_y = self.pos[1];
        let cur_end_x = cur_start_x + (self.dim[0] as i32);
        let cur_end_y = cur_start_y + (self.dim[1] as i32);

        let eff_start_x = max(x, cur_start_x);
        let eff_start_y = max(y, cur_start_y);
        let eff_end_x = min(end_x, cur_end_x);
        let eff_end_y = min(end_y, cur_end_y);

        if eff_start_x >= eff_end_x || eff_start_y >= eff_end_y {
            // There is no overlap between the canvases
            None
        } else {
            // Compute the dimensions
            let width = (eff_end_x - eff_start_x) as u32;
            let height = (eff_end_y - eff_start_y) as u32;

            let stride = self.stride;

            // Note: This will be valid since we know the current canvas only contained valid indexes
            let idx0 = self.img.index_at([eff_start_x, eff_start_y]);

            Some(Canvas {
                img: self.img,
                pos: [eff_start_x, eff_start_y],
                dim: [width, height],
                idx0: idx0,
                stride: stride,
            })
        }
    }

    pub fn sub_canvas_rect<'b>(&'b mut self, rect: Rect) -> Option<Canvas<'b>> {
        self.sub_canvas(rect.pos, rect.dim)
    }

    // Performs the same operation as sub_canvas, but takes ownership instead
    pub fn into_sub_canvas(mut self, pos: [i32; 2], dim: [u32; 2]) -> Option<Canvas<'a>> {
        let (pos, dim, idx0) = if let Some(ss) = self.sub_canvas(pos, dim) {
            (ss.pos, ss.dim, ss.idx0)
        } else {
            return None;
        };

        self.pos = pos;
        self.dim = dim;
        self.idx0 = idx0;
        Some(self)
    }

    pub fn into_sub_canvas_rect(self, rect: Rect) -> Option<Canvas<'a>> {
        self.into_sub_canvas(rect.pos, rect.dim)
    }

    fn try_index_at(&self, pos: [i32; 2]) -> Option<usize> {
        let [x, y] = pos;
        let (dx, dy) = (x - self.pos[0], y - self.pos[1]);
        if (dx as u32) < self.dim[0] && (dy as u32) < self.dim[1] {
            Some(self.idx0 + (dx as usize) + (dy as usize) * self.stride)
        } else {
            None
        }
    }

    pub fn try_get_color(&self, pos: [i32; 2]) -> Option<Rgba> {
        if let Some(idx) = self.try_index_at(pos) {
            Some(self.img.get(idx))
        } else {
            None
        }
    }

    pub fn try_get_color_mut(&mut self, pos: [i32; 2]) -> Option<&mut Rgba> {
        if let Some(idx) = self.try_index_at(pos) {
            Some(self.img.get_mut(idx))
        } else {
            None
        }
    }

    pub fn try_set_color(&mut self, pos: [i32; 2], c: Rgba) -> bool {
        if let Some(idx) = self.try_index_at(pos) {
            self.img.set(idx, c);
            true
        } else {
            false
        }
    }

    pub fn try_blend_color(&mut self, pos: [i32; 2], c: Rgba) -> bool {
        self.try_blend_color_using(ColorBlendTransparent, pos, c)
    }

    pub fn try_blend_color_using<Mode: ColorBlendMode>(
        &mut self,
        mode: Mode,
        pos: [i32; 2],
        c: Rgba,
    ) -> bool {
        if let Some(idx) = self.try_index_at(pos) {
            self.img.blend_using(mode, idx, c);
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self, c: Rgba) {
        self.fill_using(ColorBlendOverwrite, c);
    }

    pub fn fill(&mut self, c: Rgba) {
        if c.alpha() == 255 {
            self.fill_using(ColorBlendOverwrite, c);
        } else {
            self.fill_using(ColorBlendTransparent, c);
        }
    }

    pub fn fill_using<Mode: ColorBlendMode>(&mut self, mode: Mode, c: Rgba) {
        let cc = mode.prepare_color(c);
        for row in self.rows_iter_mut() {
            for pixel in row {
                mode.blend_color(pixel, &cc);
            }
        }
    }

    pub fn draw_rect(&mut self, pos: [i32; 2], dim: [u32; 2], c: Rgba) {
        if c.alpha() == 255 {
            // Fully opaque color, overwrite existing content
            self.draw_rect_using(ColorBlendOverwrite, pos, dim, c);
        } else {
            // Use transparent blending
            self.draw_rect_using(ColorBlendTransparent, pos, dim, c);
        }
    }

    pub fn draw_rect_using<Mode: ColorBlendMode>(
        &mut self,
        mode: Mode,
        pos: [i32; 2],
        dim: [u32; 2],
        c: Rgba,
    ) {
        let [x, y] = pos;
        let [w, h] = dim;
        // TODO This is a slow implementation, make it fast
        let x_end = x + (w as i32) - 1;
        let y_end = y + (h as i32) - 1;
        for rx in x..(x_end + 1) {
            self.try_blend_color_using(mode, [rx, y], c);
            self.try_blend_color_using(mode, [rx, y_end], c);
        }
        for ry in (y + 1)..y_end {
            self.try_blend_color_using(mode, [x, ry], c);
            self.try_blend_color_using(mode, [x_end, ry], c);
        }
    }

    pub fn fill_rect(&mut self, pos: [i32; 2], dim: [u32; 2], c: Rgba) {
        if c.alpha() == 255 {
            self.fill_rect_using(ColorBlendOverwrite, pos, dim, c);
        } else {
            self.fill_rect_using(ColorBlendTransparent, pos, dim, c);
        }
    }

    pub fn fill_rect_using<Mode: ColorBlendMode>(
        &mut self,
        mode: Mode,
        pos: [i32; 2],
        dim: [u32; 2],
        c: Rgba,
    ) {
        if let Some(mut sr) = self.sub_canvas(pos, dim) {
            sr.fill_using(mode, c);
        }
    }

    pub fn draw_image(&mut self, img: &Image, pos: [i32; 2]) {
        self.draw_image_using(ImageBlendTransparent, img, pos)
    }

    pub fn draw_image_using<Mode: ImageBlendMode>(
        &mut self,
        mode: Mode,
        img: &Image,
        pos: [i32; 2],
    ) {
        let [x, y] = pos;
        for src_y in 0..img.height() {
            for src_x in 0..img.width() {
                let src = img.get([src_x, src_y]);
                if let Some(dst) = self.try_get_color_mut([x + (src_x as i32), y + (src_y as i32)])
                {
                    mode.blend_color(dst, src);
                }
            }
        }
    }

    pub fn draw_text(
        &mut self,
        font_cache: &mut FontCache,
        font_size: f32,
        font_color: Rgba,
        txt: &str,
        pos: [i32; 2],
        width: Option<u32>,
        indent: Option<u32>,
    ) {
        self.draw_text_using(
            ColorAlphaBlendTransparent,
            font_cache,
            font_size,
            font_color,
            txt,
            pos,
            width,
            indent,
        );
    }

    pub fn draw_text_using<Mode: ColorAlphaBlendMode>(
        &mut self,
        mode: Mode,
        font_cache: &mut FontCache,
        font_size: f32,
        font_color: Rgba,
        txt: &str,
        pos: [i32; 2],
        width: Option<u32>,
        indent: Option<u32>,
    ) {
        let r = font_cache.render(txt, font_size, width, indent);
        self.draw_rendered_text_using(mode, &r, font_color, pos, indent.unwrap_or(0));
    }

    pub fn draw_rendered_text(
        &mut self,
        r: &RenderedText,
        font_color: Rgba,
        pos: [i32; 2],
        indent: u32,
    ) {
        self.draw_rendered_text_using(ColorAlphaBlendTransparent, r, font_color, pos, indent)
    }

    pub fn draw_rendered_text_using<Mode: ColorAlphaBlendMode>(
        &mut self,
        mode: Mode,
        r: &RenderedText,
        font_color: Rgba,
        pos: [i32; 2],
        indent: u32,
    ) {
        let cc = mode.prepare_color(font_color);
        let mut cur_x = pos[0] + (indent as i32);
        let mut cur_y = pos[1];
        for i in r.get_instructions() {
            match *i {
                RenderedTextInstruction::RenderGlyph(ref g) => {
                    self.draw_glyph_alpha_xy(mode, g, &cc, [cur_x, cur_y]);
                    cur_x += g.advance_width;
                }
                RenderedTextInstruction::Kerning(dx) => {
                    cur_x += dx;
                }
                RenderedTextInstruction::NextLine(dy, ..) => {
                    cur_y += dy as i32;
                    cur_x = pos[0];
                }
            }
        }
    }

    fn draw_glyph_alpha_xy<Mode: ColorAlphaBlendMode>(
        &mut self,
        mode: Mode,
        g: &Glyph,
        color_ctxt: &Mode::ColorContext,
        pos: [i32; 2],
    ) {
        g.render_xy(
            pos[0],
            pos[1],
            self,
            |s, x, y| {
                if let Some(dst) = s.try_get_color_mut([x, y]) {
                    mode.blend_solid_color(dst, color_ctxt);
                }
            },
            |s, x, y, alpha| {
                if let Some(dst) = s.try_get_color_mut([x, y]) {
                    mode.blend_color(dst, color_ctxt, alpha);
                }
            },
        );
    }
}
