use crate::blend::{ColorBlendMode, ColorBlendTransparent};
use crate::canvas::Canvas;
use crate::errors::ImageLoadError;
use crate::idx::Indexable2D;
use crate::resize_filter::ResizeFilter;
use crate::rgba::Rgba;
use crate::rows::{RowsIter, RowsMutIter};
use crate::sub_img_params::{SubImageBuilder, SubImageParams};
use image::ImageError;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Image {
    dim: [u32; 2],
    contents: Vec<Rgba>,
}

impl From<image::RgbaImage> for Image {
    fn from(image: image::RgbaImage) -> Self {
        use image::Pixel;

        let d = image.dimensions();
        let d = [d.0, d.1];
        let len = (d[0] as usize) * (d[1] as usize);
        let mut buf = Vec::with_capacity(len);
        for (_x, _y, pixel) in image.enumerate_pixels() {
            let c = pixel.channels();
            buf.push(Rgba([c[0], c[1], c[2], c[3]]));
        }
        Image {
            dim: d,
            contents: buf,
        }
    }
}

impl Image {
    pub fn new(dim: [u32; 2]) -> Image {
        Self::new_with_color(dim, Rgba([0, 0, 0, 0]))
    }
    pub fn new_with_color(dim: [u32; 2], bg: Rgba) -> Image {
        let [width, height] = dim;
        let len = (width as usize) * (height as usize);
        let mut buf = Vec::with_capacity(len);
        for _ in 0..len {
            buf.push(bg);
        }
        Image {
            dim: [width, height],
            contents: buf,
        }
    }

    pub fn resize(&self, new_dim: [u32; 2], filter: ResizeFilter) -> Image {
        let img = self.to_piston_image();
        image::imageops::resize(&img, new_dim[0], new_dim[1], filter.as_filter_type()).into()
    }

    pub fn from_raw_rgba_bytes(dim: [u32; 2], raw: &[u8]) -> Image {
        let [width, height] = dim;
        let len = (width as usize) * (height as usize);
        if raw.len() != len * 4 {
            panic!(
                "Dimensions do not match: {}x{}x4={}, raw buffer length is {}",
                width,
                height,
                len * 4,
                raw.len()
            );
        }
        let mut buf = Vec::with_capacity(len);
        for idx in 0..len {
            let pixel_idx = idx * 4;
            let r = raw[pixel_idx + 0];
            let g = raw[pixel_idx + 1];
            let b = raw[pixel_idx + 2];
            let a = raw[pixel_idx + 3];
            buf.push(Rgba([r, g, b, a]));
        }
        Image {
            dim: [width, height],
            contents: buf,
        }
    }

    pub fn open(file: impl AsRef<std::path::Path>) -> Result<Image, ImageLoadError> {
        Ok(image::open(file)?.to_rgba8().into())
    }

    pub fn open_bytes(buffer: &[u8]) -> Result<Image, ImageLoadError> {
        Ok(image::load_from_memory(buffer)?.to_rgba8().into())
    }

    /// Converts this image into linear color space (ex. what OpenGL uses). Since this is
    /// a lossy transformation, it is best to use the built-in functions from the graphics
    /// libraries instead (such as the SrgbTexture2d in glium).
    pub fn srgb_to_linear(&mut self) {
        for p in &mut self.contents {
            *p = p.srgb_to_linear();
        }
    }

    pub fn dim(&self) -> [u32; 2] {
        self.dim
    }
    pub fn width(&self) -> u32 {
        self.dim[0]
    }
    pub fn height(&self) -> u32 {
        self.dim[1]
    }

