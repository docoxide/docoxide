use std::sync::Arc;

use blitz_dom::DEFAULT_CSS as BLITZ_DEFAULT_CSS;
use blitz_html::HtmlDocument;
use blitz_traits::net::NetProvider;
use blitz_traits::shell::{ColorScheme, Viewport};
use parley::FontContext;
use parley::fontique::Blob;

use crate::error::Result;
use crate::fonts::build_font_context;
use crate::html::Html;
use crate::painter::PdfScenePainter;
use crate::pdf::Pdf;
use crate::source::HtmlSource;
use crate::types::Metadata;

const DEFAULT_CSS: &str = include_str!("default.css");

#[cfg(not(target_arch = "wasm32"))]
pub fn run(html: Html) -> Result<Pdf> {
    let config = html.config.unwrap_or_default();

    let Html {
        source,
        base_url,
        stylesheets,
        ..
    } = html;

    let mut font_ctx = build_font_context();
    resolve_fonts(&mut font_ctx, config.fonts)?;

    // A4 default; @page support will make this configurable
    let width_pt: f32 = 595.28;
    let height_pt: f32 = 841.89;
    let width_px = (width_pt * 96.0 / 72.0) as u32;
    let height_px = (height_pt * 96.0 / 72.0) as u32;
    let viewport = Viewport::new(width_px, height_px, 1.0, ColorScheme::Light);

    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    let _guard = rt.enter();
    let provider = Arc::new(blitz_net::Provider::new(None));
    let (html_string, base_url_str) = load_source(source, base_url, &rt, &provider)?;
    let extra_css = resolve_stylesheets(stylesheets)?;

    let doc_config = blitz_dom::DocumentConfig {
        viewport: Some(viewport),
        base_url: base_url_str,
        font_ctx: Some(font_ctx),
        ua_stylesheets: Some(vec![BLITZ_DEFAULT_CSS.to_string(), DEFAULT_CSS.to_string()]),
        net_provider: Some(provider.clone() as Arc<dyn NetProvider>),
        ..Default::default()
    };

    let mut doc = HtmlDocument::from_html(&html_string, doc_config);
    if !extra_css.is_empty() {
        doc.add_user_agent_stylesheet(&extra_css);
    }

    const MAX_TICKS: usize = 100;
    for _ in 0..MAX_TICKS {
        doc.resolve(0.0);
        if provider.is_empty() {
            break;
        }
    }
    doc.resolve(0.0);

    render_pdf(&mut doc, width_pt, height_pt, width_px, height_px, &config.metadata)
}

/// Convenience entry point for the simple `convert(html, css)` API.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_simple(html_str: &str, css: Option<&str>) -> Result<Vec<u8>> {
    let mut h = Html::new(html_str);
    if let Some(css) = css {
        h = h.with_stylesheet(css);
    }
    run(h).map(|p| p.into_bytes())
}

#[cfg(not(target_arch = "wasm32"))]
fn load_source(
    source: HtmlSource,
    base_url: Option<url::Url>,
    rt: &tokio::runtime::Runtime,
    provider: &Arc<blitz_net::Provider>,
) -> Result<(String, Option<String>)> {
    use crate::error::Error;
    match source {
        HtmlSource::String(s) => Ok((s, base_url.map(|u| u.to_string()))),
        HtmlSource::Url(url) => {
            let (resolved_url, bytes) = rt
                .block_on(provider.fetch_async(blitz_traits::net::Request::get(url)))
                .map_err(|e| Error::Network(format!("{e:?}")))?;
            let html = String::from_utf8(bytes.to_vec()).map_err(|e| Error::Network(e.to_string()))?;
            Ok((html, base_url.map(|u| u.to_string()).or(Some(resolved_url))))
        }
    }
}

fn render_pdf(
    doc: &mut HtmlDocument,
    width_pt: f32,
    height_pt: f32,
    width_px: u32,
    height_px: u32,
    metadata: &Metadata,
) -> Result<Pdf> {
    let scale = 1.0f64;
    let phys_w = (width_px as f64 * scale) as u32;
    let content_height_px = doc.root_element().final_layout.size.height.ceil();
    let page_height = height_px as f32;

    // Build page start positions from overflow
    let mut page_starts: Vec<f32> = vec![0.0];
    if content_height_px > page_height {
        let overflow_pages = ((content_height_px - page_height) / page_height).ceil() as usize;
        for i in 1..=overflow_pages {
            page_starts.push(i as f32 * page_height);
        }
    }

    let page_count = page_starts.len();

    let mut painter = PdfScenePainter::new(width_pt, height_pt, phys_w);
    if let Some(meta) = into_krilla_metadata(metadata)? {
        painter.set_metadata(meta);
    }

    for (i, start_y) in page_starts.iter().enumerate() {
        let next_y = page_starts.get(i + 1).copied().unwrap_or(*start_y + page_height);
        // Subtract 1px to avoid painting elements that start exactly at the next break
        let visible_h = ((next_y - *start_y).min(page_height).max(1.0) - 1.0).max(1.0) as u32;
        doc.set_viewport_scroll(blitz_dom::Point {
            x: 0.0,
            y: *start_y as f64,
        });
        blitz_paint::paint_scene(&mut painter, doc, scale, phys_w, visible_h, 0, 0);
        painter.finish_page();
    }

    let bytes = painter.finish().map_err(crate::error::Error::PdfGeneration)?;
    Ok(Pdf { bytes, page_count })
}

