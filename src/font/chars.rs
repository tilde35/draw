use std;
use unicode_normalization::{Recompositions, UnicodeNormalization};

pub trait RenderableCharacters: Copy {
    type IterType: Iterator<Item = char>;

    fn len_hint(&self) -> usize;
    fn chars_iter(&self) -> Self::IterType;
}

impl<'a> RenderableCharacters for &'a str {
    type IterType = Recompositions<::std::str::Chars<'a>>;

    fn len_hint(&self) -> usize {
        self.len()
    }
    fn chars_iter(&self) -> Self::IterType {
        self.nfc()
    }
}

impl<'a> RenderableCharacters for &'a [char] {
    type IterType = SliceCharIter<'a>;

    fn len_hint(&self) -> usize {
        self.len()
    }
    fn chars_iter(&self) -> Self::IterType {
        SliceCharIter { iter: self.iter() }
    }
}

pub struct SliceCharIter<'a> {
    iter: std::slice::Iter<'a, char>,
}
impl<'a> Iterator for SliceCharIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.iter.next().map(|c| *c)
    }
}
