#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum VertAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum HorzAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum ScriptPosition {
    Normal,
    Superscript,
    Subscript,
}
