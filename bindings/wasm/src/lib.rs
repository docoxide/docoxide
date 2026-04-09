use wasm_bindgen::prelude::*;

/// PDF document metadata.
#[wasm_bindgen]
#[derive(Default, Clone)]
pub struct Metadata {
    title: Option<String>,
    author: Option<String>,
    subject: Option<String>,
    keywords: Vec<String>,
    creation_date: Option<String>,
}

#[wasm_bindgen]
impl Metadata {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Metadata {
        Metadata::default()
    }

    #[wasm_bindgen(js_name = "setTitle")]
    pub fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_owned());
    }

    #[wasm_bindgen(js_name = "setAuthor")]
    pub fn set_author(&mut self, author: &str) {
        self.author = Some(author.to_owned());
    }

    #[wasm_bindgen(js_name = "setSubject")]
    pub fn set_subject(&mut self, subject: &str) {
        self.subject = Some(subject.to_owned());
    }

    #[wasm_bindgen(js_name = "addKeyword")]
    pub fn add_keyword(&mut self, keyword: &str) {
        self.keywords.push(keyword.to_owned());
    }

    #[wasm_bindgen(js_name = "setCreationDate")]
    pub fn set_creation_date(&mut self, date: &str) {
        self.creation_date = Some(date.to_owned());
    }
}

impl From<&Metadata> for docoxide::Metadata {
    fn from(m: &Metadata) -> Self {
        docoxide::Metadata {
            title: m.title.clone(),
            author: m.author.clone(),
            subject: m.subject.clone(),
            keywords: m.keywords.clone(),
            creation_date: m.creation_date.clone(),
        }
    }
}

/// An HTML document that can be converted to PDF.
///
/// Create from an HTML string or a URL, then call writePdf().
#[wasm_bindgen]
pub struct HTML {
    source: docoxide::HtmlSource,
    stylesheets: Vec<String>,
    metadata: Option<Metadata>,
}

#[wasm_bindgen]
impl HTML {
    /// Creates an HTML document from a string.
    #[wasm_bindgen(constructor)]
    pub fn new(html: &str) -> HTML {
        HTML {
            source: docoxide::HtmlSource::String(html.to_owned()),
            stylesheets: Vec::new(),
            metadata: None,
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
            stylesheets: Vec::new(),
            metadata: None,
        })
    }

    /// Adds a CSS stylesheet string.
    #[wasm_bindgen(js_name = "addStylesheet")]
    pub fn add_stylesheet(&mut self, css: &str) {
        self.stylesheets.push(css.to_owned());
    }

    /// Sets the PDF metadata.
    #[wasm_bindgen(js_name = "setMetadata")]
    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = Some(metadata);
    }

    /// Renders the document to PDF and returns a PDF object.
    ///
    /// Pass a WritePdfOptions to apply additional stylesheets.
    #[wasm_bindgen(js_name = "writePdf")]
    pub async fn write_pdf(&self, options: Option<WritePdfOptions>) -> PDF {
        let mut html = docoxide::Html::new(self.source.clone());

        for css in &self.stylesheets {
            html = html.with_stylesheet(css.as_str());
        }
        if let Some(opts) = options {
            if let Some(css) = opts.stylesheets_combined() {
                html = html.with_stylesheet(css.as_str());
            }
        }

        if let Some(meta) = &self.metadata {
            let config = docoxide::Config::new().with_metadata(meta.into());
            html = html.with_config(&config);
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
