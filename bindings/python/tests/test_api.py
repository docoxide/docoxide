import io
import tempfile
from pathlib import Path

import pytest
from docoxide import CSS, HTML, Metadata, convert


def test_html_string_source():
    pdf = HTML(string="<h1>Hello</h1>").write_pdf()
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_html_filename_source():
    with tempfile.NamedTemporaryFile(mode="w", suffix=".html", delete=False) as f:
        f.write("<h1>Hello</h1>")
        f.flush()
        pdf = HTML(filename=f.name).write_pdf()
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_html_url_source():
    with tempfile.NamedTemporaryFile(mode="w", suffix=".html", delete=False) as f:
        f.write("<h1>Hello</h1>")
        f.flush()
        pdf = HTML(url=Path(f.name).as_uri()).write_pdf()
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_html_file_obj_source():
    pdf = HTML(file_obj=io.StringIO("<h1>Hello</h1>")).write_pdf()
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_html_rejects_no_source():
    with pytest.raises((TypeError, RuntimeError)):
        HTML()


def test_html_rejects_multiple_sources():
    with pytest.raises(RuntimeError):
        HTML(string="<h1>A</h1>", url="https://example.com")


def test_add_stylesheet_with_css_object():
    html = HTML(string="<h1>Hello</h1>")
    html.add_stylesheet(CSS(string="h1 { color: red; }"))
    pdf = html.write_pdf()
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_add_stylesheet_with_string():
    html = HTML(string="<h1>Hello</h1>")
    html.add_stylesheet("h1 { color: red; }")
    pdf = html.write_pdf()
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_css_file_obj():
    html = HTML(string="<h1>Hello</h1>")
    html.add_stylesheet(CSS(file_obj=io.StringIO("h1 { color: red; }")))
    pdf = html.write_pdf()
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_stylesheets_as_css_objects():
    pdf = HTML(string="<h1>Hello</h1>").write_pdf(stylesheets=[CSS(string="body { margin: 1in; }")])
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_stylesheets_as_strings():
    pdf = HTML(string="<h1>Hello</h1>").write_pdf(stylesheets=["h1 { color: red; }"])
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_stylesheets_mixed():
    pdf = HTML(string="<h1>Hello</h1>").write_pdf(
        stylesheets=[CSS(string="body { margin: 1in; }"), "h1 { color: red; }"]
    )
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_write_pdf_to_file_path():
    with tempfile.NamedTemporaryFile(suffix=".pdf", delete=False) as f:
        HTML(string="<h1>Hello</h1>").write_pdf(f.name)
        assert Path(f.name).read_bytes()[:8] == b"%PDF-1.7"


def test_write_pdf_to_file_object():
    buf = io.BytesIO()
    HTML(string="<h1>Hello</h1>").write_pdf(buf)
    assert buf.getvalue()[:8] == b"%PDF-1.7"


def test_page_count():
    pdf = HTML(string="<h1>Hello</h1>").write_pdf()
    assert pdf.page_count == 1


def test_as_bytes():
    pdf = HTML(string="<h1>Hello</h1>").write_pdf()
    assert pdf.as_bytes()[:8] == b"%PDF-1.7"


def test_add_font_from_file():
    html = HTML(string='<h1 style="font-family: NotoSans">Hello</h1>')
    html.add_font(filename="../../crates/docoxide/fonts/NotoSans-Variable.ttf")
    pdf = html.write_pdf()
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_add_font_from_bytes():
    with open("../../crates/docoxide/fonts/NotoSans-Variable.ttf", "rb") as f:
        font_bytes = f.read()
    html = HTML(string='<h1 style="font-family: NotoSans">Hello</h1>')
    html.add_font(font_bytes=font_bytes)
    pdf = html.write_pdf()
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_metadata():
    html = HTML(string="<h1>Hello</h1>")
    html.set_metadata(Metadata(title="Test", author="Author"))
    pdf = html.write_pdf()
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_base_url():
    pdf = HTML(string="<h1>Hello</h1>", base_url="https://example.com").write_pdf()
    assert bytes(pdf)[:8] == b"%PDF-1.7"


def test_convert():
    pdf = convert("<h1>Hello</h1>")
    assert pdf[:8] == b"%PDF-1.7"


def test_convert_with_css():
    pdf = convert("<h1>Hello</h1>", "h1 { color: red; }")
    assert pdf[:8] == b"%PDF-1.7"
