use crate::font::glyph::Glyph;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum NextLineReason {
    WordWrap,
    LineBreak,
}

pub enum RenderedTextInstruction<'a> {
    RenderGlyph(&'a Glyph),
    Kerning(i32),
    NextLine(u32, NextLineReason),
}

pub struct RenderedText<'a> {
    total_width: u32,
    total_height: u32,
    instructions: Vec<RenderedTextInstruction<'a>>,
}
impl<'a> RenderedText<'a> {
    pub fn new(
        total_width: u32,
        total_height: u32,
        instructions: Vec<RenderedTextInstruction<'a>>,
    ) -> RenderedText<'a> {
        RenderedText {
            total_width: total_width,
            total_height: total_height,
            instructions: instructions,
        }
    }

    pub fn get_total_width(&self) -> u32 {
        self.total_width
    }
    pub fn get_total_height(&self) -> u32 {
        self.total_height
    }
    pub fn get_instructions<'b>(&'b self) -> &'b Vec<RenderedTextInstruction<'a>> {
        &self.instructions
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum RenderedCharInstruction {
    RenderChar {
        ch: char,
        render_width: u32,
        advance_width: i32,
    },
    Kerning(i32),
    NextLine(u32, NextLineReason),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct RenderedChars {
    total_width: u32,
    total_height: u32,
    instructions: Vec<RenderedCharInstruction>,
}
impl RenderedChars {
    pub fn new(
        total_width: u32,
        total_height: u32,
        instructions: Vec<RenderedCharInstruction>,
    ) -> Self {
        Self {
            total_width,
            total_height,
            instructions,
        }
    }
    pub fn from_rendered_text(t: &RenderedText) -> Self {
        let instructions: Vec<_> = t
            .instructions
            .iter()
            .map(|i| match i {
                RenderedTextInstruction::RenderGlyph(g) => RenderedCharInstruction::RenderChar {
                    ch: g.ch,
                    advance_width: g.advance_width,
                    render_width: g.render_width,
                },
                RenderedTextInstruction::Kerning(dx) => RenderedCharInstruction::Kerning(*dx),
                RenderedTextInstruction::NextLine(dy, r) => {
                    RenderedCharInstruction::NextLine(*dy, *r)
                }
            })
            .collect();

        Self {
            total_width: t.total_width,
            total_height: t.total_height,
            instructions,
        }
    }

    pub fn get_total_width(&self) -> u32 {
        self.total_width
    }
    pub fn get_total_height(&self) -> u32 {
        self.total_height
    }
    pub fn get_instructions<'b>(&'b self) -> &'b Vec<RenderedCharInstruction> {
        &self.instructions
    }
}
