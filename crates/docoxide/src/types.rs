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
