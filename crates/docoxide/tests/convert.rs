use std::path::Path;

use docoxide::{Config, FontSource, Html, Metadata};

const SIMPLE_HTML: &str = include_str!("fixtures/simple.html");
const UNSTYLED_HTML: &str = include_str!("fixtures/unstyled.html");

const NOTO_SANS: &[u8] = include_bytes!("../fonts/NotoSans-Variable.ttf");
const NOTO_SANS_ITALIC: &[u8] = include_bytes!("../fonts/NotoSans-Italic-Variable.ttf");

const FONT_CSS: &str = "* { font-family: 'Noto Sans', sans-serif; } ul, ol { list-style-type: '- '; }";

fn base_config() -> Config {
    Config::new()
        .with_font(FontSource::from(NOTO_SANS.to_vec()))
        .with_font(FontSource::from(NOTO_SANS_ITALIC.to_vec()))
}

#[test]
fn basic_conversion() {
    let bytes = Html::new(SIMPLE_HTML)
        .with_config(&base_config())
        .with_stylesheet(FONT_CSS)
        .write_pdf()
        .expect("conversion should succeed")
        .into_bytes();
    insta::assert_binary_snapshot!(".pdf", bytes);
}

#[test]
fn stylesheet_single() {
    let bytes = Html::new(SIMPLE_HTML)
        .with_config(&base_config())
        .with_stylesheet(FONT_CSS)
        .with_stylesheet("h1 { font-size: 48px; }")
        .write_pdf()
        .expect("conversion should succeed")
        .into_bytes();
    insta::assert_binary_snapshot!(".pdf", bytes);
}

#[test]
fn stylesheet_multiple() {
    let bytes = Html::new(SIMPLE_HTML)
        .with_config(&base_config())
        .with_stylesheet(FONT_CSS)
        .with_stylesheet("h1 { font-size: 48px; }")
        .with_stylesheet("p { font-size: 24px; }")
        .write_pdf()
        .expect("conversion should succeed")
        .into_bytes();
    insta::assert_binary_snapshot!(".pdf", bytes);
}

#[test]
fn metadata_all_fields() {
    let config = base_config().with_metadata(Metadata {
        title: Some("My Title".into()),
        author: Some("Jane Doe".into()),
        subject: Some("My Subject".into()),
        keywords: vec!["foo".into(), "bar".into()],
        creation_date: Some("2026-01-01T00:00:00".into()),
    });
    let bytes = Html::new(SIMPLE_HTML)
        .with_config(&config)
        .with_stylesheet(FONT_CSS)
        .write_pdf()
        .expect("conversion should succeed")
        .into_bytes();
    insta::assert_binary_snapshot!(".pdf", bytes);
}

#[test]
fn page_count_is_one() {
    let pdf = Html::new(SIMPLE_HTML)
        .with_config(&base_config())
        .with_stylesheet(FONT_CSS)
        .write_pdf()
        .expect("conversion should succeed");
    assert_eq!(pdf.page_count(), 1);
}

#[test]
fn external_css_file() {
    let css_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/external.css");
    let bytes = Html::new(UNSTYLED_HTML)
        .with_config(&base_config())
        .with_stylesheet(FONT_CSS)
        .with_stylesheet(css_path.as_path())
        .write_pdf()
        .expect("conversion should succeed")
        .into_bytes();
    insta::assert_binary_snapshot!(".pdf", bytes);
}

#[test]
fn linked_css() {
    let html_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/linked.html");
    let base_url = url::Url::from_file_path(&html_path).unwrap();
    let html_content = std::fs::read_to_string(&html_path).unwrap();
    let bytes = Html::new(html_content)
        .with_config(&base_config())
        .with_stylesheet(FONT_CSS)
        .with_base_url(base_url)
        .write_pdf()
        .expect("conversion should succeed")
        .into_bytes();
    insta::assert_binary_snapshot!(".pdf", bytes);
}

#[test]
fn convert_simple() {
    let bytes = docoxide::convert("<h1>Hello</h1>", None).expect("conversion should succeed");
    assert!(!bytes.is_empty());
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn convert_with_css() {
    let bytes = docoxide::convert("<h1>Hello</h1>", Some("h1 { color: red; }")).expect("conversion should succeed");
    assert!(!bytes.is_empty());
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn multipage_overflow() {
    let html = include_str!("fixtures/multipage.html");
    let pdf = Html::new(html)
        .with_config(&base_config())
        .with_stylesheet(FONT_CSS)
        .write_pdf()
        .expect("conversion should succeed");
    assert!(
        pdf.page_count() > 1,
        "expected multiple pages, got {}",
        pdf.page_count()
    );
    insta::assert_binary_snapshot!(".pdf", pdf.into_bytes());
}
