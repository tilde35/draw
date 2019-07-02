pub use crate::font::glyph::GlyphInstruction;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
enum BuilderState {
    Offset,
    Translucent(u8),
    Solid,
}

pub struct GlyphInstructionBuilder {
    cmds: Vec<GlyphInstruction>,
    state: BuilderState,
    state_len: u32,
}

impl GlyphInstructionBuilder {
    pub fn with_capacity(len: usize) -> GlyphInstructionBuilder {
        GlyphInstructionBuilder {
            cmds: Vec::with_capacity(len),
            state: BuilderState::Offset,
            state_len: 0,
        }
    }

    pub fn into_glyph_instructions(mut self) -> Vec<GlyphInstruction> {
        // Pop last value if it is NextRow or NextNRows
        if let Some(last_val) = self.cmds.pop() {
            let discard = match last_val {
                GlyphInstruction::NextRow | GlyphInstruction::NextNRows(..) => true,
                _ => false,
            };

            if !discard {
                self.cmds.push(last_val);
            }
        }
        self.cmds
    }

    pub fn next_val(&mut self, v: u8) {
        let next_state = if v == 0 {
            BuilderState::Offset
        } else if v == 255 {
            BuilderState::Solid
        } else {
            BuilderState::Translucent(v)
        };

        if next_state != self.state {
            self.emit_state();
            self.state = next_state;
            self.state_len = 1;
        } else {
            self.state_len += 1;
        }
    }

    pub fn next_row(&mut self) {
        if self.state != BuilderState::Offset {
            self.emit_state();
        }
        // TODO Check existing last value in cmds, if NextRow or NextRowN, then increment NextRowN instead
        self.cmds.push(GlyphInstruction::NextRow);

        self.state = BuilderState::Offset;
        self.state_len = 0;
    }

    fn emit_state(&mut self) {
        if self.state_len > 0 {
            match (self.state, self.state_len) {
                (BuilderState::Offset, len) => self.cmds.push(GlyphInstruction::Offset(len)),
                (BuilderState::Translucent(val), 1) => self.cmds.push(GlyphInstruction::Blend(val)),
                (BuilderState::Translucent(val), len) => self.cmds.push(GlyphInstruction::BlendN(val, len)),
                (BuilderState::Solid, 1) => self.cmds.push(GlyphInstruction::Draw),
                (BuilderState::Solid, len) => self.cmds.push(GlyphInstruction::DrawN(len)),
            }
        }
        self.state = BuilderState::Offset;
        self.state_len = 0;
    }
}
