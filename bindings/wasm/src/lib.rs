use wasm_bindgen::prelude::*;

/// An HTML document that can be converted to PDF.
///
/// Create from an HTML string or a URL, then call writePdf().
#[wasm_bindgen]
pub struct HTML {
    source: docoxide::HtmlSource,
}

#[wasm_bindgen]
impl HTML {
    /// Creates an HTML document from a string.
    #[wasm_bindgen(constructor)]
    pub fn new(html: &str) -> HTML {
        HTML {
            source: docoxide::HtmlSource::String(html.to_owned()),
        }
    }

    /// Creates an HTML document from a URL.
    #[wasm_bindgen(js_name = "fromUrl")]
    pub fn from_url(url: &str) -> Result<HTML, JsValue> {
        let parsed: url::Url = url
            .parse()
            .map_err(|e| JsValue::from_str(&format!("invalid URL: {e}")))?;
        Ok(HTML {
            source: docoxide::HtmlSource::Url(parsed),
        })
    }

    /// Renders the document to PDF and returns a PDF object.
    ///
    /// Pass a WritePdfOptions to apply stylesheets.
    #[wasm_bindgen(js_name = "writePdf")]
    pub async fn write_pdf(&self, options: Option<WritePdfOptions>) -> PDF {
        let css = options.and_then(|opts| opts.stylesheets_combined());
        let mut html = docoxide::Html::new(self.source.clone());
        if let Some(css) = css {
            html = html.with_stylesheet(css.as_str());
        }
        match html.write_pdf().await {
            Ok(pdf) => {
                let page_count = pdf.page_count();
                PDF {
                    bytes: pdf.into_bytes(),
                    page_count,
                }
            }
            Err(_) => PDF {
                bytes: Vec::new(),
                page_count: 0,
            },
        }
    }
}

/// A rendered PDF document.
#[wasm_bindgen]
pub struct PDF {
    bytes: Vec<u8>,
    page_count: usize,
}

#[wasm_bindgen]
impl PDF {
    /// Returns the PDF as bytes.
    #[wasm_bindgen(js_name = "asBytes")]
    pub fn as_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    /// Returns the number of pages.
    #[wasm_bindgen(getter, js_name = "pageCount")]
    pub fn page_count(&self) -> usize {
        self.page_count
    }
}

/// Options for writePdf(). Use addStylesheet() to attach CSS.
#[wasm_bindgen]
#[derive(Default)]
pub struct WritePdfOptions {
    stylesheets: Vec<String>,
}

#[wasm_bindgen]
impl WritePdfOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WritePdfOptions {
        WritePdfOptions {
            stylesheets: Vec::new(),
        }
    }

    /// Adds a CSS stylesheet string.
    #[wasm_bindgen(js_name = "addStylesheet")]
    pub fn add_stylesheet(&mut self, css: &str) {
        self.stylesheets.push(css.to_owned());
    }
}

impl WritePdfOptions {
    fn stylesheets_combined(&self) -> Option<String> {
        if self.stylesheets.is_empty() {
            None
        } else {
            Some(self.stylesheets.join("\n"))
        }
    }
}

/// Converts an HTML string (with optional CSS) to PDF bytes.
#[wasm_bindgen]
pub async fn convert(html: &str, css: Option<String>) -> Vec<u8> {
    docoxide::convert(html, css.as_deref()).await
}
