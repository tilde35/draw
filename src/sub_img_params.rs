use crate::img::Image;

pub struct SubImageBuilder<'a> {
    img: &'a Image,
    params: SubImageParams,
}
impl<'a> SubImageBuilder<'a> {
    pub(crate) fn new(img: &'a Image, size_dim: [u32; 2]) -> Self {
        Self {
            img,
            params: SubImageParams::size(size_dim),
        }
    }
    pub fn with_margin_left(mut self, margin: u32) -> Self {
        self.params.margin_left = margin;
        self
    }
    pub fn with_margin_right(mut self, margin: u32) -> Self {
        self.params.margin_right = margin;
        self
    }
    pub fn with_margin_top(mut self, margin: u32) -> Self {
        self.params.margin_top = margin;
        self
    }
    pub fn with_margin_bottom(mut self, margin: u32) -> Self {
        self.params.margin_bottom = margin;
        self
    }
    pub fn with_margin(mut self, margin: impl MarginValue) -> Self {
        let margin = margin.expand_margins();
        self.params.margin_top = margin[0];
        self.params.margin_right = margin[1];
        self.params.margin_bottom = margin[2];
        self.params.margin_left = margin[3];
        self
    }
    pub fn with_spacing_horz(mut self, space: u32) -> Self {
        self.params.spacing_horz = space;
        self
    }
    pub fn with_spacing_vert(mut self, space: u32) -> Self {
        self.params.spacing_vert = space;
        self
    }
    pub fn with_spacing(mut self, space: impl SpacingValue) -> Self {
        let space = space.expand_spacings();
        self.params.spacing_vert = space[0];
        self.params.spacing_horz = space[1];
        self
    }
    pub fn with_transform_for_3dgfx(mut self) -> Self {
        self.params.transform_for_3dgfx = true;
        self
    }
    pub fn create(self) -> Vec<Image> {
        self.img.sub_images_from(&self.params)
    }
}

#[derive(Default, Clone, Debug)]
pub struct SubImageParams {
    pub size: [u32; 2],
    pub margin_left: u32,
    pub margin_right: u32,
    pub margin_top: u32,
    pub margin_bottom: u32,
    pub spacing_horz: u32,
    pub spacing_vert: u32,
    pub transform_for_3dgfx: bool,
}

impl SubImageParams {
    pub fn size(size_dim: [u32; 2]) -> Self {
        SubImageParams {
            size: size_dim,
            ..Default::default()
        }
    }
    pub fn with_margin_left(mut self, margin: u32) -> Self {
        self.margin_left = margin;
        self
    }
    pub fn with_margin_right(mut self, margin: u32) -> Self {
        self.margin_right = margin;
        self
    }
    pub fn with_margin_top(mut self, margin: u32) -> Self {
        self.margin_top = margin;
        self
    }
    pub fn with_margin_bottom(mut self, margin: u32) -> Self {
        self.margin_bottom = margin;
        self
    }
    pub fn with_margin(mut self, margin: impl MarginValue) -> Self {
        let margin = margin.expand_margins();
        self.margin_top = margin[0];
        self.margin_right = margin[1];
        self.margin_bottom = margin[2];
        self.margin_left = margin[3];
        self
    }
    pub fn with_spacing_horz(mut self, space: u32) -> Self {
        self.spacing_horz = space;
        self
    }
    pub fn with_spacing_vert(mut self, space: u32) -> Self {
        self.spacing_vert = space;
        self
    }
    pub fn with_spacing(mut self, space: impl SpacingValue) -> Self {
        let space = space.expand_spacings();
        self.spacing_vert = space[0];
        self.spacing_horz = space[1];
        self
    }
    pub fn iter_for_dimensions<'a>(&'a self, img_dim: [u32; 2]) -> SubImageParamsIter<'a> {
        SubImageParamsIter {
            params: self,
            dimensions: img_dim,
            cur_pos: [self.margin_left, self.margin_top],
        }
    }
}

pub struct SubImageParamsIter<'a> {
    params: &'a SubImageParams,
    dimensions: [u32; 2],
    cur_pos: [u32; 2],
}

impl<'a> SubImageParamsIter<'a> {
    fn is_valid(&self) -> bool {
        let max_x = self.cur_pos[0] + self.params.size[0] + self.params.margin_right;
        let max_y = self.cur_pos[1] + self.params.size[1] + self.params.margin_bottom;
        max_x <= self.dimensions[0] && max_y <= self.dimensions[1]
    }
}

impl<'a> Iterator for SubImageParamsIter<'a> {
    type Item = [u32; 2];

    fn next(&mut self) -> Option<[u32; 2]> {
        if self.is_valid() {
            let result = self.cur_pos;
            self.cur_pos[0] += self.params.size[0] + self.params.spacing_horz;
            if !self.is_valid() {
                self.cur_pos[0] = self.params.margin_left;
                self.cur_pos[1] += self.params.size[1] + self.params.spacing_vert;
            }
            Some(result)
        } else {
            None
        }
    }
}

pub trait MarginValue {
    fn expand_margins(&self) -> [u32; 4];
}
impl MarginValue for u32 {
    fn expand_margins(&self) -> [u32; 4] {
        let m = *self;
        [m, m, m, m]
    }
}
impl MarginValue for [u32; 2] {
    fn expand_margins(&self) -> [u32; 4] {
        let [top, right] = *self;
        [top, right, top, right]
    }
}
impl MarginValue for [u32; 4] {
    fn expand_margins(&self) -> [u32; 4] {
        *self
    }
}

pub trait SpacingValue {
    fn expand_spacings(&self) -> [u32; 2];
}
impl SpacingValue for u32 {
    fn expand_spacings(&self) -> [u32; 2] {
        let s = *self;
        [s, s]
    }
}
impl SpacingValue for [u32; 2] {
    fn expand_spacings(&self) -> [u32; 2] {
        *self
    }
}
