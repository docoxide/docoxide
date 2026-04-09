<p align="center">
  <img src="assets/logo.svg" alt="docoxide" width="300">
</p>

# docoxide

Fast, browser-free HTML to PDF converter written in Rust with Python and WASM bindings.

## Packages

| Package | Language | Install | Docs |
|---|---|---|---|
| `docoxide` | Rust (lib + CLI) | `cargo add docoxide` / `cargo install docoxide` | [crates/docoxide](crates/docoxide/README.md) |
| `docoxide` (Python) | Python | `pip install docoxide` | [bindings/python](bindings/python/README.md) |
| `docoxide` (npm) | JavaScript / Node | `npm install docoxide` | [bindings/wasm](bindings/wasm/README.md) |

## Quick start

**Rust:**

```rust
let pdf = docoxide::Html::new("<h1>Hello</h1>").write_pdf()?;
pdf.write_pdf("hello.pdf")?;
```

**Python** (weasyprint-compatible):

```python
from docoxide import HTML
HTML(string="<h1>Hello</h1>").write_pdf("hello.pdf")
```

**JavaScript / Node:**

```javascript
const { HTML } = require("docoxide");

(async () => {
  const pdf = await new HTML("<h1>Hello</h1>").writePdf();
  require("fs").writeFileSync("hello.pdf", Buffer.from(pdf.asBytes()));
})();
```

**CLI** (all three packages ship the same CLI):

```sh
docoxide --input page.html --output page.pdf
docoxide --input page.html --stylesheet style.css --output page.pdf
echo '<h1>Hello</h1>' | docoxide --output hello.pdf
```

## API

| Feature | Rust | Python | WASM (JS/TS) |
|---|---|---|---|
| Create from string | `Html::new("...")` | `HTML(string="...")` | `new HTML("...")` |
| Create from URL | `Html::new(url)` | `HTML(url="...")` | `HTML.fromUrl("...")` |
| Create from file | `Html::new(Url::from_file_path(...))` | `HTML(filename="...")` | |
| Create from reader | `Html::from_reader(r)` | `HTML(file_obj=f)` | |
| Add stylesheet (string) | `.with_stylesheet("...")` | `html.add_stylesheet("...")` | `html.addStylesheet("...")` |
| Add stylesheet (CSS/file) | `.with_stylesheet(path)` | `html.add_stylesheet(CSS(...))` | |
| Stylesheets at render | | `write_pdf(stylesheets=[...])` | `writePdf(opts)` |
| Base URL | `.with_base_url(url)` | `HTML(base_url="...")` | |
| Add font | `config.with_font(path)` | `html.add_font(filename="...")` | |
| Set metadata | `config.with_metadata(...)` | `html.set_metadata(...)` | `html.setMetadata(meta)` |
| Render to PDF | `html.write_pdf()` | `html.write_pdf()` | `await html.writePdf()` |
| Save to file | `pdf.write_pdf("out.pdf")` | `html.write_pdf("out.pdf")` | |
| Save to writer | `pdf.write_to(writer)` | `html.write_pdf(file_obj)` | |
| Get bytes | `pdf.as_bytes()` | `pdf.as_bytes()` / `bytes(pdf)` | `pdf.asBytes()` |
| Page count | `pdf.page_count()` | `pdf.page_count` | `pdf.pageCount` |
| Simple convert | `docoxide::convert(html, css)` | `docoxide.convert(html, css)` | `await convert(html, css)` |
