use rusttype::GlyphId;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum GlyphInstruction {
    NextRow,
    NextNRows(u32),
    Offset(u32),
    Blend(u8),
    BlendN(u8, u32),
    Draw,
    DrawN(u32),
}

#[derive(Clone)]
pub struct Glyph {
    pub instructions: Vec<GlyphInstruction>,
    pub render_width: u32,
    pub advance_width: i32,
    pub id: Option<GlyphId>,
    pub ch: char,
}

impl Glyph {
    pub fn render_xy<Ctxt, FDraw, FBlend>(&self, x: i32, y: i32, ctxt: &mut Ctxt, mut draw: FDraw, mut blend: FBlend)
    where
        FDraw: FnMut(&mut Ctxt, i32, i32),
        FBlend: FnMut(&mut Ctxt, i32, i32, u8),
    {
        let mut ix = x;
        let mut iy = y;

        for i in &self.instructions {
            match *i {
                GlyphInstruction::NextRow => {
                    iy += 1;
                    ix = x;
                }
                GlyphInstruction::NextNRows(n) => {
                    iy += n as i32;
                    ix = x;
                }
                GlyphInstruction::Offset(n) => {
                    ix += n as i32;
                }
                GlyphInstruction::Blend(opacity) => {
                    blend(ctxt, ix, iy, opacity);
                    ix += 1;
                }
                GlyphInstruction::BlendN(opacity, n) => for _ in 0..n {
                    blend(ctxt, ix, iy, opacity);
                    ix += 1;
                },
                GlyphInstruction::Draw => {
                    draw(ctxt, ix, iy);
                    ix += 1;
                }
                GlyphInstruction::DrawN(n) => for _ in 0..n {
                    draw(ctxt, ix, iy);
                    ix += 1;
                },
            }
        }
    }

    pub fn render_index<Ctxt, FDraw, FBlend>(&self, start_idx: usize, stride: usize, ctxt: &mut Ctxt, mut draw: FDraw, mut blend: FBlend)
    where
        FDraw: FnMut(&mut Ctxt, usize),
        FBlend: FnMut(&mut Ctxt, usize, u8),
    {
        let mut row0 = start_idx;
        let mut idx = row0;
        for i in &self.instructions {
            match *i {
                GlyphInstruction::NextRow => {
                    row0 += stride;
                    idx = row0;
                }
                GlyphInstruction::NextNRows(n) => {
                    row0 += stride * (n as usize);
                    idx = row0;
                }
                GlyphInstruction::Offset(n) => {
                    idx += n as usize;
                }
                GlyphInstruction::Blend(opacity) => {
                    blend(ctxt, idx, opacity);
                    idx += 1;
                }
                GlyphInstruction::BlendN(opacity, n) => for _ in 0..n {
                    blend(ctxt, idx, opacity);
                    idx += 1;
                },
                GlyphInstruction::Draw => {
                    draw(ctxt, idx);
                    idx += 1;
                }
                GlyphInstruction::DrawN(n) => for _ in 0..n {
                    draw(ctxt, idx);
                    idx += 1;
                },
            }
        }
    }
}
