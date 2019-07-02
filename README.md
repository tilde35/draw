# draw #

Simple-to-use Rust image manipulation library.

## Cargo.toml Dependency Setup ##

Currently, this is not a crate configured in `crates.io` and must
be added as a git dependency.

```toml
[dependencies]
draw = { git = "https://github.com/tilde35/draw", branch = "v1" }
```

## Image - Creation and I/O ##

```rust
// Create a new image
let img = Image::new([800, 600]);
let img = Image::new_with_color([600, 600], Rgba([255, 255, 255, 255]));

// Load an existing image
let img = Image::open("sample.png")?;
let img = Image::open_bytes(include_bytes!("sample.png"))?;

// Save image
img.save("output.png")?;
```

## Image - Subparts ##

```rust
// Create a new image given the specified rectangle within the original image
let s = img.sub_image(10, 10, 16, 16); // x,y, w,h

let prms = SubImageParams::size(16, 16)
           .with_margin(4, 4, 4, 4)
           .with_spacing(1, 1);
let sprites = img.sub_images(&prms);
```

## Color - Creation and Alternative Formats ##

```rust
let clear = Rgba([0, 0, 0, 0]);
let red = Rgba::from_argb(0xff_ff0000);
let blue = Rgba::parse("#00f").unwrap();
let green = Rgba::from_u8(0, 255, 0, 255);
let gray = Rgba::from_f32(0.5, 0.5, 0.5, 1.0);

let (r, g, b, a) = red.rgba_f32();
let (r, g, b) = red.rgb_f32();
let red_u32 = red.to_rgba_u32(); // Use Rgba::from_argb to get it back
```

## Canvas ##

```rust
let mut s = img.as_canvas();

s.clear(Rgba([0,0,0,0]));
s.fill(ColorBlendOpaque, Rgba([80,80,80,128]));

if let Some(mut ss) = s.sub_canvas(-4, -8, 16, 16) {
    // Note: Location and dimensions are adjusted based on
    // available space.
    assert!(ss.loc() == [0, 0]);
    assert!(ss.dim() == [12, 8]);

    let its_gone = ss.into_sub_canvas(40, 40, 8, 8);
}

s.draw_rect(ColorBlendOpaque, 3, 3, 400, 200, Rgba([40,0,0,80]));

s.fill_rect(ColorBlendOpaque, 3, 3, 400, 200, Rgba([40,0,0,80]));

s.draw_image(ImageBlendOpaque, &sprite_img, 10, 10);

s.draw_text(
    ColorAlphaBlendOpaque, // Blending mode
    &mut font,             // Font
    24.0,                  // Font size
    Rgba([0, 0, 0, 255]),  // Font color
    "Example text",        // Text
    10,                    // X position
    10,                    // Y position
    Some(200)              // Width (for text wrapping)
);

let r = font.render_text("Example", 24.0, Some(200));
s.draw_rendered_text(
    ColorAlphaBlendOpaque,
    &r,
    font.line_advance_height(24.0),
    Rgba([0, 0, 0, 255]),
    10,
    10
);
```

## Blend Modes ##

### ColorBlendMode ###

Used when working with a single color without variation. Default implementations
are (ranked in order of fastest to slowest):

* **ColorBlendOverwrite:** Overwrites the base color regardless of source color's transparency.
* **ColorBlendOpaque:** Performs drawing while handling source color transparency assuming the background is opaque (no transparency).
* **ColorBlendTransparent:** Performs drawing while handling any combination of transparency.

### ColorAlphaBlendMode ###

Used when working with a single color, but the transparency will be varied. Default
implementations are (ranked in order of fastest to slowest):

* **SolidColorAlphaBlendOverwrite:** Assumes the source color is opaque, overwrites existing background.
* **ColorAlphaBlendOverwrite:** Overwrites background with the resulting color. Note, if the source color is transparent, then the opacity will never be greater than the source's alpha value.
* **ColorAlphaBlendOpaque:** Performs drawing while handling any relevant alpha blending assuming the background is opaque.
* **ColorAlphaBlendTransparent:** Performs drawing while handling any relevant alpha blending, regardless of source/background transparency.

### ImageBlendMode ###

Used when working with a combination of colors (such as with images).

* **ImageBlendOverwrite:** Overwrites the base color regardless of source color's transparency.
* **ImageBlendOpaque:** Performs drawing while handling source color transparency, assumes background color is opaque.
* **ImageBlendTransparent:** Performs drawing while handling any combination of transparency.

## GLIUM ##

```rust
let raw = glium::texture::RawImage2d {
    data: std::borrow::Cow::Borrowed(img.rgba_data()),
    width: img.dimensions().0,
    height: img.dimensions().1,
    format: glium::texture::ClientFormat::U8U8U8U8,
};
```

## Fonts ##

```rust
let font = FontCache::from_static(include_bytes!("Arial.ttf"));
let font = FontCache::from_slice(bytes);
let font = FontCache::from_vec(bytes);
let font = FontCache::from_font(rusttype_font);

let font_size = 24.0;
let line_height = font.line_advance_height(font_size);
let r = font.render("Example", font_size, Some(200));
let r = font.cache_only_render("Example", font_size, Some(200));

let width = r.get_total_width();
let height = r.get_total_height();
let (mut x, mut y) = (0, 0);
for i in r.get_instructions() {
    match i {
        RenderedTextInstruction::RenderGlyph(g) => {
            g.render_xy(
                x,
                y,
                self,
                |s, x, y| if let Some(dst) = s.try_get_color_mut(x, y) {
                    mode.blend_solid_color(dst, color_ctxt);
                },
                |s, x, y, alpha| if let Some(dst) = s.try_get_color_mut(x, y) {
                    mode.blend_color(dst, color_ctxt, alpha);
                },
            );
            x += g.advance_width;
        },
        RenderedTextInstruction::Kerning(dx) => {
            x += dx;
        }
        RenderedTextInstruction::NextLine(dy, _reason) => {
            y += dy;
        }
    }
}

```
