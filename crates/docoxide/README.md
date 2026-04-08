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
use docoxide::convert;

fn main() {
    let html = "<h1>Hello</h1>";
    let pdf: Vec<u8> = convert(html, None);
    std::fs::write("hello.pdf", pdf).unwrap();
}
```

With CSS:

```rust
let pdf = docoxide::convert(html, Some("h1 { color: red; }"));
```

## CLI usage

```sh
docoxide --input page.html --output page.pdf
docoxide --input page.html --css style.css --output page.pdf
```

Reads from stdin if `--input` is omitted or set to `-`:

```sh
echo '<h1>Hello</h1>' | docoxide --output hello.pdf
cat page.html | docoxide --css style.css --output page.pdf
```
