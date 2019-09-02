#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum ResizeFilter {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}
impl ResizeFilter {
    pub(crate) fn as_filter_type(&self) -> image::FilterType {
        match *self {
            ResizeFilter::Nearest => image::FilterType::Nearest,
            ResizeFilter::Triangle => image::FilterType::Triangle,
            ResizeFilter::CatmullRom => image::FilterType::CatmullRom,
            ResizeFilter::Gaussian => image::FilterType::Gaussian,
            ResizeFilter::Lanczos3 => image::FilterType::Lanczos3,
        }
    }
}
