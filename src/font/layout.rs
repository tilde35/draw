use crate::font::align::{HorzAlign, ScriptPosition};
use crate::font::*;
use crate::{Canvas, Rgba};
use std::cmp::max;

pub struct TextLayout<'fontcache> {
    min_dim: [u32; 2],
    layout_width: u32,
    cmds: Vec<TextLayoutCmd>,
    glyphs: Vec<(&'fontcache Glyph, [i32; 2], Rgba)>,
}
impl<'fontcache> TextLayout<'fontcache> {
    pub fn required_dim(&self) -> [u32; 2] {
        self.min_dim
    }
    pub fn render(&self, pos: [i32; 2], width: u32, c: &mut Canvas) {
        render(c, pos, width, self);
    }
}

#[derive(Clone, Debug)]
pub struct LinkBoundingBox {
    x_offset: i32,
    layout_width: u32,
    horz_align: HorzAlign,
    pos: [i32; 2],
    dim: [u32; 2],
}
impl LinkBoundingBox {
    pub fn rect_for_width(&self, final_width: u32) -> ([i32; 2], [u32; 2]) {
        let center_extra = (final_width as i32) - (self.layout_width as i32);

        let pos = [self.x_offset + self.pos[0], self.pos[1]];

        match self.horz_align {
            HorzAlign::Left => (pos, self.dim),
            HorzAlign::Center => ([(center_extra / 2) + pos[0], pos[1]], self.dim),
            HorzAlign::Right => ([center_extra + pos[0], pos[1]], self.dim),
        }
    }
    fn extend_to(&mut self, pos: [i32; 2], dim: [u32; 2]) {
        let end_x = pos[0] + (dim[0] as i32);
        let end_y = pos[1] + (dim[1] as i32);
        if pos[1] < self.pos[1] {
            let dy = self.pos[1] - pos[1];
            self.pos[1] = pos[1];
            self.dim[1] += dy as u32;
        }
        let cur_end_x = self.pos[0] + (self.dim[0] as i32);
        if end_x > cur_end_x {
            let dx = end_x - cur_end_x;
            self.dim[0] += dx as u32;
        }
        let cur_end_y = self.pos[1] + (self.dim[1] as i32);
        if end_y > cur_end_y {
            let dy = end_y - cur_end_y;
            self.dim[1] += dy as u32;
        }
    }
}

