#[cfg(not(target_arch = "wasm32"))]
use crate::Result;

/// A rendered PDF document.
///
/// Created by calling [`crate::Html::write_pdf`]. You can get the raw bytes,
/// save to a file, or write to any [`std::io::Write`] destination.
pub struct Pdf {
    pub(crate) bytes: Vec<u8>,
    pub(crate) page_count: usize,
}

impl Pdf {
    /// Returns the PDF contents as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Takes ownership and returns the raw bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Returns the number of pages in the document.
    pub fn page_count(&self) -> usize {
        self.page_count
    }

    /// Writes the PDF to the given file path.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn write_pdf(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
        std::io::Write::write_all(&mut f, &self.bytes)?;
        std::io::Write::flush(&mut f)?;
        Ok(())
    }

    /// Writes the PDF to any [`std::io::Write`] destination.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn write_to(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        writer.write_all(&self.bytes)?;
        writer.flush()
    }
}
