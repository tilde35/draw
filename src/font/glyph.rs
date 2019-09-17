use crate::{Canvas, Image, Rgba};
use rusttype::GlyphId;

pub struct Glyph {
    pub inst: Vec<GlyphInst>,
    pub ch: char,
    pub render_width: u32,
    pub advance_width: i32,
    pub id: Option<GlyphId>,
}
impl Glyph {
    pub fn from_image(
        ch: char,
        img: &Image,
        offset: [u32; 2],
        id: Option<GlyphId>,
        advance_width: i32,
    ) -> Self {
        let render_width = img.width() + offset[0];

        let mut builder = InstBuilder::new();
        builder.y_offset(offset[1]);

        for y in 0..img.height() {
            builder.x_offset(offset[0]);

            for x in 0..img.width() {
                let c = img.get([x, y]).alpha();
                builder.draw(c);
            }
            builder.y_offset(1);
        }

        Self {
            inst: builder.inst,
            ch,
            render_width,
            advance_width,
            id,
        }
    }

    pub fn empty(ch: char, id: Option<GlyphId>, advance_width: i32) -> Self {
        Self {
            inst: Vec::new(),
            ch,
            render_width: 0,
            advance_width,
            id,
        }
    }

    pub fn draw(&self, c: &mut Canvas, pos: [i32; 2], color: Rgba) {
        if color.alpha() == 255 {
            self.draw_solid(c, pos, color);
        } else {
            self.draw_alpha(c, pos, color);
        }
    }
    fn draw_solid(&self, c: &mut Canvas, pos: [i32; 2], color: Rgba) {
        let [mut x, mut y] = pos;
        for i in self.inst.iter() {
            match i {
                GlyphInst::NextRow => {
                    y += 1;
                    x = pos[0];
                }
                GlyphInst::NextNRow(n) => {
                    y += (*n) as i32;
                    x = pos[0];
                }
                GlyphInst::XOffset(n) => {
                    x += (*n) as i32;
                }
                GlyphInst::Blend(alpha) => {
                    let mut alpha_color = color;
                    alpha_color.0[3] = *alpha;
                    c.try_blend_color([x, y], alpha_color);
                    x += 1;
                }
                GlyphInst::Solid(n) => {
                    for _ in 0..(*n) {
                        c.try_set_color([x, y], color);
                        x += 1;
                    }
                }
            }
        }
    }
    fn draw_alpha(&self, c: &mut Canvas, pos: [i32; 2], color: Rgba) {
        let base_alpha = color.alpha() as u32;
        let [mut x, mut y] = pos;
        for i in self.inst.iter() {
            match i {
                GlyphInst::NextRow => {
                    y += 1;
                    x = pos[0];
                }
                GlyphInst::NextNRow(n) => {
                    y += (*n) as i32;
                    x = pos[0];
                }
                GlyphInst::XOffset(n) => {
                    x += (*n) as i32;
                }
                GlyphInst::Blend(alpha) => {
                    let rel_alpha = ((*alpha) as u32) * base_alpha / 255;
                    let mut alpha_color = color;
                    alpha_color.0[3] = rel_alpha as u8;
                    c.try_blend_color([x, y], alpha_color);
                    x += 1;
                }
                GlyphInst::Solid(n) => {
                    for _ in 0..(*n) {
                        c.try_blend_color([x, y], color);
                        x += 1;
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum GlyphInst {
    NextRow,
    NextNRow(u32),
    XOffset(u32),
    Blend(u8),
    Solid(u32),
}

struct InstBuilder {
    inst: Vec<GlyphInst>,
    pending_dx: u32,
    pending_dy: u32,
}
impl InstBuilder {
    fn new() -> Self {
        Self {
            inst: Vec::new(),
            pending_dx: 0,
            pending_dy: 0,
        }
    }

    fn y_offset(&mut self, v: u32) {
        self.pending_dy += v;
        self.pending_dx = 0;
    }

    fn x_offset(&mut self, v: u32) {
        self.pending_dx += v;
    }

    fn draw(&mut self, v: u8) {
        if v == 0 {
            self.x_offset(1);
        } else if v == 255 {
            // DrawN
            self.flush_pos();
            let prev = self.inst.pop().unwrap_or(GlyphInst::Solid(0));
            if let GlyphInst::Solid(n) = prev {
                self.inst.push(GlyphInst::Solid(n + 1));
            } else {
                self.inst.push(prev);
                self.inst.push(GlyphInst::Solid(1));
            }
        } else {
            // Blend
            self.flush_pos();
            self.inst.push(GlyphInst::Blend(v));
        }
    }

    fn flush_pos(&mut self) {
        if self.pending_dy != 0 {
            if self.pending_dy == 1 {
                self.inst.push(GlyphInst::NextRow);
            } else {
                self.inst.push(GlyphInst::NextNRow(self.pending_dy));
            }
        }
        if self.pending_dx != 0 {
            self.inst.push(GlyphInst::XOffset(self.pending_dx));
        }

        self.pending_dy = 0;
        self.pending_dx = 0;
    }
}
