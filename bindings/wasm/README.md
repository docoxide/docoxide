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
const { convert } = require("docoxide");
const fs = require("node:fs");

const html = "<h1>Hello</h1>";
const pdf = convert(html, null);

fs.writeFileSync("hello.pdf", Buffer.from(pdf));
```

With CSS:

```javascript
const pdf = convert(html, "h1 { color: red; }");
```

The function returns a `Uint8Array` of PDF bytes.

## CLI usage

```sh
npm install -g docoxide
docoxide --input page.html --output page.pdf
docoxide --input page.html --css style.css --output page.pdf
```

Reads from stdin if `--input` is omitted or set to `-`:

```sh
echo '<h1>Hello</h1>' | docoxide --output hello.pdf
```
