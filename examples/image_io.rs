use draw::*;

pub fn main() {
    let new_img = Image::new([128, 128]);

    new_img.save("example_file.png").unwrap();

    Image::open("example_file.png").unwrap();
}
