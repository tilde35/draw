use std::cmp::max;
use std::fmt;

/// Rect struct represents a rectangle. Note, that if either of the dimensions are zero, then the
/// rectangle is considered empty/non-existant.
#[derive(Clone, Copy, Eq, PartialEq, Default, Hash)]
pub struct Rect {
    pub pos: [i32; 2],
    pub dim: [u32; 2],
}
impl fmt::Debug for Rect {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "Rect[{:?} {:?}x{:?}]",
            (self.pos[0], self.pos[1]),
            self.dim[0],
            self.dim[1]
        )
    }
}
impl Rect {
    pub fn new(pos: [i32; 2], dim: [u32; 2]) -> Self {
        Self { pos, dim }
    }

    pub fn x(&self) -> i32 {
        self.pos[0]
    }
    pub fn y(&self) -> i32 {
        self.pos[1]
    }
    pub fn width(&self) -> u32 {
        self.dim[0]
    }
    pub fn height(&self) -> u32 {
        self.dim[1]
    }
    pub fn idim(&self) -> [i32; 2] {
        [self.dim[0] as i32, self.dim[1] as i32]
    }

    pub fn end_pos(&self) -> [i32; 2] {
        [self.end_x(), self.end_y()]
    }
    pub fn end_x(&self) -> i32 {
        self.pos[0] + (self.dim[0] as i32)
    }
    pub fn end_y(&self) -> i32 {
        self.pos[1] + (self.dim[1] as i32)
    }

    /// Returns true if either the X or the Y dimensions are zero.
    pub fn is_empty(&self) -> bool {
        self.dim[0] == 0 || self.dim[1] == 0
    }

    // Returns true if the specified point is contained in the rectangle. Note: An empty rectange will always return false.
    pub fn contains(&self, pt: [i32; 2]) -> bool {
        if pt[0] < self.pos[0] || pt[1] < self.pos[1] {
            false
        } else {
            if pt[0] >= self.pos[0] + (self.dim[0] as i32)
                || pt[1] >= self.pos[1] + (self.dim[1] as i32)
            {
                false
            } else {
                true
            }
        }
    }

    #[must_use]
    pub fn with_pos(&self, pos: [i32; 2]) -> Self {
        Self { pos, dim: self.dim }
    }
    #[must_use]
    pub fn with_dim(&self, dim: [u32; 2]) -> Self {
        Self { pos: self.pos, dim }
    }
    #[must_use]
    pub fn with_x(&self, x: i32) -> Self {
        Self {
            pos: [x, self.pos[1]],
            dim: self.dim,
        }
    }
    #[must_use]
    pub fn with_y(&self, y: i32) -> Self {
        Self {
            pos: [self.pos[0], y],
            dim: self.dim,
        }
    }
    #[must_use]
    pub fn with_width(&self, width: u32) -> Self {
        Self {
            pos: self.pos,
            dim: [width, self.dim[1]],
        }
    }
    #[must_use]
    pub fn with_height(&self, height: u32) -> Self {
        Self {
            pos: self.pos,
            dim: [self.dim[0], height],
        }
    }
    #[must_use]
    pub fn with_iwidth(&self, width: i32) -> Self {
        self.with_width(max(0, width) as u32)
    }
    #[must_use]
    pub fn with_iheight(&self, height: i32) -> Self {
        self.with_height(max(0, height) as u32)
    }

    #[must_use]
    pub fn with_idim(&self, dim: [i32; 2]) -> Self {
        self.with_dim(fmt_idim(dim))
    }
    #[must_use]
    pub fn with_delta_dim(&self, delta: [i32; 2]) -> Self {
        let d = self.idim();
        self.with_idim([d[0] + delta[0], d[1] + delta[1]])
    }
    #[must_use]
    pub fn with_delta_pos(&self, delta: [i32; 2]) -> Self {
        Self {
            pos: [self.pos[0] + delta[0], self.pos[1] + delta[1]],
            dim: self.dim,
        }
    }
    #[must_use]
    pub fn with_delta_x(&self, delta: i32) -> Self {
        Self {
            pos: [self.pos[0] + delta, self.pos[1]],
            dim: self.dim,
        }
    }
    #[must_use]
    pub fn with_delta_y(&self, delta: i32) -> Self {
        Self {
            pos: [self.pos[0], self.pos[1] + delta],
            dim: self.dim,
        }
    }
    #[must_use]
    pub fn with_delta_width(&self, delta: i32) -> Self {
        self.with_delta_dim([delta, 0])
    }
    #[must_use]
    pub fn with_delta_height(&self, delta: i32) -> Self {
        self.with_delta_dim([0, delta])
    }
}

fn fmt_idim(dim: [i32; 2]) -> [u32; 2] {
    [max(0, dim[0]) as u32, max(0, dim[1]) as u32]
}
