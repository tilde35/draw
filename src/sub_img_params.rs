#[derive(Default, Clone, Debug)]
pub struct SubImageParams {
    pub size: [u32; 2],
    pub margin_left: u32,
    pub margin_right: u32,
    pub margin_top: u32,
    pub margin_bottom: u32,
    pub spacing_horz: u32,
    pub spacing_vert: u32,
}

impl SubImageParams {
    pub fn size(sub_img_dim: [u32; 2]) -> SubImageParams {
        SubImageParams {
            size: sub_img_dim,
            ..Default::default()
        }
    }
    pub fn with_margin_left(mut self, margin: u32) -> SubImageParams {
        self.margin_left = margin;
        self
    }
    pub fn with_margin_right(mut self, margin: u32) -> SubImageParams {
        self.margin_right = margin;
        self
    }
    pub fn with_margin_top(mut self, margin: u32) -> SubImageParams {
        self.margin_top = margin;
        self
    }
    pub fn with_margin_bottom(mut self, margin: u32) -> SubImageParams {
        self.margin_bottom = margin;
        self
    }
    pub fn with_margin(mut self, margin_top: u32, margin_right: u32, margin_bottom: u32, margin_left: u32) -> SubImageParams {
        self.margin_top = margin_top;
        self.margin_right = margin_right;
        self.margin_bottom = margin_bottom;
        self.margin_left = margin_left;
        self
    }
    pub fn with_spacing_horz(mut self, space: u32) -> SubImageParams {
        self.spacing_horz = space;
        self
    }
    pub fn with_spacing_vert(mut self, space: u32) -> SubImageParams {
        self.spacing_vert = space;
        self
    }
    pub fn with_spacing(mut self, space_horz: u32, space_vert: u32) -> SubImageParams {
        self.spacing_horz = space_horz;
        self.spacing_vert = space_vert;
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
