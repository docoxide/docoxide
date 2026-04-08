#!/usr/bin/env node
const { readFileSync, writeFileSync } = require("node:fs");

const { Command, Option } = require("commander");

const { convert } = require("./pkg-nodejs/docoxide_wasm.js");
const { version } = require("./package.json");

const program = new Command();

program
  .name("docoxide")
  .description("Convert HTML to PDF.")
  .version(version)
  .addOption(
    new Option("-i, --input <file>", "Path to the input HTML file. Reads from stdin if omitted or set to '-'."),
  )
  .addOption(new Option("-c, --css <file>", "Optional path to a CSS file."))
  .requiredOption("-o, --output <file>", "Path to the output PDF file.")
  .parse();

const opts = program.opts();

const html = !opts.input || opts.input === "-"
  ? readFileSync(0, "utf8")
  : readFileSync(opts.input, "utf8");
const css = opts.css ? readFileSync(opts.css, "utf8") : null;

writeFileSync(opts.output, Buffer.from(convert(html, css)));
