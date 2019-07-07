use draw::*;

fn main() {
    let mut img = Image::new_with_color([800, 600], Rgba([255,255,255,255]));

    let mut font = FontCache::open("examples/carlito/Carlito-Regular.ttf").unwrap();

    img.as_canvas().draw_text(&mut font, 48.0, Rgba([0,0,0,255]), "Hello World!", [0, 0], None);

    img.save("text_drawing.png").unwrap();
}