#[derive(Clone)]
pub struct LinkLayout<'fontcache> {
    link_to: String,
    layout_width: u32,
    cmds: Vec<TextLayoutCmd>,
    glyphs: Vec<(&'fontcache Glyph, [i32; 2], [u32; 2], Rgba)>,
    bounding_boxes: Vec<LinkBoundingBox>,
}
impl<'fontcache> LinkLayout<'fontcache> {
    pub fn link_to(&self) -> &str {
        &self.link_to
    }
    pub fn bounding_boxes(&self) -> &[LinkBoundingBox] {
        &self.bounding_boxes
    }
    pub fn render(&self, pos: [i32; 2], final_width: u32, link_color: Rgba, c: &mut Canvas) {
        let center_extra = (final_width as i32) - (self.layout_width as i32);
        // let center_extra = max(center_extra, 0);

        let mut line_offset = pos[0];

        for cmd in self.cmds.iter() {
            match cmd {
                TextLayoutCmd::Glyphs { glyph_range } => {
                    let glyphs = &self.glyphs[(glyph_range[0] as usize)..(glyph_range[1] as usize)];
                    for (g, rel_pos, ..) in glyphs.iter() {
                        let draw_pos = [line_offset + rel_pos[0], pos[1] + rel_pos[1]];
                        g.draw(c, draw_pos, link_color);
                    }
                }
                TextLayoutCmd::LineSettings {
                    x_offset,
                    horz_align,
                } => {
                    line_offset = pos[0]
                        + match horz_align {
                            HorzAlign::Left => *x_offset,
                            HorzAlign::Center => (center_extra / 2) + (*x_offset),
                            HorzAlign::Right => center_extra + (*x_offset),
                        }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum TextLayoutCmd {
    // TextSettings {
    //     color: Rgba,
    // },
    Glyphs {
        glyph_range: [u32; 2],
    },
    LineSettings {
        x_offset: i32,
        horz_align: HorzAlign,
    },
}

pub struct TextLayoutBuilder<'fontcache> {
    font_size: u32,
    b: LayoutCmdBuilder<'fontcache>,
}
impl<'fontcache> TextLayoutBuilder<'fontcache> {
    pub fn new(expected_max_width: Option<u32>) -> Self {
        let mut b = LayoutCmdBuilder::new(expected_max_width);
        b.init_font_info();

        Self { font_size: 16, b }
    }

    pub fn add_text(&mut self, font: &mut Font<'fontcache>, txt: &str) {
        self.b.set_font_info(font, self.font_size);
        for ch in txt.chars() {
            self.add_next_char(font, ch);
        }
    }
    pub fn add_char(&mut self, font: &mut Font<'fontcache>, ch: char) {
        self.b.set_font_info(font, self.font_size);
        self.add_next_char(font, ch);
    }
    fn add_next_char(&mut self, font: &mut Font<'fontcache>, ch: char) {
        let (script_size, script_offset) = match self.b.script_pos {
            ScriptPosition::Normal => (self.font_size, 0),
            _ => {
                let size = self.font_size * 5 / 8;
                let offset = if self.b.script_pos == ScriptPosition::Superscript {
                    0
                } else {
                    (self.font_size as i32) - (size as i32)
                };
                (size, offset)
            }
        };
        match ch {
            ' ' => {
                // Standard space
                let g = font.glyph_for(script_size, ' ');
                self.b.append_space(g);
            }
            '\t' => {
                // Always translate tab to 4 spaces
                let g = font.glyph_for(script_size, ' ');
                for _ in 0..4 {
                    self.b.append_space(g);
                }
            }
            '\r' => {
                // Ignore line feed character
            }
            '\n' => {
                // New line
                self.b.next_line();
            }
            _ => {
                // Other character
                let g = font.glyph_for(script_size, ch);
                // Note: Use normal font_size for metrics
                self.b.append(font, self.font_size, g, script_offset);
            }
        }
    }
    pub fn add_word_break(&mut self) {
        self.b.word_end();
    }
    pub fn set_font_size(&mut self, font: &crate::font::Font<'static>, font_size: u32) {
        self.font_size = font_size;
        self.b.set_font_info(font, font_size);
    }
    pub fn set_color(&mut self, color: Rgba) {
        self.b.set_color(color);
    }
    pub fn set_horz_align(&mut self, horz_align: HorzAlign) {
        self.b.set_horz_align(horz_align);
    }
    pub fn set_script_pos(&mut self, script: ScriptPosition) {
        self.b.set_script_pos(script);
    }
    pub fn link_start(&mut self, link_to: String) {
        // Start tracking link
        self.b.link_active = true;
        self.b.cur_link.start_idx = self.b.glyphs.len();
        self.b.cur_link.link_to = link_to;
        self.b.cur_link.char_dims.clear();
    }
    pub fn link_end(&mut self) {
        // Complete link tracking
        self.b.link_active = false;
        let mut tmp = LinkInfo::new();
        std::mem::swap(&mut tmp, &mut self.b.cur_link);
        self.b.links.push(tmp);
    }

    pub fn build(self) -> TextLayout<'fontcache> {
        let mut b = self.b;
        b.finish();

        let layout_width = b.max_width.unwrap_or(b.required_width);
        TextLayout {
            min_dim: [b.required_width, b.required_height],
            layout_width,
            cmds: b.cmds,
            glyphs: b.glyphs,
        }
    }

    pub fn build_with_link_extract(
        self,
        font: &mut crate::font::Font<'fontcache>,
    ) -> (TextLayout<'fontcache>, Vec<LinkLayout<'fontcache>>) {
        let mut b = self.b;
        b.finish();

        let layout_width = b.max_width.unwrap_or(b.required_width);

        let mut link_layouts = Vec::new();

        // Create link glyphs
        for link in b.links.iter() {
            let mut link_glyphs = Vec::with_capacity(link.char_dims.len());

            let end_idx = link.start_idx + link.char_dims.len();

            let start_u32 = link.start_idx as u32;
            let end_u32 = end_idx as u32;

            let mut cmds = Vec::new();
            cmds.push(b.line_settings_for_glyph(link.start_idx));
            let mut found_start = false;
            for cmd in b.cmds.iter() {
                match cmd {
                    TextLayoutCmd::Glyphs { glyph_range } => {
                        if !found_start && start_u32 >= glyph_range[0] && start_u32 < glyph_range[1]
                        {
                            // Found the initial command
                            found_start = true;

                            if end_u32 <= glyph_range[1] {
                                // Start is also the end
                                cmds.push(TextLayoutCmd::Glyphs {
                                    glyph_range: [start_u32, end_u32],
                                });
                                break;
                            } else {
                                cmds.push(TextLayoutCmd::Glyphs {
                                    glyph_range: [start_u32, glyph_range[1]],
                                });
                            }
                        } else if found_start {
                            if end_u32 <= glyph_range[1] {
                                // No more commands associated with this link
                                cmds.push(TextLayoutCmd::Glyphs {
                                    glyph_range: [glyph_range[0], end_u32],
                                });
                                break;
                            } else {
                                // Add command as-is
                                cmds.push(cmd.clone());
                            }
                        }
                    }
                    TextLayoutCmd::LineSettings { .. } => {
                        if found_start {
                            cmds.push(cmd.clone());
                        }
                    }
                }
            }
            // Fix indexing on the commands
            for cmd in cmds.iter_mut() {
                match cmd {
                    TextLayoutCmd::Glyphs { glyph_range } => {
                        glyph_range[0] -= start_u32;
                        glyph_range[1] -= start_u32;
                    }
                    TextLayoutCmd::LineSettings { .. } => {}
                }
            }

            // Copy glyphs
            for offset in 0..link.char_dims.len() {
                let idx = link.start_idx + offset;
                let (glyph, pos, color) = b.glyphs[idx];
                link_glyphs.push((glyph, pos, link.char_dims[offset], color));
            }

            // Create bounding boxes (requires line settings as well)
            let mut bounding_boxes = Vec::new();
            let mut cur_box = LinkBoundingBox {
                x_offset: 0,
                layout_width,
                horz_align: HorzAlign::Left,
                pos: [0, 0],
                dim: [0, 0],
            };
            for cmd in cmds.iter() {
                match cmd {
                    TextLayoutCmd::Glyphs { glyph_range } => {
                        for (_glyph, pos, dim, _color) in
                            &link_glyphs[(glyph_range[0] as usize)..(glyph_range[1] as usize)]
                        {
                            if cur_box.dim[0] == 0 {
                                // No contents, create a new box
                                cur_box.pos = *pos;
                                cur_box.dim = *dim;
                            } else {
                                // Extend current box
                                cur_box.extend_to(*pos, *dim);
                            }
                        }
                    }
                    TextLayoutCmd::LineSettings {
                        x_offset,
                        horz_align,
                    } => {
                        if cur_box.dim[0] > 0 {
                            bounding_boxes.push(cur_box.clone());
                        }
                        cur_box.x_offset = *x_offset;
                        cur_box.horz_align = *horz_align;
                        cur_box.dim = [0, 0];
                    }
                }
            }
            if cur_box.dim[0] > 0 {
                bounding_boxes.push(cur_box);
            }

            // link_to: String,
            // layout_width: u32,
            // cmds: Vec<TextLayoutCmd>,
            // glyphs: Vec<(&'fontcache Glyph, [i32; 2], Rgba)>,
            // bounding_boxes: Vec<([i32; 2], [u32; 2])>,

            link_layouts.push(LinkLayout {
                link_to: link.link_to.clone(),
                layout_width,
                cmds,
                glyphs: link_glyphs,
                bounding_boxes,
            });
        }

        // Blank out all link characters
        let blank = font.glyph_for(16, ' ');
        for link in b.links.iter() {
            for offset in 0..link.char_dims.len() {
                let idx = link.start_idx + offset;
                b.glyphs[idx].0 = blank;
            }
        }

        // Option 1: Replace glyphs with spaces
        // Option 2: Remove from `cmds` list
        // Note: Links can span multiple lines

        let layout = TextLayout {
            min_dim: [b.required_width, b.required_height],
            layout_width,
            cmds: b.cmds,
            glyphs: b.glyphs,
        };
        (layout, link_layouts)
    }
}

fn render<'fontcache>(
    c: &mut Canvas,
    pos: [i32; 2],
    final_width: u32,
    layout: &TextLayout<'fontcache>,
) {
    let center_extra = (final_width as i32) - (layout.layout_width as i32);
    // let center_extra = max(center_extra, 0);

    let mut line_offset = pos[0];

    for cmd in layout.cmds.iter() {
        match cmd {
            TextLayoutCmd::Glyphs { glyph_range } => {
                let glyphs = &layout.glyphs[(glyph_range[0] as usize)..(glyph_range[1] as usize)];
                for (g, rel_pos, actual_color) in glyphs.iter() {
                    let draw_pos = [line_offset + rel_pos[0], pos[1] + rel_pos[1]];
                    g.draw(c, draw_pos, *actual_color);
                }
            }
            TextLayoutCmd::LineSettings {
                x_offset,
                horz_align,
            } => {
                line_offset = pos[0]
                    + match horz_align {
                        HorzAlign::Left => *x_offset,
                        HorzAlign::Center => (center_extra / 2) + (*x_offset),
                        HorzAlign::Right => center_extra + (*x_offset),
                    }
            }
        }
    }
}

struct WordInfo {
    prev_char: char,
    draw_start_x: i32,
    draw_width: i32,
    glyph_start_idx: usize,
    // The x-offset from the *start of the word* to use for the next glyph
    next_glyph_x: i32,

    horz_align: HorzAlign,
    font_dirty: bool,

    // Font metrics
    line_height: u32,
    line_advance_height: u32,
    dist_to_baseline: i32,
}
impl WordInfo {
    pub fn init_line_metrics(
        &mut self,
        line_height: u32,
        line_advance_height: u32,
        dist_to_baseline: i32,
    ) {
        self.line_height = line_height;
        self.line_advance_height = line_advance_height;
        self.dist_to_baseline = dist_to_baseline;
    }
    pub fn add_line_metrics(
        &mut self,
        line_height: u32,
        line_advance_height: u32,
        dist_to_baseline: i32,
    ) {
        self.line_height = max(self.line_height, line_height);
        self.line_advance_height = max(self.line_advance_height, line_advance_height);
        self.dist_to_baseline = max(self.dist_to_baseline, dist_to_baseline);
    }
}
struct LineInfo {
    glyph_start_idx: usize,
    glyph_end_idx: usize,
    line_settings_cmd_idx: usize,
    draw_width: i32,

    // Font metrics
    line_height: u32,
    line_advance_height: u32,
    dist_to_baseline: i32,
}
impl LineInfo {
    pub fn init_line_metrics(
        &mut self,
        line_height: u32,
        line_advance_height: u32,
        dist_to_baseline: i32,
    ) {
        self.line_height = line_height;
        self.line_advance_height = line_advance_height;
        self.dist_to_baseline = dist_to_baseline;
    }
    pub fn add_line_metrics(
        &mut self,
        line_height: u32,
        line_advance_height: u32,
        dist_to_baseline: i32,
    ) {
        self.line_height = max(self.line_height, line_height);
        self.line_advance_height = max(self.line_advance_height, line_advance_height);
        self.dist_to_baseline = max(self.dist_to_baseline, dist_to_baseline);
    }
    pub fn add_word_metrics(&mut self, w: &WordInfo) {
        self.add_line_metrics(w.line_height, w.line_advance_height, w.dist_to_baseline);
    }
}
struct LineWrapMetrics {
    // Font metrics
    line_height: u32,
    line_advance_height: u32,
    dist_to_baseline: i32,
}

struct LinkInfo {
    start_idx: usize,
    link_to: String,
    char_dims: Vec<[u32; 2]>,
}
impl LinkInfo {
    pub fn new() -> Self {
        Self {
            start_idx: 0,
            link_to: String::new(),
            char_dims: Vec::new(),
        }
    }
}

struct LayoutCmdBuilder<'fontcache> {
    draw_pos: [i32; 2],
    required_width: u32,
    required_height: u32,
    max_width: Option<u32>,
    cmds: Vec<TextLayoutCmd>,
    glyphs: Vec<(&'fontcache Glyph, [i32; 2], Rgba)>,
    deferred_line_alignment: Vec<(usize, i32)>,
    // Color
    color: Rgba,
    // color_dirty: bool,
    // Alignment
    pending_align: HorzAlign,
    // Script position
    script_pos: ScriptPosition,
    // Current word information
    active_word: bool,
    cur_word: WordInfo,
    // Current line information
    cur_line: LineInfo,
    empty_line_metrics: LineWrapMetrics,
    // Links
    link_active: bool,
    cur_link: LinkInfo,
    links: Vec<LinkInfo>,
}
impl<'fontcache> LayoutCmdBuilder<'fontcache> {
    pub fn new(max_width: Option<u32>) -> Self {
        let settings_cmd = TextLayoutCmd::LineSettings {
            x_offset: 0,
            horz_align: HorzAlign::Left,
        };
        Self {
            draw_pos: [0, 0],
            required_width: 0,
            required_height: 0,
            max_width,
            cmds: vec![settings_cmd],
            glyphs: Vec::new(),
            deferred_line_alignment: Vec::new(),
            color: Rgba([0, 0, 0, 255]),
            // color_dirty: false,
            pending_align: HorzAlign::Left,
            script_pos: ScriptPosition::Normal,
            active_word: false,
            cur_word: WordInfo {
                prev_char: ' ',
                draw_start_x: 0,
                draw_width: 0,
                glyph_start_idx: 0,
                next_glyph_x: 0,
                horz_align: HorzAlign::Left,
                font_dirty: false,

                line_height: 0,
                line_advance_height: 0,
                dist_to_baseline: 0,
            },
            cur_line: LineInfo {
                glyph_start_idx: 0,
                glyph_end_idx: 0,
                line_settings_cmd_idx: 0,
                draw_width: 0,

                line_height: 0,
                line_advance_height: 0,
                dist_to_baseline: 0,
            },
            empty_line_metrics: LineWrapMetrics {
                line_height: 0,
                line_advance_height: 0,
                dist_to_baseline: 0,
            },
            link_active: false,
            cur_link: LinkInfo::new(),
            links: Vec::new(),
        }
    }

    pub fn init_font_info(&mut self) {
        self.cur_word.font_dirty = true;

        self.empty_line_metrics.line_height = 0;
        self.empty_line_metrics.line_advance_height = 0;
        self.empty_line_metrics.dist_to_baseline = 0;
    }

    pub fn set_font_info(&mut self, font: &Font<'fontcache>, font_size: u32) {
        self.cur_word.font_dirty = true;

        // Note: Always keep track of current metrics so that empty lines can be displayed
        let line_height = font.line_height(font_size);
        let line_advance_height = font.line_advance_height(font_size);
        let dist_to_baseline = font.dist_to_baseline(font_size) as i32;

        self.empty_line_metrics.line_height = line_height;
        self.empty_line_metrics.line_advance_height = line_advance_height;
        self.empty_line_metrics.dist_to_baseline = dist_to_baseline;
    }

    pub fn set_color(&mut self, c: Rgba) {
        self.color = c;
        // self.color_dirty = true;
    }

    pub fn set_horz_align(&mut self, horz_align: HorzAlign) {
        self.pending_align = horz_align;
    }

    pub fn set_script_pos(&mut self, script: ScriptPosition) {
        self.script_pos = script;
    }

    pub fn append(
        &mut self,
        font: &mut Font<'fontcache>,
        font_size: u32,
        g: &'fontcache Glyph,
        script_offset: i32,
    ) {
        // Flush any pending commands
        // if self.color_dirty {
        //     let color = self.color;
        //     self.cmds.push(TextLayoutCmd::TextSettings { color });
        //     self.color_dirty = false;
        // }

        let line_height = font.line_height(font_size);
        let line_advance_height = font.line_advance_height(font_size);
        let dist_to_baseline = font.dist_to_baseline(font_size) as i32;

        if !self.active_word {
            // =========================================
            // ========== Start of a new word ==========
            // =========================================

            self.active_word = true;

            // Initialize all current word fields
            self.cur_word.prev_char = ' ';
            self.cur_word.draw_start_x = self.draw_pos[0];
            self.cur_word.draw_width = 0;

            self.cur_word.glyph_start_idx = self.glyphs.len();
            self.cur_word.next_glyph_x = 0;

            // Alignment
            self.cur_word.horz_align = self.pending_align;

            // Font metrics
            self.cur_word
                .init_line_metrics(line_height, line_advance_height, dist_to_baseline);
            self.cur_word.font_dirty = false;
        } else {
            // ============================================
            // ========== Continue existing word ==========
            // ============================================

            let kerning = font.kerning_for(font_size, self.cur_word.prev_char, g.ch);
            self.cur_word.next_glyph_x += kerning;

            // Font metrics
            if self.cur_word.font_dirty {
                self.cur_word
                    .add_line_metrics(line_height, line_advance_height, dist_to_baseline);
                self.cur_word.font_dirty = false;
            }
        }
        self.cur_word.prev_char = g.ch;
        self.cur_word.draw_width += g.advance_width;

        // All words should start with the y-offset set so that baseline is at offset zero
        // This will be adjusted when the line is finalized
        let y_offset = -(font.dist_to_baseline(font_size) as i32);

        let glyph_pos = [self.cur_word.next_glyph_x, y_offset + script_offset];
        self.glyphs.push((g, glyph_pos, self.color));
        self.cur_word.next_glyph_x += g.advance_width;

        if self.link_active {
            // Add glyph information to link
            self.cur_link.char_dims.push([g.render_width, line_height]);
        }
    }

    pub fn append_space(&mut self, space: &'fontcache Glyph) {
        // Determines where the next word will start
        self.word_end();
        self.draw_pos[0] += space.advance_width;
    }

    pub fn next_line(&mut self) {
        self.word_end();
        self.start_new_line();
    }

    pub fn word_end(&mut self) {
        if self.active_word {
            self.active_word = false;
            let mut word_start = self.cur_word.draw_start_x;

            // Handle word wrapping
            {
                let cur_line_width = word_start + self.cur_word.draw_width;
                let max_width = self.max_width.map(|v| v as i32).unwrap_or(std::i32::MAX);

                // Word split policy: If word cannot fit on a single line, then it will be truncated (rather than splitting and wrapping)
                // If this will be changed in the future, then `word_start==0` should be handled by splitting the current word
                if cur_line_width > max_width && word_start != 0 {
                    // Move word to next line
                    self.start_new_line();
                    word_start = 0;
                }
            }

            // Update glyph locations based on word start location
            let start_idx = self.cur_word.glyph_start_idx;
            for e in &mut self.glyphs[start_idx..] {
                e.1[0] += word_start;
            }

            // Update draw position for next word
            self.draw_pos[0] = word_start + self.cur_word.draw_width;

            // Update current draw width
            self.cur_line.draw_width = self.draw_pos[0];

            // Update glyph index range
            self.cur_line.glyph_end_idx = self.glyphs.len();

            // Update current line metrics
            self.cur_line.add_word_metrics(&self.cur_word);

            // Update horizontal alignment
            self.set_cur_line_horz_align(self.cur_word.horz_align);
        }
    }

    /// This method starts a new line without regards to the current word
    fn start_new_line(&mut self) {
        // Finalize current line
        self.finalize_line();

        let line_settings_cmd_idx = self.cmds.len();
        self.cmds.push(TextLayoutCmd::LineSettings {
            x_offset: 0,
            horz_align: HorzAlign::Left,
        });

        // The first glyph index will be after the end of the previous line
        let glyph_start_idx = self.cur_line.glyph_end_idx;

        self.cur_line.glyph_start_idx = glyph_start_idx;
        self.cur_line.glyph_end_idx = self.cur_line.glyph_start_idx;

        self.cur_line.line_settings_cmd_idx = line_settings_cmd_idx;

        self.cur_line.draw_width = 0;

        // New line: initialize all line sizes to zero
        // Note: If the next line ends up being empty, then the empty_line_metrics will be used
        self.cur_line.init_line_metrics(0, 0, 0);
    }

    /// This method finalizes the current line, but does NOT start the next line.
    fn finalize_line(&mut self) {
        let start_idx = self.cur_line.glyph_start_idx;
        let end_idx = self.cur_line.glyph_end_idx;

        let is_empty = start_idx == end_idx;

        if is_empty {
            // Line is empty, the current metrics will all be zero - change this
            let line_height = self.empty_line_metrics.line_height;
            let line_advance_height = self.empty_line_metrics.line_advance_height;
            let dist_to_baseline = self.empty_line_metrics.dist_to_baseline;
            self.cur_line
                .init_line_metrics(line_height, line_advance_height, dist_to_baseline);
        } else {
            // Create glyph render command
            let glyph_range = [start_idx as u32, end_idx as u32];
            self.cmds.push(TextLayoutCmd::Glyphs { glyph_range });
        }

        // Update y position using baseline height and draw position
        let y_draw_offset = self.draw_pos[1] + self.cur_line.dist_to_baseline;
        for e in &mut self.glyphs[start_idx..end_idx] {
            e.1[1] += y_draw_offset;
        }

        // Update required dimensions
        self.required_width = max(self.required_width, self.cur_line.draw_width as u32);
        self.required_height = (self.draw_pos[1] as u32) + self.cur_line.line_height;

        // Adjust draw position for the next line
        self.draw_pos[0] = 0;
        self.draw_pos[1] += self.cur_line.line_advance_height as i32;

        // Adjust line's x_offset based on horizontal alignment
        if let Some(max_width) = self.max_width {
            let max_width = max_width as i32;
            let line_width = self.cur_line.draw_width;

            if let TextLayoutCmd::LineSettings {
                ref mut x_offset,
                horz_align,
            } = &mut self.cmds[self.cur_line.line_settings_cmd_idx]
            {
                match horz_align {
                    HorzAlign::Left => {
                        *x_offset = 0;
                    }
                    HorzAlign::Center => {
                        *x_offset = (max_width - line_width) / 2;
                    }
                    HorzAlign::Right => {
                        *x_offset = max_width - line_width;
                    }
                }
            }
        } else {
            // Cannot evaluate offset until all lines have been rendered
            self.deferred_line_alignment.push((
                self.cur_line.line_settings_cmd_idx,
                self.cur_line.draw_width,
            ));
        }
    }

    pub fn finish(&mut self) {
        self.word_end();
        self.finalize_line();

        // If the max width was not originally specified, then handle the deferred alignments
        if self.max_width.is_none() {
            let max_width = self.required_width as i32;

            for (settings_idx, line_width) in self.deferred_line_alignment.iter().cloned() {
                if let TextLayoutCmd::LineSettings {
                    ref mut x_offset,
                    horz_align,
                } = &mut self.cmds[settings_idx]
                {
                    match horz_align {
                        HorzAlign::Left => {
                            *x_offset = 0;
                        }
                        HorzAlign::Center => {
                            *x_offset = (max_width - line_width) / 2;
                        }
                        HorzAlign::Right => {
                            *x_offset = max_width - line_width;
                        }
                    }
                }
            }
        }
    }

    fn line_settings_for_glyph(&self, glyph_idx: usize) -> TextLayoutCmd {
        let glyph_u32 = glyph_idx as u32;
        let mut cur_line_settings = TextLayoutCmd::LineSettings {
            x_offset: 0,
            horz_align: HorzAlign::Left,
        };
        for cmd in self.cmds.iter() {
            match cmd {
                TextLayoutCmd::Glyphs { glyph_range } => {
                    if glyph_u32 >= glyph_range[0] && glyph_u32 < glyph_range[1] {
                        return cur_line_settings;
                    }
                }
                TextLayoutCmd::LineSettings { .. } => {
                    cur_line_settings = cmd.clone();
                }
            }
        }
        cur_line_settings
    }

    fn set_cur_line_horz_align(&mut self, align: HorzAlign) {
        let idx = self.cur_line.line_settings_cmd_idx;
        if let TextLayoutCmd::LineSettings {
            ref mut horz_align, ..
        } = &mut self.cmds[idx]
        {
            *horz_align = align;
        }
    }
}
