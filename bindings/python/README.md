<p align="center">
  <img src="https://raw.githubusercontent.com/docoxide/docoxide/main/assets/logo.svg" alt="docoxide" width="300">
</p>

# docoxide

Python bindings for [docoxide](https://crates.io/crates/docoxide), a fast, browser-free HTML to PDF converter.

## Install

```sh
pip install docoxide
```

Requires Python 3.9 or newer.

## Library usage

```python
from docoxide import convert

html = "<h1>Hello</h1>"
pdf: bytes = convert(html)

with open("hello.pdf", "wb") as f:
    f.write(pdf)
```

With CSS:

```python
pdf = convert(html, "h1 { color: red; }")
```

## CLI usage

```sh
docoxide --input page.html --output page.pdf
docoxide --input page.html --css style.css --output page.pdf
```

Reads from stdin if `--input` is omitted or set to `-`:

```sh
echo '<h1>Hello</h1>' | docoxide --output hello.pdf
```
