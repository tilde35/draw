use crate::rgba::Rgba;

// ===================== Blending Mode Traits =====================

// Default implementations: ColorBlendOverwrite, ColorBlendOpaque, ColorBlendTransparent
pub trait ColorBlendMode: Copy {
    type ColorContext;

    fn prepare_color(&self, color: Rgba) -> Self::ColorContext;
    fn blend_color(&self, bg: &mut Rgba, color_context: &Self::ColorContext);
}

// Default implementations: SolidColorAlphaBlendOverwrite, ColorAlphaBlendOverwrite, ColorAlphaBlendOpaque, ColorAlphaBlendTransparent
pub trait ColorAlphaBlendMode: Copy {
    type ColorContext;

    fn prepare_color(&self, color: Rgba) -> Self::ColorContext;
    fn blend_color(&self, bg: &mut Rgba, color_context: &Self::ColorContext, alpha: u8);
    fn blend_solid_color(&self, bg: &mut Rgba, color_context: &Self::ColorContext) {
        self.blend_color(bg, color_context, 255);
    }
}

// Default implementations: ImageBlendOverwrite, ImageBlendOpaque, ImageBlendTransparent
pub trait ImageBlendMode: Copy {
    fn blend_color(&self, bg: &mut Rgba, color: Rgba);
}

// ===================== Helper Functions =====================

fn alpha_blend(base: &mut Rgba, color: Rgba) {
    let (r_below, g_below, b_below, a_below) = base.rgba_f32();
    let (r_above, g_above, b_above, a_above) = color.rgba_f32();

    let (ratio_below, ratio_above) = {
        let tmp_below = (1.0 - a_above) * a_below;
        let total = a_above + tmp_below;
        if total <= 0.0001 {
            (0.5, 0.5)
        } else {
            (tmp_below / total, a_above / total)
        }
    };

    let r = ratio_below * r_below + ratio_above * r_above;
    let g = ratio_below * g_below + ratio_above * g_above;
    let b = ratio_below * b_below + ratio_above * b_above;
    // From: https://stackoverflow.com/questions/3658804/formula-for-alpha-value-when-blending-two-transparent-colors
    let a = a_below + (1.0 - a_below) * a_above;

    *base = Rgba::from_f32(r, g, b, a);
}

fn fast_alpha_blend_opaque(base: &mut Rgba, color: Rgba) {
    let (r1, g1, b1, a1) = color.rgba();
    let (r1, g1, b1, a1) = (r1 as i32, g1 as i32, b1 as i32, a1 as i32);

    let (r2, g2, b2) = base.rgb();
    let (r2, g2, b2) = (r2 as i32, g2 as i32, b2 as i32);

    let r3 = fast_alpha_blend_opaque_calc(a1, r1, r2);
    let g3 = fast_alpha_blend_opaque_calc(a1, g1, g2);
    let b3 = fast_alpha_blend_opaque_calc(a1, b1, b2);

    base.0[0] = r3;
    base.0[1] = g3;
    base.0[2] = b3;
}

fn fast_alpha_blend_opaque_calc(alpha: i32, src: i32, dest: i32) -> u8 {
    // From: https://www.gamedev.net/forums/topic/34688-alpha-blend-formula/
    (((alpha * (src - dest)) >> 8) + dest) as u8
}

// ===================== ColorBlendMode Implementations =====================

#[derive(Copy, Clone, Debug)]
pub struct ColorBlendOverwrite;
impl ColorBlendMode for ColorBlendOverwrite {
    type ColorContext = Rgba;

    fn prepare_color(&self, color: Rgba) -> Self::ColorContext {
        color
    }
    fn blend_color(&self, bg: &mut Rgba, color_context: &Self::ColorContext) {
        *bg = *color_context;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ColorBlendOpaque;
impl ColorBlendMode for ColorBlendOpaque {
    type ColorContext = Rgba;

    fn prepare_color(&self, color: Rgba) -> Self::ColorContext {
        color
    }
    fn blend_color(&self, bg: &mut Rgba, color_context: &Self::ColorContext) {
        // TODO Use premultiplied-alpha without updating existing alpha value
        fast_alpha_blend_opaque(bg, *color_context);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ColorBlendTransparent;
impl ColorBlendMode for ColorBlendTransparent {
    type ColorContext = Rgba;

    fn prepare_color(&self, color: Rgba) -> Self::ColorContext {
        color
    }
    fn blend_color(&self, bg: &mut Rgba, color_context: &Self::ColorContext) {
        // TODO Use premultiplied-alpha
        alpha_blend(bg, *color_context);
    }
}

// ===================== ColorAlphaBlendMode Implementations =====================

#[derive(Copy, Clone, Debug)]
pub struct SolidColorAlphaBlendOverwrite;
impl ColorAlphaBlendMode for SolidColorAlphaBlendOverwrite {
    type ColorContext = Rgba;

    fn prepare_color(&self, color: Rgba) -> Self::ColorContext {
        color
    }
    fn blend_color(&self, bg: &mut Rgba, color_context: &Self::ColorContext, alpha: u8) {
        *bg = *color_context;
        bg.set_alpha(alpha);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ColorAlphaBlendOverwrite;
impl ColorAlphaBlendMode for ColorAlphaBlendOverwrite {
    type ColorContext = Rgba;

    fn prepare_color(&self, color: Rgba) -> Self::ColorContext {
        color
    }
    fn blend_color(&self, bg: &mut Rgba, color_context: &Self::ColorContext, alpha: u8) {
        let mut color = *color_context;
        let value = (color.alpha() as u32) * (alpha as u32) / 255;
        color.set_alpha(value as u8);
        *bg = color;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ColorAlphaBlendOpaque;
impl ColorAlphaBlendMode for ColorAlphaBlendOpaque {
    type ColorContext = Rgba;

    fn prepare_color(&self, color: Rgba) -> Self::ColorContext {
        color
    }
    fn blend_color(&self, bg: &mut Rgba, color_context: &Self::ColorContext, alpha: u8) {
        // TODO Use premultiplied-alpha and ignore existing alpha value
        let mut color = *color_context;
        let value = (color.alpha() as u32) * (alpha as u32) / 255;
        color.set_alpha(value as u8);
        fast_alpha_blend_opaque(bg, color);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ColorAlphaBlendTransparent;
impl ColorAlphaBlendMode for ColorAlphaBlendTransparent {
    type ColorContext = Rgba;

    fn prepare_color(&self, color: Rgba) -> Self::ColorContext {
        color
    }
    fn blend_color(&self, bg: &mut Rgba, color_context: &Self::ColorContext, alpha: u8) {
        // TODO Use premultiplied-alpha
        let mut color = *color_context;
        let value = (color.alpha() as u32) * (alpha as u32) / 255;
        color.set_alpha(value as u8);
        alpha_blend(bg, color);
    }
}

// ===================== ImageBlendMode Implementations =====================

#[derive(Copy, Clone, Debug)]
pub struct ImageBlendOverwrite;
impl ImageBlendMode for ImageBlendOverwrite {
    fn blend_color(&self, bg: &mut Rgba, color: Rgba) {
        *bg = color;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ImageBlendOpaque;
impl ImageBlendMode for ImageBlendOpaque {
    fn blend_color(&self, bg: &mut Rgba, color: Rgba) {
        fast_alpha_blend_opaque(bg, color);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ImageBlendTransparent;
impl ImageBlendMode for ImageBlendTransparent {
    fn blend_color(&self, bg: &mut Rgba, color: Rgba) {
        alpha_blend(bg, color);
    }
}
