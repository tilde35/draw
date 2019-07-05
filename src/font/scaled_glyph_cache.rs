use crate::font::glyph::Glyph;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ScaledFontCache {
    glyph_cache: HashMap<char, Glyph>,
}
impl ScaledFontCache {
    pub fn new(_size: f32) -> ScaledFontCache {
        ScaledFontCache {
            glyph_cache: HashMap::new(),
        }
    }
    pub fn create_if_missing<F>(&mut self, ch: char, init: F)
    where
        F: FnOnce() -> Glyph,
    {
        self.glyph_cache.entry(ch).or_insert_with(init);
    }
    pub fn get(&self, ch: char) -> Option<&Glyph> {
        self.glyph_cache.get(&ch)
    }
}
