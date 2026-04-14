#[derive(Debug, Clone, PartialEq)]
pub enum Length {
    Mm(f32),
    Cm(f32),
    In(f32),
    Px(f32),
    Pt(f32),
}

impl Length {
    pub fn to_pt(&self) -> f32 {
        match self {
            Length::Mm(v) => v * 2.8346457,
            Length::Cm(v) => v * 28.346457,
            Length::In(v) => v * 72.0,
            Length::Px(v) => v * 0.75,
            Length::Pt(v) => *v,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum PageSize {
    A3,
    #[default]
    A4,
    A5,
    Letter,
    Legal,
    Tabloid,
    Custom {
        width: Length,
        height: Length,
    },
}

impl PageSize {
    pub fn to_pts(&self) -> (f32, f32) {
        match self {
            PageSize::A3 => (841.89, 1190.55),
            PageSize::A4 => (595.28, 841.89),
            PageSize::A5 => (419.53, 595.28),
            PageSize::Letter => (612.0, 792.0),
            PageSize::Legal => (612.0, 1008.0),
            PageSize::Tabloid => (792.0, 1224.0),
            PageSize::Custom { width, height } => (width.to_pt(), height.to_pt()),
        }
    }
}

/// PDF document metadata embedded in the output file.
///
/// `creation_date` accepts "YYYY-MM-DD" or "YYYY-MM-DDTHH:MM:SS".
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Vec<String>,
    pub creation_date: Option<String>,
}

/// Page margins in PDF points (1pt = 1/72 inch).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PageMargins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}
