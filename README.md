# draw #

Simple-to-use Rust image manipulation library.

## Cargo.toml Dependency Setup ##

Currently, this is not a crate configured in `crates.io` and must
be added as a git dependency.

```toml
[dependencies]
draw = { git = "https://github.com/tilde35/draw", branch = "v3" }
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
let s = img.sub_image([10, 10], [16, 16]); // [x,y], [w,h]

let sprites = img.sub_images([16, 16])
                 .with_margin(4)
                 .with_spacing(1)
                 .create();

let prms = SubImageParams::size([16, 16])
           .with_margin([4, 4, 4, 4])
           .with_spacing([1, 1]);
let sprites = img.sub_images_from(&prms);
```

## Image - Resizing ##

```rust
let img = Image::open("sample.png")?;
let icon = img.resize([16, 16], ResizeFilter::Nearest);
```

## Color - Creation and Alternative Formats ##

```rust
let clear = Rgba([0, 0, 0, 0]);
let red = Rgba::from_argb_u32(0xff_ff0000);
let blue: Rgba = "#00f".parse().unwrap();
let gray = Rgba::from_f32([0.5, 0.5, 0.5, 1.0]);

let [r, g, b, a] = red.rgba_f32();
let [r, g, b] = red.rgb_f32();
let red_u32 = red.to_rgba_u32(); // Use Rgba::from_argb to get it back
```

## Canvas ##

```rust
let mut c = img.as_canvas();

c.clear(Rgba([0, 0, 0, 255]));
c.fill(Rgba([80, 80, 80, 128]));

if let Some(mut sc) = c.sub_canvas([-4, -8], [16, 16]) {
    // Note: Location and dimensions are adjusted based on
    // available space.
    assert!(sc.pos() == [0, 0]);
    assert!(sc.dim() == [12, 8]);

    let its_gone = sc.into_sub_canvas([40, 40], [8, 8]);
}

c.draw_rect([3, 3], [400, 200], Rgba([40, 0, 0, 80]));

c.fill_rect([3, 3], [400, 200], Rgba([40, 0, 0, 80]));

c.draw_image(&sprite_img, [10, 10]);

let mut font = FontCache::from_static(include_bytes!("Arial.ttf")).unwrap();

c.draw_text(
    &mut font,             // Font
    24.0,                  // Font size
    Rgba([0, 0, 0, 255]),  // Font color
    "Example text",        // Text
    [10, 10],              // Position
    Some(200),             // Width (for text wrapping)
    None,                  // Indentation (first line only)
);

let r = font.render("Example", 24.0, Some(200), None);
c.draw_rendered_text(&r, Rgba([0, 0, 0, 255]), [10, 10], 0);
```

## GLIUM ##

```rust
let img_tx = {
    let mut img = Image::open("my_file.png").unwrap();
    // Flip in y-direction, adjust alpha transparency color
    img.transform_for_3dgfx();

    let raw = glium::texture::RawImage2d {
        data: std::borrow::Cow::Borrowed(img.raw_rgba_bytes()),
        width: img.width(),
        height: img.height(),
        format: glium::texture::ClientFormat::U8U8U8U8,
    };
    glium::texture::SrgbTexture2d::new(&display, raw).unwrap()
};

let img_tx_array = {
    let img = Image::open("my_file.png").unwrap();

    let sub_imgs = img.sub_images([128, 128])
        .with_margin(1)
        .with_spacing(1)
        .with_transform_for_3dgfx()
        .create();

    let mut raw_entries = Vec::new();
    for i in sub_imgs.iter() {
        let raw = glium::texture::RawImage2d {
            data: std::borrow::Cow::Borrowed(i.raw_rgba_bytes()),
            width: i.width(),
            height: i.height(),
            format: glium::texture::ClientFormat::U8U8U8U8,
        };
        raw_entries.push(raw);
    }

    glium::texture::SrgbTexture2dArray::new(&display, raw_entries).unwrap()
};
```

## NSVG ##

```rust
// Dependency:
// nsvg = { version = "0.5.0", default-features = false }
fn create_ico(c: Rgba) -> Image {
    let s = include_str!("star.svg");
    let s = s.replace(
        "#00ff00",
        &format!("#{:02x}{:02x}{:02x}", c.red(), c.green(), c.blue()),
    );
    let svg = nsvg::parse_str(&s, nsvg::Units::Pixel, 96.0).unwrap();
    let (width, height, raw_rgba) = svg.rasterize_to_raw_rgba(1.0).unwrap();
    Image::from_raw_rgba_bytes([width, height], &raw_rgba)
}
```

## Font Loading ##

```rust
let font = FontCache::ttf_from_static(include_bytes!("Arial.ttf")).unwrap();
let font = FontCache::ttf_from_vec(bytes).unwrap();
let font = FontCache::ttf_from_file("Arial.ttf").unwrap();

// Create a hashmap that relates a character to the SVG contents and scaling factor (between 0.0 and 1.0).
let mut svg_data: HashMap<char, (String, f32)> = HashMap::new();
svg_data.insert('*', (include_str!("star.svg").into(), 1.0));
let svg_font = FontCache::svg_from_text(svg_data);
```

## Blend Modes ##

By default, the draw operations will use the blending that will work in all circumstances (may not be the fastest).
If a faster performing blend strategy is required, then the `_using` version of the method may be called and the
appropriate blend strategy specified.

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
