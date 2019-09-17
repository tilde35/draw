use draw::*;

fn main() {
    let mut img = Image::new_with_color([800, 600], Rgba([255, 255, 255, 255]));

    let font_cache = FontCache::ttf_from_static(include_bytes!("carlito/Carlito-Regular.ttf")).unwrap();

    img.as_canvas().draw_text(
        &mut font_cache.font(),
        48,
        Rgba([0, 0, 0, 255]),
        "Hello World!",
        [0, 0],
        None,
    );

    img.save("text_drawing.png").unwrap();
}
