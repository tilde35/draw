use crate::img::Image;
use crate::rgba::Rgba;

pub struct RowsIter<'a> {
    buf: &'a [Rgba],
    cur_idx: usize,
    width: usize,
    stride: usize,
    max_idx: usize,
}
impl<'a> RowsIter<'a> {
    pub fn new(img: &'a Image, x: u32, y: u32, dim: [u32; 2]) -> RowsIter<'a> {
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
            width: width,
            stride: stride,
            max_idx: idx0 + (height as usize) * stride,
        }
    }
    pub unsafe fn unchecked_from_index(buf: &'a [Rgba], idx0: usize, width: usize, stride: usize, max_idx: usize) -> RowsIter<'a> {
        RowsIter {
            buf: buf,
            cur_idx: idx0,
            width: width,
            stride: stride,
            max_idx: max_idx,
        }
    }
}
impl<'a> Iterator for RowsIter<'a> {
    type Item = &'a [Rgba];

    fn next(&mut self) -> Option<&'a [Rgba]> {
        if self.cur_idx <= self.max_idx {
            let result = &self.buf[self.cur_idx..(self.cur_idx + self.width)];
            self.cur_idx += self.stride;
            Some(result)
        } else {
            None
        }
    }
}

pub struct RowsMutIter<'a> {
    buf: &'a mut [Rgba],
    cur_idx: usize,
    width: usize,
    stride: usize,
    max_idx: usize,
}
impl<'a> RowsMutIter<'a> {
    pub fn new(img: &'a mut Image, x: u32, y: u32, dim: [u32; 2]) -> RowsMutIter<'a> {
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
            width: width,
            stride: stride,
            max_idx: idx0 + (height as usize) * stride,
        }
    }
    pub unsafe fn unchecked_from_index(buf: &'a mut [Rgba], idx0: usize, width: usize, stride: usize, max_idx: usize) -> RowsMutIter<'a> {
        RowsMutIter {
            buf: buf,
            cur_idx: idx0,
            width: width,
            stride: stride,
            max_idx: max_idx,
        }
    }
}
impl<'a> Iterator for RowsMutIter<'a> {
    type Item = &'a mut [Rgba];

    fn next(&mut self) -> Option<&'a mut [Rgba]> {
        if self.cur_idx < self.max_idx {
            let (from_idx, to_idx) = (self.cur_idx, self.cur_idx + self.width);
            self.cur_idx += self.stride;
            unsafe {
                // Note: This is safe assuming width less/equal to stride (verified in new method)
                let slice = &mut self.buf[from_idx..to_idx];
                let raw = slice as *mut [Rgba];
                Some(&mut *raw)
            }
        } else {
            None
        }
    }
}
