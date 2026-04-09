use crate::source::FontSource;
use crate::types::Metadata;

/// Conversion options: PDF metadata and custom fonts.
///
/// Use the builder methods to configure, then pass to [`crate::Html::with_config`].
#[derive(Clone, Default)]
pub struct Config {
    pub(crate) fonts: Vec<FontSource>,
    pub(crate) metadata: Metadata,
}

impl Config {
    /// Creates a default config (no metadata, built-in fonts).
    pub fn new() -> Self {
        Config::default()
    }

    /// Adds a custom font. Accepts a file path or raw bytes.
    pub fn with_font(mut self, font: impl Into<FontSource>) -> Self {
        self.fonts.push(font.into());
        self
    }

    /// Sets the PDF document metadata (title, author, subject, keywords, creation_date).
    pub fn with_metadata(mut self, meta: Metadata) -> Self {
        self.metadata = meta;
        self
    }
}
