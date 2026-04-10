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

function looksLikeUrl(s) {
  return s.startsWith("http://") || s.startsWith("https://");
}

function loadLocalFile(filePath, resolve, pathToFileURL) {
  const { fileURLToPath } = require("node:url");
  const content = readFileSync(filePath, "utf8");
  const baseUrl = pathToFileURL(resolve(filePath));
  const html = new HTML(content);
  html.setBaseUrl(baseUrl.href);

  // Read <link rel="stylesheet"> files and add them explicitly
  // since WASM fetch() cannot access local files
  const linkRe = /<link\b[^>]*>/gi;
  const relRe = /\brel=["']stylesheet["']/i;
  const hrefRe = /\bhref=["']([^"']+)["']/i;
  let match;
  while ((match = linkRe.exec(content)) !== null) {
    if (!relRe.test(match[0])) continue;
    const hrefMatch = match[0].match(hrefRe);
    if (hrefMatch) {
      try {
        const cssUrl = new URL(hrefMatch[1], baseUrl);
        if (cssUrl.protocol === "file:") {
          html.addStylesheet(readFileSync(fileURLToPath(cssUrl), "utf8"));
        }
      } catch (err) {
        process.stderr.write(`docoxide: warning: could not load stylesheet '${hrefMatch[1]}': ${err.message}\n`);
      }
    }
  }
  return html;
}

async function main() {
  const { resolve } = require("node:path");
  const { pathToFileURL, fileURLToPath } = require("node:url");

  let html;
  if (!opts.input || opts.input === "-") {
    html = new HTML(readFileSync(0, "utf8"));
  } else if (opts.input.startsWith("file://")) {
    const filePath = fileURLToPath(opts.input);
    html = loadLocalFile(filePath, resolve, pathToFileURL);
  } else if (looksLikeUrl(opts.input)) {
    html = HTML.fromUrl(opts.input);
  } else {
    html = loadLocalFile(opts.input, resolve, pathToFileURL);
  }

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
