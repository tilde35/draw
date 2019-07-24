use crate::minmaxf32::*;
use crate::rgba::Rgba;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Hsv {
    hue: f32,        // 0 to 360
    saturation: f32, // 0 to 1 (0 = shade of gray)
    value: f32,      // 0 to 1 (0 = black, 1 = fully saturated color)
    alpha: f32,      // 0 to 1
}
impl From<Rgba> for Hsv {
    fn from(v: Rgba) -> Self {
        Self::from_rgba(v)
    }
}
impl Hsv {
    pub fn new(hsva: [f32; 4]) -> Self {
        let [h, s, v, a] = hsva;
        Self {
            hue: valid_range(h, "hue", 0.0, 360.0),
            saturation: valid_range(s, "saturation", 0.0, 1.0),
            value: valid_range(v, "value", 0.0, 1.0),
            alpha: valid_range(a, "alpha", 0.0, 1.0),
        }
    }
    pub fn new_hsv(hsv: [f32; 3]) -> Self {
        Self::new([hsv[0], hsv[1], hsv[2], 1.0])
    }
    pub fn from_rgba(rgba: Rgba) -> Self {
        let hsv = rgb2hsv(rgba.rgb_f32());
        Self {
            hue: hsv[0],
            saturation: hsv[1],
            value: hsv[2],
            alpha: rgba.alpha_f32(),
        }
    }
    pub fn rgba(&self) -> Rgba {
        let hsv = [self.hue, self.saturation, self.value];
        let rgb = hsv2rgb(hsv);
        Rgba::from_f32([rgb[0], rgb[1], rgb[2], self.alpha])
    }
    pub fn hue(&self) -> f32 {
        self.hue
    }
    pub fn saturation(&self) -> f32 {
        self.saturation
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    pub fn alpha(&self) -> f32 {
        self.alpha
    }
    pub fn set_hue(&mut self, hue: f32) {
        self.hue = valid_range(hue, "hue", 0.0, 360.0);
    }
    pub fn set_saturation(&mut self, s: f32) {
        self.saturation = valid_range(s, "saturation", 0.0, 1.0);
    }
    pub fn set_value(&mut self, v: f32) {
        self.value = valid_range(v, "value", 0.0, 1.0);
    }
    pub fn set_alpha(&mut self, a: f32) {
        self.alpha = valid_range(a, "alpha", 0.0, 1.0);
    }
    pub fn offset_saturation(&mut self, offset: f32) {
        self.saturation = offset_range(self.saturation, offset, "saturation", 0.0, 1.0);
    }
    pub fn offset_value(&mut self, offset: f32) {
        self.value = offset_range(self.value, offset, "value", 0.0, 1.0);
    }
    pub fn offset_hue(&mut self, offset: f32) {
        assert!(offset.is_finite(), "Hue offset must be a valid number");
        let v = (self.hue + offset) % 360.0;
        if v < 0.0 {
            self.hue = 360.0 + v;
        } else {
            self.hue = v;
        }
    }
    pub fn hsva(&self) -> [f32; 4] {
        [self.hue, self.saturation, self.value, self.alpha]
    }
    pub fn hsv(&self) -> [f32; 3] {
        [self.hue, self.saturation, self.value]
    }
}

pub(crate) fn rgb2hsv(rgb: [f32; 3]) -> [f32; 3] {
    let [in_r, in_g, in_b] = rgb;

    let min = minf3(in_r, in_g, in_b);
    let max = maxf3(in_r, in_g, in_b);

    let mut out_h: f32;
    let out_s: f32;
    let out_v = max;

    let delta = max - min;
    if delta < 0.00001 {
        out_s = 0.0f32;
        out_h = 0.0f32;
        return [out_h, out_s, out_v];
    }
    if max > 0.0 {
        out_s = delta / max;
    } else {
        out_s = 0.0;
        out_h = 0.0;
        return [out_h, out_s, out_v];
    }

    if in_r >= max {
        out_h = (in_g - in_b) / delta;
    } else if in_g >= max {
        out_h = 2.0 + (in_b - in_r) / delta;
    } else {
        out_h = 4.0 + (in_r - in_g) / delta;
    }

    out_h *= 60.0;

    if out_h < 0.0 {
        out_h += 360.0;
    }

    [out_h, out_s, out_v]
}

pub(crate) fn hsv2rgb(hsv: [f32; 3]) -> [f32; 3] {
    let [in_h, in_s, in_v] = hsv;

    if in_s <= 0.0 {
        return [in_v, in_v, in_v];
    }

    let mut hh = in_h;
    if hh >= 360.0 {
        hh = 0.0;
    }
    hh /= 60.0;
    let i = hh as i32;
    let ff = hh - (i as f32);
    let p = in_v * (1.0 - in_s);
    let q = in_v * (1.0 - (in_s * ff));
    let t = in_v * (1.0 - (in_s * (1.0 - ff)));

    match i {
        0 => [in_v, t, p],
        1 => [q, in_v, p],
        2 => [p, in_v, t],
        3 => [p, q, in_v],
        4 => [t, p, in_v],
        _ => [in_v, p, q],
    }
}
