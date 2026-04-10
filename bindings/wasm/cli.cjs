#!/usr/bin/env node
const { readFileSync, writeFileSync } = require("node:fs");

const { Command, Option } = require("commander");

const { HTML, Metadata, WritePdfOptions } = require("./pkg-nodejs/docoxide_wasm.js");
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
      "PDF metadata as key=value. Can be repeated. Keys: title, author, subject, keywords, creation_date."
    ).argParser((val, prev) => (prev || []).concat(val))
  )
  .addOption(new Option("-o, --output <file>", "Output PDF file. Writes to stdout if omitted."))
  .addOption(new Option("-q, --quiet", "Suppress progress output."))
  .parse();

const opts = program.opts();

function parseMetadata(entries) {
  if (!entries || entries.length === 0) return null;
  const meta = new Metadata();
  for (const entry of entries) {
    const eq = entry.indexOf("=");
    if (eq === -1) {
      process.stderr.write(`docoxide: error: metadata must be key=value, got '${entry}'\n`);
      process.exit(1);
    }
    const key = entry.slice(0, eq).trim();
    const value = entry.slice(eq + 1);
    switch (key) {
      case "title": meta.setTitle(value); break;
      case "author": meta.setAuthor(value); break;
      case "subject": meta.setSubject(value); break;
      case "keywords": meta.addKeyword(value); break;
      case "creation_date": meta.setCreationDate(value); break;
      default:
        process.stderr.write(`docoxide: error: unknown metadata key '${key}'\n`);
        process.exit(1);
    }
  }
  return meta;
}

async function main() {
  const inputHtml = !opts.input || opts.input === "-"
    ? readFileSync(0, "utf8")
    : readFileSync(opts.input, "utf8");

  const html = new HTML(inputHtml);

  if (opts.stylesheet) {
    for (const cssFile of opts.stylesheet) {
      html.addStylesheet(readFileSync(cssFile, "utf8"));
    }
  }

  const meta = parseMetadata(opts.metadata);
  if (meta) {
    html.setMetadata(meta);
  }

  const pdf = await html.writePdf();
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
  process.stderr.write(`docoxide: error: ${err.message || err}\n`);
  process.exit(1);
});
