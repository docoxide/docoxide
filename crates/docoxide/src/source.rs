use std::sync::Arc;

/// The source of an HTML document.
#[derive(Clone)]
pub enum HtmlSource {
    String(String),
    Url(url::Url),
}

impl From<&str> for HtmlSource {
    fn from(s: &str) -> Self {
        HtmlSource::String(s.to_owned())
    }
}

impl From<String> for HtmlSource {
    fn from(s: String) -> Self {
        HtmlSource::String(s)
    }
}

impl From<url::Url> for HtmlSource {
    fn from(u: url::Url) -> Self {
        HtmlSource::Url(u)
    }
}

/// The source of a CSS stylesheet.
#[derive(Clone)]
pub enum StylesheetSource {
    String(String),
    #[cfg(not(target_arch = "wasm32"))]
    File(std::path::PathBuf),
}

impl From<&str> for StylesheetSource {
    fn from(s: &str) -> Self {
        StylesheetSource::String(s.to_owned())
    }
}

impl From<String> for StylesheetSource {
    fn from(s: String) -> Self {
        StylesheetSource::String(s)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<&std::path::Path> for StylesheetSource {
    fn from(p: &std::path::Path) -> Self {
        StylesheetSource::File(p.to_owned())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<std::path::PathBuf> for StylesheetSource {
    fn from(p: std::path::PathBuf) -> Self {
        StylesheetSource::File(p)
    }
}

/// The source of a custom font.
pub enum FontSource {
    Bytes(Arc<[u8]>),
    #[cfg(not(target_arch = "wasm32"))]
    File(std::path::PathBuf),
}

impl Clone for FontSource {
    fn clone(&self) -> Self {
        match self {
            FontSource::Bytes(arc) => FontSource::Bytes(arc.clone()),
            #[cfg(not(target_arch = "wasm32"))]
            FontSource::File(p) => FontSource::File(p.clone()),
        }
    }
}

impl From<&[u8]> for FontSource {
    fn from(b: &[u8]) -> Self {
        FontSource::Bytes(Arc::from(b))
    }
}

impl From<Vec<u8>> for FontSource {
    fn from(v: Vec<u8>) -> Self {
        FontSource::Bytes(Arc::from(v.as_slice()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<&std::path::Path> for FontSource {
    fn from(p: &std::path::Path) -> Self {
        FontSource::File(p.to_owned())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<std::path::PathBuf> for FontSource {
    fn from(p: std::path::PathBuf) -> Self {
        FontSource::File(p)
    }
}
