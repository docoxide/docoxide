#!/usr/bin/env node
const { readFileSync, writeFileSync } = require("node:fs");

const { Command, Option } = require("commander");

const { HTML, WritePdfOptions, convert } = require("./pkg-nodejs/docoxide_wasm.js");
const { version } = require("./package.json");

const program = new Command();

program
  .name("docoxide")
  .description("Fast, browser-free HTML to PDF converter.")
  .version(version)
  .addOption(
    new Option(
      "-i, --input <file>",
      "HTML input: file path or - for stdin. Reads from stdin if omitted."
    )
  )
  .addOption(
    new Option(
      "-s, --stylesheet <file>",
      "Additional CSS stylesheet file. Can be repeated."
    ).argParser((val, prev) => (prev || []).concat(val))
  )
  .addOption(
    new Option(
      "-m, --metadata <key=value>",
      "PDF metadata as key=value. Can be repeated."
    ).argParser((val, prev) => (prev || []).concat(val))
  )
  .addOption(new Option("-o, --output <file>", "Output PDF file. Writes to stdout if omitted."))
  .addOption(new Option("-q, --quiet", "Suppress progress output."))
  .parse();

const opts = program.opts();

async function main() {
  const inputHtml = !opts.input || opts.input === "-"
    ? readFileSync(0, "utf8")
    : readFileSync(opts.input, "utf8");

  const pdfOpts = new WritePdfOptions();
  if (opts.stylesheet) {
    for (const cssFile of opts.stylesheet) {
      pdfOpts.addStylesheet(readFileSync(cssFile, "utf8"));
    }
  }

  const html = new HTML(inputHtml);
  const pdf = await html.writePdf(pdfOpts);
  const bytes = Buffer.from(pdf.asBytes());

  if (opts.output) {
    writeFileSync(opts.output, bytes);
    if (!opts.quiet) {
      process.stderr.write(`docoxide: written ${pdf.pageCount} page(s) to ${opts.output}\n`);
    }
  } else {
    process.stdout.write(bytes);
  }
}

main().catch((err) => {
  process.stderr.write(`docoxide: error: ${err.message}\n`);
  process.exit(1);
});
