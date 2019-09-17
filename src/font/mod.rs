// Information regarding unicode text:
// https://manishearth.github.io/blog/2017/01/14/stop-ascribing-meaning-to-unicode-code-points/

pub(crate) mod align;
pub(crate) mod glyph;
pub(crate) mod layout;
pub(crate) mod pinned_cache;
pub(crate) mod pinned_data;
pub(crate) mod svg_font;
pub(crate) mod ttf_font;

use self::glyph::Glyph;
use self::pinned_cache::{CacheEditor, PinnedCache};
use self::svg_font::SvgFont;
use self::ttf_font::TtfFont;
use crate::errors::FontLoadError;
use crate::{Canvas, Rgba};
use std::collections::HashMap;

const FONT_CACHE_PAGE_SIZE: usize = 512;

#[derive(Clone)]
pub struct Font<'a> {
    editor: CacheEditor<'a, (u32, char), Glyph>,
    src: &'a FontSource,
}
impl<'a> Font<'a> {
    /// Note: line_height should be identical to font_size
    pub fn line_height(&self, font_size: u32) -> u32 {
        match &self.src {
            FontSource::Ttf(f) => f.line_height(font_size),
            FontSource::Svg(f) => f.line_height(font_size),
        }
    }

    /// Typically this will be `font_size * 1.25`
    pub fn line_advance_height(&self, font_size: u32) -> u32 {
        match &self.src {
            FontSource::Ttf(f) => f.line_advance_height(font_size),
            FontSource::Svg(f) => f.line_advance_height(font_size),
        }
    }

    /// Typically this will be `font_size * 0.75`
    pub fn dist_to_baseline(&self, font_size: u32) -> u32 {
        match &self.src {
            FontSource::Ttf(f) => f.dist_to_baseline(font_size),
            FontSource::Svg(f) => f.dist_to_baseline(font_size),
        }
    }

    pub fn advance_width(&mut self, font_size: u32, ch: char) -> i32 {
        self.glyph_for(font_size, ch).advance_width
    }

    pub fn glyph_for(&mut self, font_size: u32, ch: char) -> &'a Glyph {
        if let Some(g) = self.editor.get(&(font_size, ch)) {
            g
        } else {
            let g = match &self.src {
                FontSource::Ttf(f) => f.create_glyph(font_size, ch),
                FontSource::Svg(f) => f.create_glyph(font_size, ch),
            };
            self.editor.add((font_size, ch), g)
        }
    }

    pub fn glyphs_for_txt(&mut self, font_size: u32, txt: &str) -> Vec<&'a Glyph> {
        let mut result = Vec::with_capacity(txt.len());
        for ch in txt.chars() {
            result.push(self.glyph_for(font_size, ch));
        }
        result
    }

    pub fn kerning_for(&mut self, font_size: u32, first: char, second: char) -> i32 {
        match &self.src {
            FontSource::Ttf(f) => {
                let first = self.glyph_for(font_size, first);
                let second = self.glyph_for(font_size, second);
                f.kerning_for(font_size, first, second)
            }
            FontSource::Svg(f) => {
                let first = self.glyph_for(font_size, first);
                let second = self.glyph_for(font_size, second);
                f.kerning_for(font_size, first, second)
            }
        }
    }

    /// Performs simple rendering of the text. For a more advanced usage, refer to TextLayoutBuilder.
    pub fn render(
        &mut self,
        font_size: u32,
        color: Rgba,
        text: &str,
        pos: [i32; 2],
        width: Option<u32>,
        c: &mut Canvas,
    ) -> [u32; 2] {
        let font = unsafe {
            let lifetime_ptr = self as *mut Font<'a>;
            let static_ptr: *mut Font<'static> = std::mem::transmute(lifetime_ptr);
            &mut (*static_ptr)
        };
        let w = width.unwrap_or(std::u32::MAX / 4);
        let mut b = crate::font::layout::TextLayoutBuilder::new(Some(w));
        b.set_font_size(font, font_size);
        b.set_color(color);
        b.add_text(font, text);
        let layout = b.build();
        layout.render(pos, w, c);
        layout.required_dim()
    }
}

enum FontSource {
    Ttf(TtfFont),
    Svg(SvgFont),
}

pub struct FontCache {
    data: PinnedCache<(u32, char), Glyph>,
    src: FontSource,
}
impl FontCache {
    pub fn ttf_from_static(font_data: &'static [u8]) -> Result<Self, FontLoadError> {
        Ok(Self::ttf(TtfFont::from_static(font_data)?))
    }
    pub fn ttf_from_vec(font_data: Vec<u8>) -> Result<Self, FontLoadError> {
        Ok(Self::ttf(TtfFont::from_vec(font_data)?))
    }
    pub fn ttf_from_file(font_file: impl AsRef<std::path::Path>) -> Result<Self, FontLoadError> {
        use std::fs::File;
        use std::io::Read;

        let mut f = File::open(font_file)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        let result = Self::ttf_from_vec(buf)?;
        Ok(result)
    }
    pub fn ttf(ttf: TtfFont) -> Self {
        Self {
            data: PinnedCache::for_page_size(FONT_CACHE_PAGE_SIZE),
            src: FontSource::Ttf(ttf),
        }
    }
    pub fn svg_from_files(
        data: HashMap<char, (std::path::PathBuf, f32)>,
    ) -> Result<Self, FontLoadError> {
        use std::io::Read;

        let mut text_data = HashMap::with_capacity(data.len());
        for (k, (v, f)) in data.into_iter() {
            let mut file = std::fs::File::open(&v)?;
            let mut text = String::new();
            file.read_to_string(&mut text)?;
            text_data.insert(k, (text, f));
        }
        Self::svg_from_text(text_data)
    }
    pub fn svg_from_text(data: HashMap<char, (String, f32)>) -> Result<Self, FontLoadError> {
        let mut chars = HashMap::with_capacity(data.len());

        for (ch, (svg_text, rel_factor)) in data.into_iter() {
            let svg_img = nsvg::parse_str(&svg_text, nsvg::Units::Pixel, 96.0)?;

            let finding_scale = 1.0;
            let (_, height, _) = svg_img.rasterize_to_raw_rgba(finding_scale)?;
            if height == 0 {
                panic!("SVG rasterized to a zero-sized image");
            }
            let factor = rel_factor * finding_scale / (height as f32);

            chars.insert(ch, (svg_img, factor));
        }

        let font = SvgFont::new(chars);
        Ok(Self {
            data: PinnedCache::for_page_size(FONT_CACHE_PAGE_SIZE),
            src: FontSource::Svg(font),
        })
    }

    pub fn font<'a>(&'a self) -> Font<'a> {
        Font {
            editor: self.data.editor(),
            src: &self.src,
        }
    }
}

/// Font cache that exists with a 'static lifetime. This allows font results to be stored without
/// any lifetime concerns. However, this means that once a static cache is created, that memory
/// can never be freed.
#[derive(Clone, Copy)]
pub struct StaticFontCache(pub &'static FontCache);
impl StaticFontCache {
    pub fn new(cache: FontCache) -> Self {
        Self(Box::leak(Box::new(cache)))
    }

    pub fn font(&self) -> Font<'static> {
        self.0.font()
    }
}
impl From<FontCache> for StaticFontCache {
    fn from(cache: FontCache) -> Self {
        Self::new(cache)
    }
}
