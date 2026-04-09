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
const { HTML, WritePdfOptions, convert } = require("docoxide");
const fs = require("node:fs");

// Using the HTML class
const html = new HTML("<h1>Hello</h1>");
const pdf = html.writePdf();
fs.writeFileSync("hello.pdf", Buffer.from(pdf));
```

With stylesheets:

```javascript
const opts = new WritePdfOptions();
opts.addStylesheet("h1 { color: red; }");
opts.addStylesheet("body { margin: 1in; }");

const pdf = new HTML("<h1>Hello</h1>").writePdf(opts);
```

Simple one-liner:

```javascript
const pdf = convert("<h1>Hello</h1>", "h1 { color: red; }");
```

The functions return a `Uint8Array` of PDF bytes.

## CLI usage

```sh
npm install -g docoxide
docoxide --input page.html --output page.pdf
docoxide --input page.html --stylesheet style.css --output page.pdf
```

Reads from stdin if `--input` is omitted or set to `-`:

```sh
echo '<h1>Hello</h1>' | docoxide --output hello.pdf
```
