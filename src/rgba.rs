use crate::errors::RgbaParseError;
use crate::hsl_color::Hsl;
use crate::hsv_color::Hsv;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Rgba(pub [u8; 4]);

fn u8_to_f32(v: u8) -> f32 {
    (v as u32 as f32) * (1.0 / 255.0)
}
fn f32_to_u8(v: f32) -> u8 {
    if v > 0.0 {
        if v < 1.0 {
            (v * 255.0 + 0.5) as u32 as u8
        } else {
            255
        }
    } else {
        // Note: Infinity, NaN should also end up here
        0
    }
}

fn srgb_to_linear(v: f32) -> f32 {
    // From http://entropymine.com/imageworsener/srgbformula/
    // Converts sRGB to linear
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

impl std::str::FromStr for Rgba {
    type Err = RgbaParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use std::u32;
        if s[..1] == *"#" {
            if let Ok(mut c) = u32::from_str_radix(&s[1..], 16) {
                if s.len() == 1 + 3 {
                    // Translate #aaa to #aaaaaa
                    let b = (c & 0xf) as u8;
                    let g = ((c >> 4) & 0xf) as u8;
                    let r = ((c >> 8) & 0xf) as u8;
                    return Ok(Rgba([r | (r << 4), g | (g << 4), b | (b << 4), 0xff]));
                }
                if s.len() == 1 + 6 {
                    // No alpha specified, assume it is solid
                    c |= 0xff000000;
                }
                Ok(Rgba::from_argb_u32(c))
            } else {
                Err(RgbaParseError::hex_parse(s))
            }
        } else {
            Err(RgbaParseError::unrecognized(s))
        }
    }
}
impl From<Hsv> for Rgba {
    fn from(v: Hsv) -> Self {
        v.rgba()
    }
}
impl From<Hsl> for Rgba {
    fn from(v: Hsl) -> Self {
        v.rgba()
    }
}

impl Rgba {
    pub fn from_f32(rgba: [f32; 4]) -> Rgba {
        let [r, g, b, a] = rgba;
        Rgba([f32_to_u8(r), f32_to_u8(g), f32_to_u8(b), f32_to_u8(a)])
    }
    pub fn from_argb_u32(argb: u32) -> Rgba {
        let argb = argb.to_le_bytes();
        Rgba([argb[2], argb[1], argb[0], argb[3]])
    }
    pub fn from_rgba_u32(rgba: u32) -> Rgba {
        Rgba(rgba.to_be_bytes())
    }
    pub fn from_le_u32(v: u32) -> Rgba {
        Rgba(v.to_le_bytes())
    }

    pub fn srgb_to_linear(&self) -> Rgba {
        let c = self.0;

        let cf = [
            srgb_to_linear((c[0] as f32) / 255.0),
            srgb_to_linear((c[1] as f32) / 255.0),
            srgb_to_linear((c[2] as f32) / 255.0),
        ];

        Rgba([f32_to_u8(cf[0]), f32_to_u8(cf[1]), f32_to_u8(cf[2]), c[3]])
    }

    pub fn srgb_to_linear_f32(&self) -> [f32; 4] {
        let c = self.0;

        [
            srgb_to_linear((c[0] as f32) / 255.0),
            srgb_to_linear((c[1] as f32) / 255.0),
            srgb_to_linear((c[2] as f32) / 255.0),
            (c[3] as f32) / 255.0,
        ]
    }

    pub fn with_alpha(&self, alpha: u8) -> Rgba {
        let c = self.0;
        Rgba([c[0], c[1], c[2], alpha])
    }

    pub fn relative_alpha(&self, alpha_factor: u8) -> Rgba {
        let c = self.0;
        let alpha = (alpha_factor as u32) * (c[3] as u32) / 255;
        Rgba([c[0], c[1], c[2], alpha as u8])
    }

    pub fn fast_relative_alpha(&self, alpha_factor: u8) -> Rgba {
        let c = self.0;
        // Aproximately: alpha_factor * alpha / 255
        let alpha = ((alpha_factor as u32) * (c[3] as u32)) >> 8;
        Rgba([c[0], c[1], c[2], alpha as u8])
    }

    pub fn to_argb_u32(&self) -> u32 {
        let [r, g, b, a] = self.0;
        u32::from_le_bytes([b, g, r, a])
    }
    pub fn to_rgba_u32(&self) -> u32 {
        let [r, g, b, a] = self.0;
        u32::from_le_bytes([a, b, g, r])
    }
    pub fn to_le_u32(&self) -> u32 {
        u32::from_le_bytes(self.0)
    }

    pub fn rgba(&self) -> [u8; 4] {
        self.0
    }
    pub fn rgb(&self) -> [u8; 3] {
        let c = self.0;
        [c[0], c[1], c[2]]
    }
    pub fn red(&self) -> u8 {
        self.0[0]
    }
    pub fn green(&self) -> u8 {
        self.0[1]
    }
    pub fn blue(&self) -> u8 {
        self.0[2]
    }
    pub fn alpha(&self) -> u8 {
        self.0[3]
    }
    pub fn set_red(&mut self, v: u8) {
        self.0[0] = v;
    }
    pub fn set_green(&mut self, v: u8) {
        self.0[1] = v;
    }
    pub fn set_blue(&mut self, v: u8) {
        self.0[2] = v;
    }
    pub fn set_alpha(&mut self, v: u8) {
        self.0[3] = v;
    }
    pub fn set_hue(&mut self, v: f32) {
        let mut hsl: Hsl = (*self).into();
        hsl.set_hue(v);
        *self = hsl.into();
    }
    pub fn set_saturation(&mut self, s: f32) {
        let mut hsl: Hsl = (*self).into();
        hsl.set_saturation(s);
        *self = hsl.into();
    }
    pub fn set_lightness(&mut self, l: f32) {
        let mut hsl: Hsl = (*self).into();
        hsl.set_lightness(l);
        *self = hsl.into();
    }
    pub fn offset_hue(&mut self, offset: f32) {
        let mut hsl: Hsl = (*self).into();
        hsl.offset_hue(offset);
        *self = hsl.into();
    }
    pub fn offset_saturation(&mut self, offset: f32) {
        let mut hsl: Hsl = (*self).into();
        hsl.offset_saturation(offset);
        *self = hsl.into();
    }
    pub fn offset_lightness(&mut self, offset: f32) {
        let mut hsl: Hsl = (*self).into();
        hsl.offset_lightness(offset);
        *self = hsl.into();
    }
    pub fn rgba_f32(&self) -> [f32; 4] {
        let c = self.0;
        [
            u8_to_f32(c[0]),
            u8_to_f32(c[1]),
            u8_to_f32(c[2]),
            u8_to_f32(c[3]),
        ]
    }
    pub fn rgb_f32(&self) -> [f32; 3] {
        let c = self.0;
        [u8_to_f32(c[0]), u8_to_f32(c[1]), u8_to_f32(c[2])]
    }
    pub fn red_f32(&self) -> f32 {
        u8_to_f32(self.0[0])
    }
    pub fn green_f32(&self) -> f32 {
        u8_to_f32(self.0[1])
    }
    pub fn blue_f32(&self) -> f32 {
        u8_to_f32(self.0[2])
    }
    pub fn alpha_f32(&self) -> f32 {
        u8_to_f32(self.0[3])
    }
}
