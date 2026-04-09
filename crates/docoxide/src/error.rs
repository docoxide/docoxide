use thiserror::Error;

/// Errors that can occur during HTML to PDF conversion.
#[derive(Debug, Error)]
pub enum Error {
    #[error("PDF generation failed: {0}")]
    PdfGeneration(String),

    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("Network error: {0}")]
    Network(String),
}

pub type Result<T> = std::result::Result<T, Error>;
