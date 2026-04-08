use pyo3::prelude::*;

/// Convert an HTML document (with optional CSS) into PDF bytes.
#[pyfunction]
#[pyo3(signature = (html, css=None))]
fn convert(html: &str, css: Option<&str>) -> Vec<u8> {
    ::docoxide::convert(html, css)
}

#[pymodule]
fn docoxide(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(convert, m)?)?;
    Ok(())
}
