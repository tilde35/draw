#[derive(Clone)]
pub struct RgbaParseError {
    kind: RgbaParseErrorKind,
}
#[derive(Clone)]
enum RgbaParseErrorKind {
    HexParse,
    Unrecognized,
}
impl RgbaParseError {
    pub(crate) fn hex_parse(_s: &str) -> Self {
        Self {
            kind: RgbaParseErrorKind::HexParse,
        }
    }
    pub(crate) fn unrecognized(_s: &str) -> Self {
        Self {
            kind: RgbaParseErrorKind::Unrecognized,
        }
    }
}
impl std::fmt::Display for RgbaParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.kind {
            RgbaParseErrorKind::HexParse => {
                write!(f, "Error when parsing color as a hexadecimal number")
            }
            RgbaParseErrorKind::Unrecognized => write!(f, "Unrecognized color option"),
        }
    }
}
impl std::fmt::Debug for RgbaParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for RgbaParseError {}

#[derive(Debug)]
pub struct ImageLoadError(image::ImageError);
impl std::fmt::Display for ImageLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ImageLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}
impl From<image::ImageError> for ImageLoadError {
    fn from(v: image::ImageError) -> Self {
        ImageLoadError(v)
    }
}

#[derive(Debug)]
pub enum FontLoadError {
    IoError(std::io::Error),
    FontError(rusttype::Error),
    SvgError(nsvg::Error),
}
impl From<std::io::Error> for FontLoadError {
    fn from(e: std::io::Error) -> Self {
        FontLoadError::IoError(e)
    }
}
impl From<rusttype::Error> for FontLoadError {
    fn from(e: rusttype::Error) -> Self {
        FontLoadError::FontError(e)
    }
}
impl From<nsvg::Error> for FontLoadError {
    fn from(e: nsvg::Error) -> Self {
        FontLoadError::SvgError(e)
    }
}
impl std::fmt::Display for FontLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FontLoadError::IoError(e) => write!(f, "{}", e),
            FontLoadError::FontError(e) => write!(f, "{}", e),
            FontLoadError::SvgError(e) => write!(f, "{}", e),
        }
    }
}
impl std::error::Error for FontLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FontLoadError::IoError(e) => Some(e),
            FontLoadError::FontError(e) => Some(e),
            FontLoadError::SvgError(e) => Some(e),
        }
    }
}
