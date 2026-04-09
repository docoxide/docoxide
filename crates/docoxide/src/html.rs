use crate::Result;
use crate::config::Config;
use crate::pdf::Pdf;
use crate::pipeline;
use crate::source::{HtmlSource, StylesheetSource};

/// An HTML document ready to be converted to PDF.
///
/// Build one with [`Html::new`], chain options, then call [`Html::write_pdf`].
pub struct Html {
    pub(crate) source: HtmlSource,
    pub(crate) stylesheets: Vec<StylesheetSource>,
    pub(crate) base_url: Option<url::Url>,
    pub(crate) config: Option<Config>,
}

impl Html {
    /// Creates an [`Html`] document from any [`std::io::Read`] source.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_reader(mut reader: impl std::io::Read) -> std::io::Result<Self> {
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        Ok(Html::new(buf))
    }

    /// Creates an [`Html`] document from a string, [`String`], or [`url::Url`].
    pub fn new(source: impl Into<HtmlSource>) -> Self {
        Html {
            source: source.into(),
            stylesheets: Vec::new(),
            base_url: None,
            config: None,
        }
    }

    /// Attaches a CSS stylesheet. Can be called multiple times to add several sheets.
    pub fn with_stylesheet(mut self, css: impl Into<StylesheetSource>) -> Self {
        self.stylesheets.push(css.into());
        self
    }

    /// Sets the base URL used to resolve relative links and images.
    pub fn with_base_url(mut self, url: url::Url) -> Self {
        self.base_url = Some(url);
        self
    }

    /// Applies a [`Config`] (metadata, custom fonts).
    pub fn with_config(mut self, config: &Config) -> Self {
        self.config = Some(config.clone());
        self
    }

    /// Renders the document to a [`Pdf`].
    #[cfg(not(target_arch = "wasm32"))]
    pub fn write_pdf(self) -> Result<Pdf> {
        pipeline::run(self)
    }

    /// Renders the document to a [`Pdf`] (async for WASM).
    #[cfg(target_arch = "wasm32")]
    pub async fn write_pdf(self) -> Result<Pdf> {
        pipeline::run(self).await
    }
}