    pub fn buffer<'a>(&'a self) -> &'a [Rgba] {
        &self.contents[..]
    }

    pub fn buffer_mut<'a>(&'a mut self) -> &'a mut [Rgba] {
        &mut self.contents[..]
    }

    pub fn stride(&self) -> usize {
        self.dim[0] as usize
    }

    pub fn flip_y(&mut self) {
        // TODO Implement this efficiently
        let mut result = Vec::with_capacity(self.contents.len());
        for y in 0..self.dim[1] {
            for x in 0..self.dim[0] {
                let flip_y = self.dim[1] - y - 1;
                result.push(self.get([x, flip_y]));
            }
        }
        std::mem::swap(&mut self.contents, &mut result);
    }

    pub fn flip_x(&mut self) {
        // TODO Implement this efficiently
        let mut result = Vec::with_capacity(self.contents.len());
        for y in 0..self.dim[1] {
            for x in 0..self.dim[0] {
                let flip_x = self.dim[0] - x - 1;
                result.push(self.get([flip_x, y]));
            }
        }
        std::mem::swap(&mut self.contents, &mut result);
    }

    pub fn get(&self, pos: impl Indexable2D) -> Rgba {
        let idx = pos.as_index(self);
        self.contents[idx]
    }
    pub fn get_mut<'a>(&'a mut self, pos: impl Indexable2D) -> &'a mut Rgba {
        let idx = pos.as_index(self);
        &mut self.contents[idx]
    }
    pub fn set(&mut self, pos: impl Indexable2D, color: Rgba) {
        let idx = pos.as_index(self);
        self.contents[idx] = color;
    }
    pub fn blend(&mut self, pos: impl Indexable2D, color: Rgba) {
        self.blend_using(ColorBlendTransparent, pos, color);
    }
    pub fn blend_using(&mut self, mode: impl ColorBlendMode, pos: impl Indexable2D, color: Rgba) {
        let idx = self.index_at(pos);
        let cc = mode.prepare_color(color);
        mode.blend_color(&mut self.contents[idx], &cc);
    }
    pub fn try_get(&self, pos: impl Indexable2D) -> Option<Rgba> {
        if let Some(idx) = pos.try_as_index(self) {
            Some(self.contents[idx])
        } else {
            None
        }
    }
    pub fn try_get_mut<'a>(&'a mut self, pos: impl Indexable2D) -> Option<&'a mut Rgba> {
        if let Some(idx) = pos.try_as_index(self) {
            Some(&mut self.contents[idx])
        } else {
            None
        }
    }
    pub fn try_set(&mut self, pos: impl Indexable2D, color: Rgba) -> bool {
        if let Some(idx) = pos.try_as_index(self) {
            self.contents[idx] = color;
            true
        } else {
            false
        }
    }
    pub fn try_blend(&mut self, pos: impl Indexable2D, color: Rgba) -> bool {
        self.try_blend_using(ColorBlendTransparent, pos, color)
    }
    pub fn try_blend_using(
        &mut self,
        mode: impl ColorBlendMode,
        pos: impl Indexable2D,
        color: Rgba,
    ) -> bool {
        if let Some(idx) = pos.try_as_index(self) {
            let cc = mode.prepare_color(color);
            mode.blend_color(&mut self.contents[idx], &cc);
            true
        } else {
            false
        }
    }

    pub fn index_at(&self, pos: impl Indexable2D) -> usize {
        pos.as_index(self)
    }
    pub fn try_index_at(&self, pos: impl Indexable2D) -> Option<usize> {
        pos.try_as_index(self)
    }

    pub fn set_height(&mut self, h: u32) {
        if h == 0 {
            panic!("Height must be greater than zero");
        }

        let len = (h as usize) * self.stride();
        self.dim[1] = h;

        if len <= self.contents.len() {
            // Shrink
            self.contents.truncate(len);
        } else {
            // Grow
            let diff = len - self.contents.len();
            self.contents.reserve(diff);
            for _ in 0..diff {
                self.contents.push(Rgba([0, 0, 0, 0]));
            }
        }
    }

    pub fn sub_image(&self, pos: impl Indexable2D, dim: [u32; 2]) -> Image {
        let [w, h] = dim;
        if w == 0 || h == 0 {
            panic!(
                "Subimage width and height must be greater than zero (width={}, height={})",
                w, h
            );
        }
        let [x, y] = if let Some(pos) = pos.try_as_xy_pos(self) {
            pos
        } else {
            panic!("{}", pos.out_of_bounds_text(self))
        };
        let d = self.dim;
        if x + w > d[0] || y + h > d[1] {
            panic!(
                "Subimage at ({},{}) size {} x {} does not fit on main image {} x {}",
                x, y, w, h, d[0], d[1]
            );
        }

        let mut buf = Vec::with_capacity((w as usize) * (h as usize));

        let mut row_idx = pos.as_index(self);
        let stride = self.stride();
        for _ in 0..h {
            let mut idx = row_idx;
            for _ in 0..w {
                buf.push(self.contents[idx]);
                idx += 1;
            }
            row_idx += stride;
        }

        Image {
            dim: [w, h],
            contents: buf,
        }
    }

    pub fn sub_images_from(&self, params: &SubImageParams) -> Vec<Image> {
        let mut result = Vec::new();
        for pos in params.iter_for_dimensions(self.dim()) {
            let mut img = self.sub_image(pos, params.size);
            if params.transform_for_3dgfx {
                img.transform_for_3dgfx();
            }
            result.push(img);
        }
        result
    }

    pub fn sub_images<'a>(&'a self, size_dim: [u32; 2]) -> SubImageBuilder<'a> {
        SubImageBuilder::new(self, size_dim)
    }

    fn to_piston_image(&self) -> image::RgbaImage {
        let mut imgbuf: image::RgbaImage = image::ImageBuffer::new(self.dim[0], self.dim[1]);
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            // Note: Assuming the enumeration happens in standard order, then this could be a
            // simple index increment instead.
            let c = self.get([x, y]).0;
            *pixel = image::Rgba(c);
        }
        imgbuf
    }

    pub fn save(&self, file: impl AsRef<std::path::Path>) -> Result<(), ImageError> {
        self.to_piston_image().save(file)
    }

    pub fn rows<'a>(&'a self) -> RowsIter<'a> {
        RowsIter::new(self, [0, 0], self.dim)
    }
    pub fn rows_at<'a>(&'a self, pos: impl Indexable2D, dim: [u32; 2]) -> RowsIter<'a> {
        let [x, y] = if let Some(pos) = pos.try_as_xy_pos(self) {
            pos
        } else {
            panic!("{}", pos.out_of_bounds_text(self))
        };
        RowsIter::new(self, [x, y], dim)
    }
    pub fn rows_mut<'a>(&'a mut self) -> RowsMutIter<'a> {
        RowsMutIter::new(self, [0, 0], self.dim)
    }
    pub fn rows_mut_at<'a>(
        &'a mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> RowsMutIter<'a> {
        RowsMutIter::new(self, [x, y], [width, height])
    }

    pub fn as_canvas<'a>(&'a mut self) -> Canvas<'a> {
        let d = self.dim;
        Canvas::new(self, 0, [0, 0], d)
    }

    /// Returns the raw data in RGBA format.
    pub fn raw_rgba_bytes(&self) -> &[u8] {
        unsafe {
            // This is the dangerous part: Create a [u8] slice from the raw pointer.
            // It relies on Rgba to contain exactly four u8 values without any padding/extras.
            let u8_len = self.contents.len() * 4;
            let rgba_slice = &self.contents[0] as *const _ as *const u8;
            std::slice::from_raw_parts(rgba_slice, u8_len)
        }
    }

    pub fn transform_for_3dgfx(&mut self) {
        self.create_alpha_color_for_3dgfx();
        self.flip_y();
    }

    pub fn create_alpha_color_for_3dgfx(&mut self) {
        self.create_alpha_color_for_3dgfx_using(16, 0);
    }
    pub fn create_alpha_color_for_3dgfx_using(&mut self, alpha_threshold: u8, alpha_result: u8) {
        // Note: This is not performance-critical
        let mut visited = Vec::with_capacity(self.contents.len());
        for _ in 0..self.contents.len() {
            visited.push(false);
        }

        let neighboors = [
            (0.7f32, [-1, -1]),
            (1.0f32, [0, -1]),
            (0.7f32, [1, -1]),
            (1.0f32, [-1, 0]),
            (1.0f32, [1, 0]),
            (0.7f32, [-1, 1]),
            (1.0f32, [0, 1]),
            (0.7f32, [1, 1]),
        ];

        // Step 1: Gather all transparent pixels next to a non-transparent pixel
        let mut to_visit: Vec<[i32; 2]> = Vec::with_capacity(self.contents.len());
        let mut next_set: Vec<[i32; 2]> = Vec::with_capacity(self.contents.len());
        for y in 0..(self.height() as i32) {
            for x in 0..(self.width() as i32) {
                let pos = [x, y];
                if self.get(pos).alpha() > alpha_threshold {
                    continue;
                }
                let mut has_neighbor = false;
                for (_, dpos) in neighboors.iter().cloned() {
                    let npos = [pos[0] + dpos[0], pos[1] + dpos[1]];
                    if let Some(c) = self.try_get(npos) {
                        if c.alpha() > alpha_threshold {
                            has_neighbor = true;
                        }
                    }
                }
                if has_neighbor {
                    to_visit.push(pos);
                }
            }
        }

        // Step 2: For every transparent pixel: blend all visited/non-transparent pixel colors
        while to_visit.len() > 0 {
            // Update the next round of transparent pixels
            for pos in to_visit.iter().cloned() {
                // RGB + weight
                let mut result = [0.0, 0.0, 0.0, 0.0f32];
                // Get surrounding colors
                for (weight, dpos) in neighboors.iter().cloned() {
                    let npos = [pos[0] + dpos[0], pos[1] + dpos[1]];
                    self.gfx_combine(alpha_threshold, &mut result, &visited, weight, npos);
                }
                for (_, dpos) in neighboors.iter().cloned() {
                    let npos = [pos[0] + dpos[0], pos[1] + dpos[1]];
                    if let Some(nidx) = self.try_index_at(npos) {
                        let c = self.contents[nidx];
                        if c.alpha() <= alpha_threshold {
                            if !next_set.contains(&npos)
                                && !to_visit.contains(&npos)
                                && !visited[nidx]
                            {
                                next_set.push(npos);
                            }
                        }
                    }
                }

                result[3] = (alpha_result as f32) / 255.0;
                *self.get_mut(pos) = Rgba::from_f32(result);
            }

            // Mark everything as visited, move next_set into to_visit
            for pos in to_visit.iter().cloned() {
                if let Some(idx) = self.try_index_at(pos) {
                    visited[idx as usize] = true;
                }
            }
            to_visit.clear();
            std::mem::swap(&mut to_visit, &mut next_set);
        }
    }
    fn gfx_combine(
        &self,
        alpha_threshold: u8,
        result: &mut [f32; 4],
        visited: &Vec<bool>,
        weight: f32,
        pos: [i32; 2],
    ) {
        if let Some(idx) = self.try_index_at(pos) {
            let c = self.contents[idx];
            if visited[idx] || c.alpha() > alpha_threshold {
                let total_weight = result[3] + weight;
                let pct = weight / total_weight;
                let rgb = c.rgb_f32();
                result[0] = (1.0 - pct) * result[0] + pct * rgb[0];
                result[1] = (1.0 - pct) * result[1] + pct * rgb[1];
                result[2] = (1.0 - pct) * result[2] + pct * rgb[2];
                result[3] = total_weight;
            }
        } else {
            // Nothing to do (not a valid index)
        }
    }
}
