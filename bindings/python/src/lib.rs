use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

/// PDF document metadata (title, author, subject, keywords, creation_date).
///
/// creation_date accepts "YYYY-MM-DD" or "YYYY-MM-DDTHH:MM:SS".
#[pyclass(from_py_object)]
#[derive(Default, Clone)]
pub struct Metadata {
    title: Option<String>,
    author: Option<String>,
    subject: Option<String>,
    keywords: Vec<String>,
    creation_date: Option<String>,
}

#[pymethods]
impl Metadata {
    #[new]
    #[pyo3(signature = (*, title=None, author=None, subject=None, keywords=None, creation_date=None))]
    fn new(
        title: Option<String>,
        author: Option<String>,
        subject: Option<String>,
        keywords: Option<Vec<String>>,
        creation_date: Option<String>,
    ) -> Self {
        Metadata {
            title,
            author,
            subject,
            keywords: keywords.unwrap_or_default(),
            creation_date,
        }
    }
}

impl From<&Metadata> for ::docoxide::Metadata {
    fn from(m: &Metadata) -> Self {
        ::docoxide::Metadata {
            title: m.title.clone(),
            author: m.author.clone(),
            subject: m.subject.clone(),
            keywords: m.keywords.clone(),
            creation_date: m.creation_date.clone(),
        }
    }
}

/// A CSS stylesheet that can be applied during PDF conversion.
///
/// Create from a string, a file path, or a file-like object.
#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct CSS {
    source: ::docoxide::StylesheetSource,
}

#[pymethods]
impl CSS {
    #[new]
    #[pyo3(signature = (*, string=None, filename=None, file_obj=None))]
    fn new(string: Option<String>, filename: Option<String>, file_obj: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        let source_count = [string.is_some(), filename.is_some(), file_obj.is_some()]
            .iter()
            .filter(|&&x| x)
            .count();
        if source_count != 1 {
            return Err(PyRuntimeError::new_err(
                "Exactly one of 'string', 'filename', or 'file_obj' must be provided",
            ));
        }

        let source = if let Some(s) = string {
            ::docoxide::StylesheetSource::String(s)
        } else if let Some(fname) = filename {
            let abs = std::fs::canonicalize(&fname)
                .map_err(|e| PyRuntimeError::new_err(format!("Could not resolve path '{fname}': {e}")))?;
            ::docoxide::StylesheetSource::File(abs)
        } else if let Some(obj) = file_obj {
            let content: String = obj.call_method0("read")?.extract()?;
            ::docoxide::StylesheetSource::String(content)
        } else {
            unreachable!()
        };

        Ok(CSS { source })
    }
}

/// An HTML document that can be converted to PDF.
///
/// Create from a string, URL, file path, or file-like object, then call
/// write_pdf() to render it.
#[pyclass(subclass)]
pub struct HTML {
    html_string: Option<String>,
    url: Option<String>,
    stylesheets: Vec<::docoxide::StylesheetSource>,
    fonts: Vec<::docoxide::FontSource>,
    base_url: Option<String>,
    metadata: Metadata,
}

#[pymethods]
impl HTML {
    #[new]
    #[pyo3(signature = (*, string=None, url=None, filename=None, file_obj=None, base_url=None))]
    fn new(
        string: Option<String>,
        url: Option<String>,
        filename: Option<String>,
        file_obj: Option<&Bound<'_, PyAny>>,
        base_url: Option<String>,
    ) -> PyResult<Self> {
        let source_count = [string.is_some(), url.is_some(), filename.is_some(), file_obj.is_some()]
            .iter()
            .filter(|&&x| x)
            .count();
        if source_count != 1 {
            return Err(PyRuntimeError::new_err(
                "Exactly one of 'string', 'url', 'filename', or 'file_obj' must be provided",
            ));
        }

        let (resolved_string, resolved_url) = if let Some(s) = string {
            (Some(s), None)
        } else if let Some(u) = url {
            (None, Some(u))
        } else if let Some(fname) = filename {
            let abs = std::fs::canonicalize(&fname)
                .map_err(|e| PyRuntimeError::new_err(format!("Could not resolve path '{fname}': {e}")))?;
            let file_url = url::Url::from_file_path(&abs)
                .map_err(|_| PyRuntimeError::new_err(format!("Could not convert path to file URL: {fname}")))?;
            (None, Some(file_url.to_string()))
        } else if let Some(obj) = file_obj {
            let content: String = obj.call_method0("read")?.extract()?;
            (Some(content), None)
        } else {
            unreachable!()
        };

        Ok(HTML {
            html_string: resolved_string,
            url: resolved_url,
            stylesheets: Vec::new(),
            fonts: Vec::new(),
            base_url,
            metadata: Metadata::default(),
        })
    }

    /// Attaches a CSS stylesheet. Accepts a CSS object or a plain CSS string.
    fn add_stylesheet(&mut self, css: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Ok(css_obj) = css.extract::<CSS>() {
            self.stylesheets.push(css_obj.source.clone());
        } else if let Ok(s) = css.extract::<String>() {
            self.stylesheets.push(::docoxide::StylesheetSource::String(s));
        } else {
            return Err(PyRuntimeError::new_err(
                "add_stylesheet accepts a CSS object or a string",
            ));
        }
        Ok(())
    }

