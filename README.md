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

All three packages expose the same `convert(html, css)` function and ship a `docoxide` CLI with the same flags.

## Quick start

```sh
docoxide --input page.html --output page.pdf
```

Add `--css style.css` to apply a stylesheet. The CLI also accepts HTML from
stdin if `--input` is omitted, which is useful for piping from other tools.