fn into_krilla_metadata(meta: &Metadata) -> Result<Option<krilla::metadata::Metadata>> {
    if meta.title.is_none()
        && meta.author.is_none()
        && meta.subject.is_none()
        && meta.keywords.is_empty()
        && meta.creation_date.is_none()
    {
        return Ok(None);
    }
    let mut kmeta = krilla::metadata::Metadata::new();
    if let Some(title) = &meta.title {
        kmeta = kmeta.title(title.clone());
    }
    if let Some(author) = &meta.author {
        kmeta = kmeta.authors(vec![author.clone()]);
    }
    if let Some(subject) = &meta.subject {
        kmeta = kmeta.description(subject.clone());
    }
    if !meta.keywords.is_empty() {
        kmeta = kmeta.keywords(meta.keywords.clone());
    }
    if let Some(date_str) = &meta.creation_date {
        let dt = parse_datetime(date_str).ok_or_else(|| {
            crate::error::Error::PdfGeneration(format!(
                "invalid creation_date '{date_str}', expected YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS"
            ))
        })?;
        kmeta = kmeta.creation_date(dt);
    }
    Ok(Some(kmeta))
}

/// Parses "YYYY-MM-DD" or "YYYY-MM-DDTHH:MM:SS" into krilla DateTime.
/// Returns None if the format is invalid.
fn parse_datetime(s: &str) -> Option<krilla::metadata::DateTime> {
    let parts: Vec<&str> = s.split('T').collect();
    let date_parts: Vec<&str> = parts.first()?.split('-').collect();
    if date_parts.len() != 3 {
        return None;
    }
    let year: u16 = date_parts[0].parse().ok()?;
    let month: u8 = date_parts[1].parse().ok()?;
    let day: u8 = date_parts[2].parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let mut dt = krilla::metadata::DateTime::new(year).month(month).day(day);

    if let Some(time_str) = parts.get(1) {
        let time_parts: Vec<&str> = time_str.split(':').collect();
        if time_parts.len() != 3 {
            return None;
        }
        let hour: u8 = time_parts[0].parse().ok()?;
        let minute: u8 = time_parts[1].parse().ok()?;
        let second: u8 = time_parts[2].parse().ok()?;
        if hour > 23 || minute > 59 || second > 59 {
            return None;
        }
        dt = dt.hour(hour).minute(minute).second(second);
    }

    Some(dt)
}

#[cfg(target_arch = "wasm32")]
pub async fn run(html: Html) -> Result<Pdf> {
    let config = html.config.unwrap_or_default();

    let Html {
        source,
        stylesheets,
        base_url,
        ..
    } = html;

    let mut font_ctx = build_font_context();
    resolve_fonts(&mut font_ctx, config.fonts)?;

    let (html_string, base_url_str) = match source {
        HtmlSource::String(s) => (s, base_url.map(|u| u.to_string())),
        HtmlSource::Url(url) => {
            let resolved = url.to_string();
            let bytes = js_fetch(url.as_str())
                .await
                .ok_or_else(|| crate::error::Error::Network(format!("failed to fetch {url}")))?;
            let html_str =
                String::from_utf8(bytes.to_vec()).map_err(|e| crate::error::Error::Network(e.to_string()))?;
            (html_str, base_url.map(|u| u.to_string()).or(Some(resolved)))
        }
    };
    let extra_css = resolve_stylesheets(stylesheets)?;

    let width_pt: f32 = 595.28;
    let height_pt: f32 = 841.89;
    let width_px = (width_pt * 96.0 / 72.0) as u32;
    let height_px = (height_pt * 96.0 / 72.0) as u32;
    let viewport = Viewport::new(width_px, height_px, 1.0, ColorScheme::Light);

    let provider = WasmNetProvider::new();

    let doc_config = blitz_dom::DocumentConfig {
        viewport: Some(viewport),
        base_url: base_url_str,
        font_ctx: Some(font_ctx),
        ua_stylesheets: Some(vec![BLITZ_DEFAULT_CSS.to_string(), DEFAULT_CSS.to_string()]),
        net_provider: Some(provider.clone() as Arc<dyn NetProvider>),
        ..Default::default()
    };

    let mut doc = HtmlDocument::from_html(&html_string, doc_config);
    if !extra_css.is_empty() {
        doc.add_user_agent_stylesheet(&extra_css);
    }

    // Poll until all pending network requests (images, CSS, fonts) finish.
    // Each tick yields to the JS event loop via setTimeout(0). If a fetch
    // hangs, we give up after MAX_TICKS (~400ms in browsers). Assets that
    // fail to load are silently skipped by blitz (broken images, missing CSS).
    const MAX_TICKS: usize = 100;
    for _ in 0..MAX_TICKS {
        doc.resolve(0.0);
        if provider.is_empty() {
            break;
        }
        next_tick().await;
    }
    doc.resolve(0.0);

    render_pdf(&mut doc, width_pt, height_pt, width_px, height_px, &config.metadata)
}

