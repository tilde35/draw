use crate::img::Image;

pub trait Indexable2D {
    fn try_as_index(&self, img: &Image) -> Option<usize>;
    fn is_valid(&self, img: &Image) -> bool {
        self.try_as_index(img).is_some()
    }
    fn as_index(&self, img: &Image) -> usize {
        if let Some(idx) = self.try_as_index(img) {
            idx
        } else {
            panic!("{}", &self.out_of_bounds_text(img))
        }
    }
    fn out_of_bounds_text(&self, img: &Image) -> String;
    fn try_as_xy_pos(&self, img: &Image) -> Option<[u32; 2]>;
}

impl Indexable2D for usize {
    fn try_as_index(&self, img: &Image) -> Option<usize> {
        if *self >= img.buffer().len() {
            None
        } else {
            Some(*self)
        }
    }
    fn out_of_bounds_text(&self, img: &Image) -> String {
        format!(
            "The buffer index {:?} is not valid for the image of size {:?} (buffer length = {})",
            self,
            img.dim(),
            img.buffer().len()
        )
    }
    fn try_as_xy_pos(&self, img: &Image) -> Option<[u32; 2]> {
        if let Some(idx) = self.try_as_index(img) {
            let stride = img.stride();
            let x = idx % stride;
            let y = idx / stride;
            Some([x as u32, y as u32])
        } else {
            None
        }
    }
}

impl Indexable2D for isize {
    fn try_as_index(&self, img: &Image) -> Option<usize> {
        if *self < 0 || (*self as usize) >= img.buffer().len() {
            None
        } else {
            Some(*self as usize)
        }
    }
    fn out_of_bounds_text(&self, img: &Image) -> String {
        format!(
            "The buffer index {:?} is not valid for the image of size {:?} (buffer length = {})",
            self,
            img.dim(),
            img.buffer().len()
        )
    }
    fn try_as_xy_pos(&self, img: &Image) -> Option<[u32; 2]> {
        if let Some(idx) = self.try_as_index(img) {
            let stride = img.stride();
            let x = idx % stride;
            let y = idx / stride;
            Some([x as u32, y as u32])
        } else {
            None
        }
    }
}

impl Indexable2D for [i32; 2] {
    fn try_as_index(&self, img: &Image) -> Option<usize> {
        let [x, y] = *self;
        let [w, h] = img.dim();
        if x < 0 || x >= (w as i32) || y < 0 || y >= (h as i32) {
            None
        } else {
            Some((x as usize) + (y as usize) * (w as usize))
        }
    }
    fn out_of_bounds_text(&self, img: &Image) -> String {
        format!(
            "The pixel index {:?} is not valid for the image of size {:?}",
            self,
            img.dim()
        )
    }
    fn try_as_xy_pos(&self, img: &Image) -> Option<[u32; 2]> {
        if self.is_valid(img) {
            Some([self[0] as u32, self[1] as u32])
        } else {
            None
        }
    }
}
impl Indexable2D for (i32, i32) {
    fn try_as_index(&self, img: &Image) -> Option<usize> {
        let (x, y) = *self;
        let [w, h] = img.dim();
        if x < 0 || x >= (w as i32) || y < 0 || y >= (h as i32) {
            None
        } else {
            Some((x as usize) + (y as usize) * (w as usize))
        }
    }
    fn out_of_bounds_text(&self, img: &Image) -> String {
        format!(
            "The pixel index {:?} is not valid for the image of size {:?}",
            self,
            img.dim()
        )
    }
    fn try_as_xy_pos(&self, img: &Image) -> Option<[u32; 2]> {
        if self.is_valid(img) {
            Some([self.0 as u32, self.1 as u32])
        } else {
            None
        }
    }
}

impl Indexable2D for [u32; 2] {
    fn try_as_index(&self, img: &Image) -> Option<usize> {
        let [x, y] = *self;
        let [w, h] = img.dim();
        if x >= w || y >= h {
            None
        } else {
            Some((x as usize) + (y as usize) * (w as usize))
        }
    }
    fn out_of_bounds_text(&self, img: &Image) -> String {
        format!(
            "The pixel index {:?} is not valid for the image of size {:?}",
            self,
            img.dim()
        )
    }
    fn try_as_xy_pos(&self, img: &Image) -> Option<[u32; 2]> {
        if self.is_valid(img) {
            Some(*self)
        } else {
            None
        }
    }
}
impl Indexable2D for (u32, u32) {
    fn try_as_index(&self, img: &Image) -> Option<usize> {
        let (x, y) = *self;
        let [w, h] = img.dim();
        if x >= w || y >= h {
            None
        } else {
            Some((x as usize) + (y as usize) * (w as usize))
        }
    }
    fn out_of_bounds_text(&self, img: &Image) -> String {
        format!(
            "The pixel index {:?} is not valid for the image of size {:?}",
            self,
            img.dim()
        )
    }
    fn try_as_xy_pos(&self, img: &Image) -> Option<[u32; 2]> {
        if self.is_valid(img) {
            Some([self.0, self.1])
        } else {
            None
        }
    }
}