    /// Sets the PDF metadata (title, author, etc.).
    fn set_metadata(&mut self, meta: &Metadata) {
        self.metadata = meta.clone();
    }

    /// Adds a custom font from a file path or raw bytes.
    #[pyo3(signature = (*, filename=None, font_bytes=None))]
    fn add_font(&mut self, filename: Option<String>, font_bytes: Option<Vec<u8>>) -> PyResult<()> {
        match (filename, font_bytes) {
            (Some(f), None) => {
                let abs = std::fs::canonicalize(&f)
                    .map_err(|e| PyRuntimeError::new_err(format!("Could not resolve font path '{f}': {e}")))?;
                self.fonts.push(::docoxide::FontSource::File(abs));
            }
            (None, Some(b)) => {
                self.fonts.push(::docoxide::FontSource::from(b));
            }
            _ => {
                return Err(PyRuntimeError::new_err(
                    "Exactly one of 'filename' or 'font_bytes' must be provided",
                ));
            }
        }
        Ok(())
    }

    /// Renders the document to PDF and returns a PDF object.
    ///
    /// If target is a file path (str) or file-like object, also writes there.
    /// stylesheets accepts CSS objects or plain CSS strings.
    #[pyo3(signature = (target=None, *, stylesheets=None))]
    fn write_pdf(
        &self,
        target: Option<&Bound<'_, PyAny>>,
        stylesheets: Option<Vec<Bound<'_, PyAny>>>,
    ) -> PyResult<PDF> {
        use ::docoxide::{Config, Html, HtmlSource};

        let mut config = Config::new().with_metadata((&self.metadata).into());
        for font in &self.fonts {
            config = config.with_font(font.clone());
        }

        let source: HtmlSource = match (&self.html_string, &self.url) {
            (Some(s), None) => HtmlSource::String(s.clone()),
            (None, Some(u)) => {
                let parsed = url::Url::parse(u).map_err(|e| PyRuntimeError::new_err(format!("Invalid URL: {e}")))?;
                parsed.into()
            }
            _ => unreachable!(),
        };

        let mut h = Html::new(source).with_config(&config);
        for css in &self.stylesheets {
            h = h.with_stylesheet(css.clone());
        }
        if let Some(sheets) = &stylesheets {
            for item in sheets {
                if let Ok(css_obj) = item.extract::<CSS>() {
                    h = h.with_stylesheet(css_obj.source.clone());
                } else if let Ok(s) = item.extract::<String>() {
                    h = h.with_stylesheet(s.as_str());
                } else {
                    return Err(PyRuntimeError::new_err(
                        "stylesheets must contain CSS objects or strings",
                    ));
                }
            }
        }
        if let Some(base) = &self.base_url {
            match url::Url::parse(base) {
                Ok(parsed) => h = h.with_base_url(parsed),
                Err(e) => return Err(PyRuntimeError::new_err(format!("Invalid base URL '{base}': {e}"))),
            }
        }

        let pdf = h.write_pdf().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let page_count = pdf.page_count();
        let result = PDF {
            bytes: pdf.into_bytes(),
            page_count,
        };

        match target {
            None => Ok(result),
            Some(t) if t.is_instance_of::<pyo3::types::PyString>() => {
                let path: String = t.extract()?;
                std::fs::write(&path, &result.bytes).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
                Ok(result)
            }
            Some(t) => {
                t.call_method1("write", (result.bytes.clone(),))?;
                Ok(result)
            }
        }
    }
}

/// A rendered PDF document.
#[pyclass(name = "PDF")]
pub struct PDF {
    bytes: Vec<u8>,
    page_count: usize,
}

#[pymethods]
impl PDF {
    /// Returns the PDF as bytes.
    fn as_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, pyo3::types::PyBytes> {
        pyo3::types::PyBytes::new(py, &self.bytes)
    }

    /// Returns the number of pages.
    #[getter]
    fn page_count(&self) -> usize {
        self.page_count
    }

    fn __bytes__<'py>(&self, py: Python<'py>) -> Bound<'py, pyo3::types::PyBytes> {
        pyo3::types::PyBytes::new(py, &self.bytes)
    }

    fn __len__(&self) -> usize {
        self.bytes.len()
    }
}

/// Converts an HTML string (with optional CSS) to PDF bytes.
#[pyfunction]
#[pyo3(signature = (html, css=None))]
fn convert(html: &str, css: Option<&str>) -> PyResult<Vec<u8>> {
    ::docoxide::convert(html, css).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pymodule]
fn docoxide(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Metadata>()?;
    m.add_class::<CSS>()?;
    m.add_class::<HTML>()?;
    m.add_class::<PDF>()?;
    m.add_function(wrap_pyfunction!(convert, m)?)?;
    Ok(())
}
