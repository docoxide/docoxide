//! `docoxide` is an HTML to PDF conversion library.
//!
//! [`convert`] takes an HTML document and an optional CSS stylesheet and
//! returns the rendered PDF as a byte vector.
//!
//! # Example
//!
//! ```
//! let html = "<h1>Hello</h1>";
//! let pdf = docoxide::convert(html, None);
//! assert!(pdf.is_empty()); // stub implementation
//! ```

/// Convert an HTML document into PDF bytes.
///
/// # Arguments
///
/// * `html` - The HTML source to render.
/// * `css` - Optional CSS stylesheet to apply.
///
/// # Returns
///
/// A `Vec<u8>` containing the rendered PDF document.
pub fn convert(_html: &str, _css: Option<&str>) -> Vec<u8> {
    Vec::new()
}
