pub mod config;
pub mod error;
pub mod fonts;
pub mod html;
pub mod painter;
pub mod pdf;
pub mod pipeline;
pub mod source;
pub mod types;

pub use config::Config;
pub use error::{Error, Result};
pub use html::Html;
pub use pdf::Pdf;
pub use source::{FontSource, HtmlSource, StylesheetSource};
pub use types::Metadata;

/// Convert an HTML document into PDF bytes.
///
/// This is a convenience wrapper around [`Html::new`] and [`Html::write_pdf`].
#[cfg(not(target_arch = "wasm32"))]
pub fn convert(html: &str, css: Option<&str>) -> Vec<u8> {
    pipeline::run_simple(html, css).unwrap_or_default()
}

/// Convert an HTML document into PDF bytes (async for WASM).
#[cfg(target_arch = "wasm32")]
pub async fn convert(html: &str, css: Option<&str>) -> Vec<u8> {
    pipeline::run_simple(html, css).await.unwrap_or_default()
}
