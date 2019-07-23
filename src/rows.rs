use crate::img::Image;
use crate::rgba::Rgba;

pub struct RowsIter<'a> {
    buf: &'a [Rgba],
    cur_idx: usize,
    cur_pos: [i32; 2],
    width: usize,
    stride: usize,
    max_idx: usize,
}
impl<'a> RowsIter<'a> {
    pub fn new(img: &'a Image, loc: [u32; 2], dim: [u32; 2]) -> RowsIter<'a> {
        let [x, y] = loc;
        let [width, height] = dim;
        if x + width > img.width() {
            panic!(
                "Iterator extends beyond image (x={}) + (width={}) must not be greater than image width ({})",
                x,
                width,
                img.width()
            );
        }
        if y + height > img.height() {
            panic!(
                "Iterator extends beyond image (y={}) + (height={}) must not be greater than image height ({})",
                y,
                height,
                img.height()
            );
        }
        let idx0 = img.index_at([x, y]);
        let stride = img.stride();
        let width = width as usize;
        RowsIter {
            buf: img.buffer(),
            cur_idx: idx0,
            cur_pos: [loc[0] as i32, loc[1] as i32],
            width: width,
            stride: stride,
            max_idx: idx0 + (height as usize) * stride,
        }
    }
    pub unsafe fn unchecked_from_index(
        buf: &'a [Rgba],
        idx0: usize,
        pos0: [i32; 2],
        width: usize,
        stride: usize,
        max_idx: usize,
    ) -> RowsIter<'a> {
        RowsIter {
            buf: buf,
            cur_idx: idx0,
            cur_pos: pos0,
            width: width,
            stride: stride,
            max_idx: max_idx,
        }
    }
    pub fn with_pos(self) -> RowsPosIter<'a> {
        RowsPosIter(self)
    }
}
impl<'a> Iterator for RowsIter<'a> {
    type Item = &'a [Rgba];

    fn next(&mut self) -> Option<&'a [Rgba]> {
        if self.cur_idx <= self.max_idx {
            let result = &self.buf[self.cur_idx..(self.cur_idx + self.width)];
            self.cur_idx += self.stride;
            self.cur_pos[1] += 1;
            Some(result)
        } else {
            None
        }
    }
}

pub struct RowsPosIter<'a>(RowsIter<'a>);
impl<'a> Iterator for RowsPosIter<'a> {
    type Item = ([i32; 2], &'a [Rgba]);

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.0.cur_pos;
        if let Some(row) = self.0.next() {
            Some((pos, row))
        } else {
            None
        }
    }
}

pub struct RowsMutIter<'a> {
    buf: &'a mut [Rgba],
    cur_idx: usize,
    cur_pos: [i32; 2],
    width: usize,
    stride: usize,
    max_idx: usize,
}
impl<'a> RowsMutIter<'a> {
    pub fn new(img: &'a mut Image, loc: [u32; 2], dim: [u32; 2]) -> RowsMutIter<'a> {
        let [x, y] = loc;
        let [width, height] = dim;
        if x + width > img.width() {
            panic!(
                "Iterator extends beyond image (x={}) + (width={}) must not be greater than image width ({})",
                x,
                width,
                img.width()
            );
        }
        if y + height > img.height() {
            panic!(
                "Iterator extends beyond image (y={}) + (height={}) must not be greater than image height ({})",
                y,
                height,
                img.height()
            );
        }
        let idx0 = img.index_at([x, y]);
        let stride = img.stride();
        let width = width as usize;
        RowsMutIter {
            buf: img.buffer_mut(),
            cur_idx: idx0,
            cur_pos: [loc[0] as i32, loc[1] as i32],
            width: width,
            stride: stride,
            max_idx: idx0 + (height as usize) * stride,
        }
    }
    pub unsafe fn unchecked_from_index(
        buf: &'a mut [Rgba],
        idx0: usize,
        pos0: [i32; 2],
        width: usize,
        stride: usize,
        max_idx: usize,
    ) -> RowsMutIter<'a> {
        RowsMutIter {
            buf: buf,
            cur_idx: idx0,
            cur_pos: pos0,
            width: width,
            stride: stride,
            max_idx: max_idx,
        }
    }
    pub fn with_pos(self) -> RowsMutPosIter<'a> {
        RowsMutPosIter(self)
    }
}
impl<'a> Iterator for RowsMutIter<'a> {
    type Item = &'a mut [Rgba];

    fn next(&mut self) -> Option<&'a mut [Rgba]> {
        if self.cur_idx < self.max_idx {
            let (from_idx, to_idx) = (self.cur_idx, self.cur_idx + self.width);
            self.cur_idx += self.stride;
            unsafe {
                // Note: This is safe assuming width is less/equal to stride (verified in new method)
                let slice = &mut self.buf[from_idx..to_idx];
                let raw = slice as *mut [Rgba];
                Some(&mut *raw)
            }
        } else {
            None
        }
    }
}

pub struct RowsMutPosIter<'a>(RowsMutIter<'a>);
impl<'a> Iterator for RowsMutPosIter<'a> {
    type Item = ([i32; 2], &'a mut [Rgba]);

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.0.cur_pos;
        if let Some(row) = self.0.next() {
            Some((pos, row))
        } else {
            None
        }
    }
}
