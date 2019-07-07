use crate::minmaxf32::*;
use crate::rgba::Rgba;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Hsl {
    hue: f32,        // 0 to 360
    saturation: f32, // 0 to 1 (0 = shade of gray)
    lightness: f32,  // 0 to 1 (0 = black, 1 = white)
    alpha: f32,      // 0 to 1
}
impl From<Rgba> for Hsl {
    fn from(v: Rgba) -> Self {
        Self::from_rgba(v)
    }
}
impl Hsl {
    pub fn new(hsla: [f32; 4]) -> Self {
        let [h, s, l, a] = hsla;
        Self {
            hue: valid_range(h, "hue", 0.0, 360.0),
            saturation: valid_range(s, "saturation", 0.0, 1.0),
            lightness: valid_range(l, "lightness", 0.0, 1.0),
            alpha: valid_range(a, "alpha", 0.0, 1.0),
        }
    }
    pub fn new_hsl(hsl: [f32; 3]) -> Self {
        Self::new([hsl[0], hsl[1], hsl[2], 1.0])
    }
    pub fn from_rgba(rgba: Rgba) -> Self {
        let hsl = rgb2hsl(rgba.rgb_f32());
        Self {
            hue: hsl[0],
            saturation: hsl[1],
            lightness: hsl[2],
            alpha: rgba.alpha_f32(),
        }
    }
    pub fn rgba(&self) -> Rgba {
        let hsl = [self.hue, self.saturation, self.lightness];
        let rgb = hsl2rgb(hsl);
        Rgba::from_f32([rgb[0], rgb[1], rgb[2], self.alpha])
    }
    pub fn hue(&self) -> f32 {
        self.hue
    }
    pub fn saturation(&self) -> f32 {
        self.saturation
    }
    pub fn lightness(&self) -> f32 {
        self.lightness
    }
    pub fn alpha(&self) -> f32 {
        self.alpha
    }
    pub fn hsla(&self) -> [f32; 4] {
        [self.hue, self.saturation, self.lightness, self.alpha]
    }
    pub fn hsl(&self) -> [f32; 3] {
        [self.hue, self.saturation, self.lightness]
    }
}

// http://axonflux.com/handy-rgb-to-hsl-and-rgb-to-hsv-color-model-c
pub(crate) fn rgb2hsl(rgb: [f32; 3]) -> [f32; 3] {
    let [r, g, b] = rgb;
    let min = minf3(r, g, b);
    let max = maxf3(r, g, b);
    let h: f32;
    let l = (max + min) / 2.0;

    if max == min {
        // Shade of gray
        [0.0, 0.0, l]
    } else {
        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };

        if r == max {
            h = (g - b) / d + (if g < b { 6.0 } else { 0.0 });
        } else if g == max {
            h = (b - r) / d + 2.0;
        } else {
            h = (r - g) / d + 4.0;
        }
        [h * 360.0 / 6.0, s, l]
    }
}
pub(crate) fn hsl2rgb(hsl: [f32; 3]) -> [f32; 3] {
    let [mut h, s, l] = hsl;
    h /= 360.0;

    if s <= 0.0 {
        [l, l, l]
    } else {
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        [
            hue2rgb(p, q, h + 1.0 / 3.0),
            hue2rgb(p, q, h),
            hue2rgb(p, q, h - 1.0 / 3.0),
        ]
    }
}
fn hue2rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < (1.0 / 6.0) {
        p + (q - p) * 6.0 * t
    } else if t < 0.5 {
        q
    } else if t < (2.0 / 3.0) {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}
