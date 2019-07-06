use draw::*;

fn main() {
    {
        let mut img = Image::new([360, 128]);

        for y in 0..img.height() {
            for x in 0..img.width() {
                let hue = x as f32;
                let saturation = 1.0f32;
                let value = 1.0 - (y as f32) / (img.height() as f32 - 1.0);

                let c = Hsv::new([hue, saturation, value, 1.0]);
                let c: Rgba = c.into();
                img.set([x, y], c);
            }
        }

        img.save("hsv.png").unwrap();
    }

    {
        let mut img = Image::new([360, 128]);

        for y in 0..img.height() {
            for x in 0..img.width() {
                let hue = x as f32;
                let saturation = 1.0f32;
                let lightness = 1.0 - (y as f32) / (img.height() as f32 - 1.0);

                let c = Hsl::new([hue, saturation, lightness, 1.0]);
                let c: Rgba = c.into();
                img.set([x, y], c);
            }
        }

        img.save("hsl.png").unwrap();
    }
}