#[cfg(target_arch = "wasm32")]
pub async fn run_simple(html_str: &str, css: Option<&str>) -> Result<Vec<u8>> {
    let mut h = Html::new(html_str);
    if let Some(css) = css {
        h = h.with_stylesheet(css);
    }
    run(h).await.map(|p| p.into_bytes())
}

#[cfg(target_arch = "wasm32")]
struct WasmNetProvider {
    pending: Arc<std::sync::atomic::AtomicUsize>,
}

#[cfg(target_arch = "wasm32")]
impl WasmNetProvider {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            pending: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        })
    }

    fn is_empty(&self) -> bool {
        self.pending.load(std::sync::atomic::Ordering::SeqCst) == 0
    }
}

#[cfg(target_arch = "wasm32")]
impl NetProvider for WasmNetProvider {
    fn fetch(
        &self,
        _doc_id: usize,
        request: blitz_traits::net::Request,
        handler: Box<dyn blitz_traits::net::NetHandler>,
    ) {
        use std::sync::atomic::Ordering;
        self.pending.fetch_add(1, Ordering::SeqCst);
        let pending = self.pending.clone();
        let url = request.url.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            let bytes = js_fetch(&url).await.unwrap_or_default();
            handler.bytes(url, bytes);
            pending.fetch_sub(1, Ordering::SeqCst);
        });
    }
}

#[cfg(target_arch = "wasm32")]
async fn js_fetch(url: &str) -> Option<blitz_traits::net::Bytes> {
    use js_sys::{Function, Promise, Uint8Array};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    let global = js_sys::global();
    let fetch_fn: Function = js_sys::Reflect::get(&global, &JsValue::from_str("fetch"))
        .ok()?
        .dyn_into()
        .ok()?;
    let response = JsFuture::from(
        fetch_fn
            .call1(&JsValue::UNDEFINED, &JsValue::from_str(url))
            .ok()?
            .dyn_into::<Promise>()
            .ok()?,
    )
    .await
    .ok()?;
    let ab_fn: Function = js_sys::Reflect::get(&response, &JsValue::from_str("arrayBuffer"))
        .ok()?
        .dyn_into()
        .ok()?;
    let array_buffer = JsFuture::from(ab_fn.call0(&response).ok()?.dyn_into::<Promise>().ok()?)
        .await
        .ok()?;
    Some(blitz_traits::net::Bytes::from(Uint8Array::new(&array_buffer).to_vec()))
}

#[cfg(target_arch = "wasm32")]
async fn next_tick() {
    use js_sys::{Function, Reflect};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    let promise = js_sys::Promise::new(&mut |resolve, _| {
        let global = js_sys::global();
        if let Ok(val) = Reflect::get(&global, &JsValue::from_str("setTimeout")) {
            let set_timeout: Function = val.unchecked_into();
            let _ = set_timeout.call2(&JsValue::UNDEFINED, &resolve, &JsValue::from(0));
        }
    });
    let _ = JsFuture::from(promise).await;
}

fn resolve_fonts(font_ctx: &mut FontContext, fonts: Vec<crate::source::FontSource>) -> Result<()> {
    use crate::source::FontSource;
    for font in fonts {
        let bytes: Vec<u8> = match font {
            FontSource::Bytes(b) => b.to_vec(),
            #[cfg(not(target_arch = "wasm32"))]
            FontSource::File(path) => std::fs::read(&path)?,
        };
        font_ctx
            .collection
            .register_fonts(Blob::new(Arc::new(bytes) as _), None);
    }
    Ok(())
}

fn resolve_stylesheets(sheets: Vec<crate::source::StylesheetSource>) -> Result<String> {
    use crate::source::StylesheetSource;
    let mut combined = String::new();
    for sheet in sheets {
        if !combined.is_empty() {
            combined.push('\n');
        }
        match sheet {
            StylesheetSource::String(s) => combined.push_str(&s),
            #[cfg(not(target_arch = "wasm32"))]
            StylesheetSource::File(path) => {
                combined.push_str(&std::fs::read_to_string(&path)?);
            }
        }
    }
    Ok(combined)
}
