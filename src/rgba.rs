use std;

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

fn f64_to_u8(v: f64) -> u8 {
    let u8v = v * 255.0;

    if u8v < 0.0 {
        0
    } else if u8v >= 255.0 {
        255
    } else {
        u8v as u32 as u8
    }
}

fn srgb_to_linear(v: f64) -> f64 {
    // From http://entropymine.com/imageworsener/srgbformula/
    // Converts sRGB to linear
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

impl Rgba {
    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Rgba {
        Rgba([f32_to_u8(r), f32_to_u8(g), f32_to_u8(b), f32_to_u8(a)])
    }
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Rgba {
        Rgba([r, g, b, a])
    }
    pub fn from_argb(argb: u32) -> Rgba {
        let b = (argb & 0xff) as u8;
        let g = ((argb >> 8) & 0xff) as u8;
        let r = ((argb >> 16) & 0xff) as u8;
        let a = ((argb >> 24) & 0xff) as u8;
        Self::from_u8(r, g, b, a)
    }
    pub fn from_rgba_native(argb: u32) -> Rgba {
        let c = unsafe { std::mem::transmute::<u32, [u8; 4]>(argb) };
        Rgba(c)
    }
    pub fn parse(s: &str) -> Result<Rgba, &'static str> {
        use std::u32;
        if s[..1] == *"#" {
            if let Ok(mut c) = u32::from_str_radix(&s[1..], 16) {
                if s.len() == 1 + 3 {
                    // Translate #aaa to #aaaaaa
                    let b = (c & 0xf) as u8;
                    let g = ((c >> 4) & 0xf) as u8;
                    let r = ((c >> 8) & 0xf) as u8;
                    return Ok(Rgba::from_u8(r | (r << 4), g | (g << 4), b | (b << 4), 0xff));
                }
                if s.len() == 1 + 6 {
                    // No alpha specified, assume it is solid
                    c |= 0xff000000;
                }
                Ok(Rgba::from_argb(c))
            } else {
                Err("Invalid RGBA hex code")
            }
        } else {
            Err("Unrecognized RGBA color type")
        }
    }

    pub fn srgb_to_linear(&self) -> Rgba {
        let c = self.0;

        let cf = (c[0] as f64 / 255.0, c[1] as f64 / 255.0, c[2] as f64 / 255.0);

        let cf = (srgb_to_linear(cf.0), srgb_to_linear(cf.1), srgb_to_linear(cf.2));

        Rgba([f64_to_u8(cf.0), f64_to_u8(cf.1), f64_to_u8(cf.2), c[3]])
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
        ((self.alpha() as u32) << 24) | ((self.red() as u32) << 16) | ((self.green() as u32) << 8) | (self.blue() as u32)
    }
    pub fn to_rgba_u32_native(&self) -> u32 {
        let c = self.0;
        unsafe { std::mem::transmute::<[u8; 4], u32>(c) }
    }

    pub fn rgba(&self) -> (u8, u8, u8, u8) {
        let c = self.0;
        (c[0], c[1], c[2], c[3])
    }
    pub fn rgb(&self) -> (u8, u8, u8) {
        let c = self.0;
        (c[0], c[1], c[2])
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
    pub fn rgba_f32(&self) -> (f32, f32, f32, f32) {
        let c = self.0;
        (u8_to_f32(c[0]), u8_to_f32(c[1]), u8_to_f32(c[2]), u8_to_f32(c[3]))
    }
    pub fn rgb_f32(&self) -> (f32, f32, f32) {
        let c = self.0;
        (u8_to_f32(c[0]), u8_to_f32(c[1]), u8_to_f32(c[2]))
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
