const { HTML, WritePdfOptions, convert } = require('../pkg-nodejs/docoxide_wasm.js');
const assert = require('assert');
const { describe, it } = require('node:test');

describe('convert()', () => {
  it('returns valid PDF bytes', async () => {
    const pdf = await convert('<h1>Hello</h1>');
    assert.ok(pdf.length > 0);
    assert.strictEqual(Buffer.from(pdf.slice(0, 8)).toString(), '%PDF-1.7');
  });

  it('accepts optional CSS', async () => {
    const pdf = await convert('<h1>Hello</h1>', 'h1 { color: red; }');
    assert.strictEqual(Buffer.from(pdf.slice(0, 8)).toString(), '%PDF-1.7');
  });
});

describe('HTML class', () => {
  it('creates from string and renders', async () => {
    const pdf = await new HTML('<h1>Hello</h1>').writePdf();
    assert.strictEqual(Buffer.from(pdf.asBytes().slice(0, 8)).toString(), '%PDF-1.7');
    assert.strictEqual(pdf.pageCount, 1);
  });

  it('accepts WritePdfOptions with stylesheets', async () => {
    const opts = new WritePdfOptions();
    opts.addStylesheet('h1 { color: red; }');
    const pdf = await new HTML('<h1>Hello</h1>').writePdf(opts);
    assert.strictEqual(Buffer.from(pdf.asBytes().slice(0, 8)).toString(), '%PDF-1.7');
  });

  it('works without options', async () => {
    const pdf = await new HTML('<h1>Hello</h1>').writePdf();
    assert.ok(pdf.asBytes().length > 0);
  });

  it('creates from URL via fromUrl()', () => {
    const html = HTML.fromUrl('https://example.com');
    assert.ok(html);
  });

  it('rejects invalid URL', () => {
    assert.throws(() => HTML.fromUrl('not a url'), /invalid URL/);
  });

  it('exposes page count', async () => {
    const pdf = await new HTML('<h1>Hello</h1>').writePdf();
    assert.strictEqual(pdf.pageCount, 1);
  });
});
