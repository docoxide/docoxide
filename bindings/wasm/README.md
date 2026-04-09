<p align="center">
  <img src="https://raw.githubusercontent.com/docoxide/docoxide/main/assets/logo.svg" alt="docoxide" width="300">
</p>

# docoxide

WebAssembly bindings for [docoxide](https://crates.io/crates/docoxide), a fast, browser-free HTML to PDF converter.

## Install

```sh
npm install docoxide
```

## Library usage

```javascript
const { HTML, Metadata, WritePdfOptions, convert } = require("docoxide");
const fs = require("node:fs");

// From a string
const pdf = await new HTML("<h1>Hello</h1>").writePdf();
fs.writeFileSync("hello.pdf", Buffer.from(pdf.asBytes()));
console.log(`${pdf.pageCount} page(s)`);
```

From a URL:

```javascript
const html = HTML.fromUrl("https://example.com");
const pdf = await html.writePdf();
```

With stylesheets:

```javascript
const html = new HTML("<h1>Hello</h1>");
html.addStylesheet("h1 { color: red; }");
html.addStylesheet("body { margin: 1in; }");
const pdf = await html.writePdf();
```

With metadata:

```javascript
const html = new HTML("<h1>Hello</h1>");
const meta = new Metadata();
meta.setTitle("My Document");
meta.setAuthor("Jane Doe");
html.setMetadata(meta);
const pdf = await html.writePdf();
```

Simple one-liner:

```javascript
const pdfBytes = await convert("<h1>Hello</h1>", "h1 { color: red; }");
```

## CLI usage

```sh
npm install -g docoxide
docoxide --input page.html --output page.pdf
docoxide --input page.html --stylesheet style.css --output page.pdf
docoxide --input page.html --metadata title="My Doc" --output page.pdf
```

Reads from stdin if `--input` is omitted or set to `-`:

```sh
echo '<h1>Hello</h1>' | docoxide --output hello.pdf
```