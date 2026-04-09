<p align="center">
  <img src="https://raw.githubusercontent.com/docoxide/docoxide/main/assets/logo.svg" alt="docoxide" width="300">
</p>

# docoxide

Fast, browser-free HTML to PDF converter written in Rust.

## Install

As a library:

```sh
cargo add docoxide
```

As a CLI:

```sh
cargo install docoxide
```

## Library usage

```rust
use docoxide::Html;

fn main() -> docoxide::Result<()> {
    let pdf = Html::new("<h1>Hello</h1>").write_pdf()?;
    pdf.write_pdf("hello.pdf")?;
    Ok(())
}
```

With stylesheets:

```rust
let pdf = Html::new("<h1>Hello</h1>")
    .with_stylesheet("h1 { color: red; }")
    .write_pdf()?;
pdf.write_pdf("styled.pdf")?;
```

From a URL:

```rust
let url: url::Url = "https://example.com".parse()?;
let pdf = Html::new(url).write_pdf()?;
pdf.write_pdf("example.pdf")?;
```

With metadata:

```rust
use docoxide::{Config, Html, Metadata};

let config = Config::new()
    .with_metadata(Metadata {
        title: Some("My Document".into()),
        author: Some("Jane Doe".into()),
        ..Default::default()
    });

let pdf = Html::new("<h1>Hello</h1>")
    .with_config(&config)
    .write_pdf()?;
pdf.write_pdf("configured.pdf")?;
```

Simple one-liner:

```rust
let pdf_bytes: Vec<u8> = docoxide::convert("<h1>Hello</h1>", None);
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
