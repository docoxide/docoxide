<p align="center">
  <img src="https://raw.githubusercontent.com/docoxide/docoxide/main/assets/logo.svg" alt="docoxide" width="300">
</p>

# docoxide

Python bindings for [docoxide](https://crates.io/crates/docoxide), a fast, browser-free HTML to PDF converter. A drop-in replacement for weasyprint.

## Install

```sh
pip install docoxide
```

Requires Python 3.9 or newer.

## Library usage

```python
from docoxide import HTML, CSS, Metadata

# From a string
HTML(string="<h1>Hello</h1>").write_pdf("hello.pdf")

# From a file
HTML(filename="page.html").write_pdf("page.pdf")

# From a URL
HTML(url="https://example.com").write_pdf("example.pdf")
```

With stylesheets:

```python
HTML(string="<h1>Hello</h1>").write_pdf(
    "styled.pdf",
    stylesheets=[CSS(string="h1 { color: red; }")]
)
```

Stylesheets also accept plain strings:

```python
HTML(string="<h1>Hello</h1>").write_pdf(
    stylesheets=["h1 { color: red; }", "body { margin: 1in; }"]
)
```

Get the PDF object (use `bytes(pdf)` or `pdf.as_bytes()` for raw bytes):

```python
pdf = HTML(string="<h1>Hello</h1>").write_pdf()
print(pdf.page_count)
```

Write to a file object:

```python
with open("output.pdf", "wb") as f:
    HTML(string="<h1>Hello</h1>").write_pdf(f)
```

Metadata:

```python
html = HTML(string="<h1>Hello</h1>")
html.set_metadata(Metadata(title="My Document", author="Jane Doe"))
html.write_pdf("configured.pdf")
```

## CLI usage

```sh
docoxide --input page.html --output page.pdf
docoxide --input page.html --stylesheet style.css --output page.pdf
```

Reads from stdin if `--input` is omitted or set to `-`:

```sh
echo '<h1>Hello</h1>' | docoxide --output hello.pdf
```
