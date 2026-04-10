use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use anyrender::{Glyph as AnyGlyph, NormalizedCoord, PaintRef, PaintScene};
use kurbo::{Affine, PathEl, Shape, Stroke as KurboStroke};
use peniko::color::Srgb;
use peniko::{BlendMode, Color, Fill, FontData, GradientKind, ImageFormat, StyleRef};
use skrifa::MetadataProvider;

use krilla::color::rgb;
use krilla::geom::{Path, PathBuilder, Point, Size as KrillaSize, Transform};
use krilla::image::Image as KrillaImage;
use krilla::num::NormalizedF32;
use krilla::paint::{Fill as KrillaFill, FillRule, LineCap, LineJoin, Stroke as KrillaStroke};
use krilla::text::{Font, GlyphId};

use crate::types::PageMargins;

struct StoredGlyph {
    id: u32,
    x: f32,
    y: f32,
}

#[derive(Clone)]
enum PathElement {
    Move(f32, f32),
    Line(f32, f32),
    Quad(f32, f32, f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Close,
}

#[derive(Clone)]
struct PathData(Vec<PathElement>);

impl PathData {
    fn from_shape(shape: &impl Shape) -> Self {
        Self::from_shape_transformed(shape, Affine::IDENTITY)
    }

    fn from_shape_transformed(shape: &impl Shape, transform: Affine) -> Self {
        let mut els = Vec::new();
        for el in shape.path_elements(0.1) {
            match el {
                PathEl::MoveTo(p) => {
                    let tp = transform * p;
                    els.push(PathElement::Move(tp.x as f32, tp.y as f32));
                }
                PathEl::LineTo(p) => {
                    let tp = transform * p;
                    els.push(PathElement::Line(tp.x as f32, tp.y as f32));
                }
                PathEl::QuadTo(p1, p2) => {
                    let tp1 = transform * p1;
                    let tp2 = transform * p2;
                    els.push(PathElement::Quad(
                        tp1.x as f32,
                        tp1.y as f32,
                        tp2.x as f32,
                        tp2.y as f32,
                    ));
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    let tp1 = transform * p1;
                    let tp2 = transform * p2;
                    let tp3 = transform * p3;
                    els.push(PathElement::Cubic(
                        tp1.x as f32,
                        tp1.y as f32,
                        tp2.x as f32,
                        tp2.y as f32,
                        tp3.x as f32,
                        tp3.y as f32,
                    ));
                }
                PathEl::ClosePath => els.push(PathElement::Close),
            }
        }
        PathData(els)
    }

    fn to_krilla(&self) -> Option<Path> {
        let mut b = PathBuilder::new();
        for el in &self.0 {
            match el {
                PathElement::Move(x, y) => b.move_to(*x, *y),
                PathElement::Line(x, y) => b.line_to(*x, *y),
                PathElement::Quad(x1, y1, x, y) => b.quad_to(*x1, *y1, *x, *y),
                PathElement::Cubic(x1, y1, x2, y2, x, y) => {
                    b.cubic_to(*x1, *y1, *x2, *y2, *x, *y);
                }
                PathElement::Close => b.close(),
            }
        }
        b.finish()
    }
}

fn affine_to_transform(affine: Affine) -> Transform {
    let c = affine.as_coeffs();
    Transform::from_row(
        c[0] as f32,
        c[1] as f32,
        c[2] as f32,
        c[3] as f32,
        c[4] as f32,
        c[5] as f32,
    )
}

fn color_to_paint(color: Color) -> (krilla::paint::Paint, NormalizedF32) {
    let rgba = color.to_rgba8();
    let paint = rgb::Color::new(rgba.r, rgba.g, rgba.b).into();
    let opacity = NormalizedF32::new(rgba.a as f32 / 255.0).unwrap_or(NormalizedF32::ONE);
    (paint, opacity)
}

fn brush_to_fill(brush: PaintRef<'_>, brush_alpha: f32, rule: FillRule) -> KrillaFill {
    match brush {
        anyrender::Paint::Solid(color) => {
            let (paint, opacity) = color_to_paint(color);
            let combined = (opacity.get() * brush_alpha).clamp(0.0, 1.0);
            KrillaFill {
                paint,
                opacity: NormalizedF32::new(combined).unwrap_or(NormalizedF32::ONE),
                rule,
            }
        }
        _ => KrillaFill::default(),
    }
}

fn brush_to_stroke(brush: PaintRef<'_>, style: &KurboStroke) -> KrillaStroke {
    match brush {
        anyrender::Paint::Solid(color) => {
            let (paint, opacity) = color_to_paint(color);
            KrillaStroke {
                paint,
                opacity,
                width: style.width as f32,
                miter_limit: style.miter_limit as f32,
                line_cap: match style.start_cap {
                    kurbo::Cap::Butt => LineCap::Butt,
                    kurbo::Cap::Round => LineCap::Round,
                    kurbo::Cap::Square => LineCap::Square,
                },
                line_join: match style.join {
                    kurbo::Join::Bevel => LineJoin::Bevel,
                    kurbo::Join::Miter => LineJoin::Miter,
                    kurbo::Join::Round => LineJoin::Round,
                },
                dash: None,
            }
        }
        _ => KrillaStroke::default(),
    }
}

enum Op {
    PushTransform(Transform),
    PushClipPath(PathData, FillRule),
    PushOpacity(f32),
    Pop,
    Fill(PathData, KrillaFill),
    Stroke(PathData, KrillaStroke),
    DrawImage {
        pixels: Vec<u8>,
        width: u32,
        height: u32,
        draw_width: f32,
        draw_height: f32,
    },
    DrawGlyphs {
        bytes: Arc<Vec<u8>>,
        orig_font_ptr: usize,
        index: u32,
        size: f32,
        transform: Transform,
        fill: KrillaFill,
        glyphs: Vec<StoredGlyph>,
    },
}

fn gradient_to_krilla_fill(gradient: &peniko::Gradient, brush_transform: Option<Affine>, rule: FillRule) -> KrillaFill {
    use krilla::paint::{LinearGradient as KrillaLinear, SpreadMethod, Stop};

    let stops: Vec<Stop> = gradient
        .stops
        .iter()
        .map(|s| {
            let rgba = s.color.to_alpha_color::<Srgb>().to_rgba8();
            Stop {
                offset: NormalizedF32::new(s.offset).unwrap_or(NormalizedF32::ZERO),
                color: rgb::Color::new(rgba.r, rgba.g, rgba.b).into(),
                opacity: NormalizedF32::new(rgba.a as f32 / 255.0).unwrap_or(NormalizedF32::ZERO),
            }
        })
        .collect();

    if stops.is_empty() {
        return KrillaFill::default();
    }

    let spread = match gradient.extend {
        peniko::Extend::Pad => SpreadMethod::Pad,
        peniko::Extend::Repeat => SpreadMethod::Repeat,
        peniko::Extend::Reflect => SpreadMethod::Reflect,
    };

    let g_transform = brush_transform
        .map(affine_to_transform)
        .unwrap_or(Transform::from_row(1.0, 0.0, 0.0, 1.0, 0.0, 0.0));

    let paint = match &gradient.kind {
        GradientKind::Linear(pos) => KrillaLinear {
            x1: pos.start.x as f32,
            y1: pos.start.y as f32,
            x2: pos.end.x as f32,
            y2: pos.end.y as f32,
            transform: g_transform,
            spread_method: spread,
            stops,
            anti_alias: true,
        }
        .into(),
        _ => return KrillaFill::default(),
    };

    KrillaFill {
        paint,
        opacity: NormalizedF32::ONE,
        rule,
    }
}

fn build_glyph_char_map(bytes: &[u8], index: u32) -> HashMap<u32, char> {
    let mut map = HashMap::new();
    let Ok(font) = skrifa::FontRef::from_index(bytes, index) else {
        return map;
    };
    let charmap = font.charmap();
    for cp in 0x20u32..=0x10FFFF {
        if let Some(ch) = char::from_u32(cp) {
            if let Some(gid) = charmap.map(cp) {
                map.entry(gid.to_u32()).or_insert(ch);
            }
        }
    }
    map
}

struct PageData {
    ops: Vec<Op>,
    width_pt: f32,
    height_pt: f32,
    margins: PageMargins,
}

pub struct PdfScenePainter {
    width_pt: f32,
    height_pt: f32,
    scale: f32,
    pages: Vec<PageData>,
    current_ops: Vec<Op>,
    layer_push_counts: Vec<u8>,
    metadata: Option<krilla::metadata::Metadata>,
    margins: PageMargins,
}

impl PdfScenePainter {
    pub fn new(width_pt: f32, height_pt: f32, width_px: u32) -> Self {
        let scale = if width_px > 0 { width_pt / width_px as f32 } else { 1.0 };
        Self {
            width_pt,
            height_pt,
            scale,
            pages: Vec::new(),
            current_ops: Vec::new(),
            layer_push_counts: Vec::new(),
            metadata: None,
            margins: PageMargins::default(),
        }
    }

    pub fn set_metadata(&mut self, metadata: krilla::metadata::Metadata) {
        self.metadata = Some(metadata);
    }

    pub fn finish_page(&mut self) {
        let ops = std::mem::take(&mut self.current_ops);
        self.pages.push(PageData {
            ops,
            width_pt: self.width_pt,
            height_pt: self.height_pt,
            margins: self.margins,
        });
        self.layer_push_counts.clear();
    }

    pub fn finish(self) -> Result<Vec<u8>, String> {
        use krilla::document::Document;
        use krilla::page::PageSettings;

        let mut doc = Document::new();
        if let Some(meta) = self.metadata {
            doc.set_metadata(meta);
        }

        for page_data in &self.pages {
            let settings = PageSettings::from_wh(page_data.width_pt, page_data.height_pt).ok_or_else(|| {
                format!(
                    "invalid page dimensions: {}x{} pt",
                    page_data.width_pt, page_data.height_pt
                )
            })?;
            let mut page = doc.start_page_with(settings);
            let mut surface = page.surface();

            surface.push_transform(&Transform::from_scale(self.scale, self.scale));

            let page_w = page_data.width_pt / self.scale;
            let page_h = page_data.height_pt / self.scale;
            let mut b = PathBuilder::new();
            b.move_to(0.0, 0.0);
            b.line_to(page_w, 0.0);
            b.line_to(page_w, page_h);
            b.line_to(0.0, page_h);
            b.close();
            surface.push_clip_path(&b.finish().unwrap(), &FillRule::NonZero);

            let margin_x_px = page_data.margins.left / self.scale;
            let margin_y_px = page_data.margins.top / self.scale;
            let has_margin = margin_x_px != 0.0 || margin_y_px != 0.0;
            if has_margin {
                surface.push_transform(&Transform::from_translate(margin_x_px, margin_y_px));
            }

            let ops = &page_data.ops;

            struct GlyphWithText<'a> {
                glyph: &'a StoredGlyph,
                tr: Range<usize>,
                font_size: f32,
            }
            impl krilla::text::Glyph for GlyphWithText<'_> {
                fn glyph_id(&self) -> GlyphId {
                    GlyphId::new(self.glyph.id)
                }
                fn text_range(&self) -> Range<usize> {
                    self.tr.clone()
                }
                fn x_advance(&self, _: f32) -> f32 {
                    0.0
                }
                fn x_offset(&self, size: f32) -> f32 {
                    self.glyph.x * size / self.font_size
                }
                fn y_offset(&self, _: f32) -> f32 {
                    0.0
                }
                fn y_advance(&self, _: f32) -> f32 {
                    0.0
                }
                fn location(&self) -> Option<krilla::surface::Location> {
                    None
                }
            }

            let mut font_char_cache: HashMap<(usize, u32), HashMap<u32, char>> = HashMap::new();

            for op in ops {
                match op {
                    Op::PushTransform(t) => surface.push_transform(t),
                    Op::PushClipPath(path, rule) => {
                        let p = path.to_krilla().unwrap_or_else(|| {
                            let mut b = PathBuilder::new();
                            b.move_to(0.0, 0.0);
                            b.line_to(page_data.width_pt / self.scale, 0.0);
                            b.line_to(page_data.width_pt / self.scale, page_data.height_pt / self.scale);
                            b.line_to(0.0, page_data.height_pt / self.scale);
                            b.close();
                            b.finish().unwrap()
                        });
                        surface.push_clip_path(&p, rule);
                    }
                    Op::PushOpacity(alpha) => {
                        if let Some(n) = NormalizedF32::new(*alpha) {
                            surface.push_opacity(n);
                        }
                    }
                    Op::Pop => surface.pop(),
                    Op::Fill(path, fill) => {
                        if let Some(p) = path.to_krilla() {
                            surface.set_fill(Some(fill.clone()));
                            surface.set_stroke(None);
                            surface.draw_path(&p);
                        }
                    }
                    Op::Stroke(path, stroke) => {
                        if let Some(p) = path.to_krilla() {
                            surface.set_fill(None);
                            surface.set_stroke(Some(stroke.clone()));
                            surface.draw_path(&p);
                        }
                    }
                    Op::DrawImage {
                        pixels,
                        width,
                        height,
                        draw_width,
                        draw_height,
                    } => {
                        let image = KrillaImage::from_rgba8(pixels.clone(), *width, *height);
                        if let Some(size) = KrillaSize::from_wh(*draw_width, *draw_height) {
                            surface.draw_image(image, size);
                        }
                    }
                    Op::DrawGlyphs {
                        bytes,
                        orig_font_ptr,
                        index,
                        size,
                        transform,
                        fill,
                        glyphs,
                    } => {
                        if let Some(font) = Font::new(krilla::Data::from(bytes.as_ref().clone()), *index) {
                            let cache_key = (*orig_font_ptr, *index);
                            let char_map = font_char_cache
                                .entry(cache_key)
                                .or_insert_with(|| build_glyph_char_map(bytes, *index));

                            let mut text = String::new();
                            let glyphs_with_text: Vec<GlyphWithText> = glyphs
                                .iter()
                                .map(|g| {
                                    let start = text.len();
                                    let ch = char_map.get(&g.id).copied().unwrap_or('\u{FFFD}');
                                    text.push(ch);
                                    GlyphWithText {
                                        glyph: g,
                                        tr: start..text.len(),
                                        font_size: *size,
                                    }
                                })
                                .collect();

                            let baseline = glyphs.first().map(|g| -g.y).unwrap_or(0.0);
                            surface.set_fill(Some(fill.clone()));
                            surface.set_stroke(None);
                            surface.push_transform(transform);
                            surface.draw_glyphs(
                                Point::from_xy(0.0, baseline),
                                &glyphs_with_text,
                                font,
                                &text,
                                *size,
                                false,
                            );
                            surface.pop();
                        }
                    }
                }
            }

            if has_margin {
                surface.pop();
            }
            surface.pop();
            surface.pop();
            surface.finish();
            page.finish();
        }

        doc.finish().map_err(|e| format!("PDF generation failed: {e:?}"))
    }
}

impl PaintScene for PdfScenePainter {
    fn reset(&mut self) {
        self.current_ops.clear();
        self.layer_push_counts.clear();
    }

    fn push_layer(&mut self, _blend: impl Into<BlendMode>, alpha: f32, transform: Affine, clip: &impl Shape) {
        let mut count = 0u8;
        let bb = clip.bounding_box();
        if bb.width() >= 1.0 && bb.height() >= 1.0 {
            self.current_ops.push(Op::PushClipPath(
                PathData::from_shape_transformed(clip, transform),
                FillRule::NonZero,
            ));
            count += 1;
        }

        if alpha > 0.0 && alpha < 1.0 {
            self.current_ops.push(Op::PushOpacity(alpha));
            count += 1;
        }

        self.layer_push_counts.push(count);
    }

    fn push_clip_layer(&mut self, transform: Affine, clip: &impl Shape) {
        let bb = clip.bounding_box();
        if bb.width() >= 1.0 && bb.height() >= 1.0 {
            self.current_ops.push(Op::PushClipPath(
                PathData::from_shape_transformed(clip, transform),
                FillRule::NonZero,
            ));
            self.layer_push_counts.push(1);
        } else {
            self.layer_push_counts.push(0);
        }
    }

    fn pop_layer(&mut self) {
        let count = self.layer_push_counts.pop().unwrap_or(0);
        for _ in 0..count {
            self.current_ops.push(Op::Pop);
        }
    }

    fn stroke<'a>(
        &mut self,
        style: &KurboStroke,
        transform: Affine,
        brush: impl Into<PaintRef<'a>>,
        _brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        let stroke = brush_to_stroke(brush.into(), style);
        let path = PathData::from_shape(shape);
        self.current_ops.push(Op::PushTransform(affine_to_transform(transform)));
        self.current_ops.push(Op::Stroke(path, stroke));
        self.current_ops.push(Op::Pop);
    }

    fn fill<'a>(
        &mut self,
        style: Fill,
        transform: Affine,
        brush: impl Into<PaintRef<'a>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        let brush_ref = brush.into();

        if let anyrender::Paint::Image(image_brush) = &brush_ref {
            let image_data = image_brush.image;
            let bb = shape.bounding_box();
            let draw_width = bb.width() as f32;
            let draw_height = bb.height() as f32;
            if draw_width < 1.0 || draw_height < 1.0 {
                return;
            }
            let raw = image_data.data.data();
            let pixels: Vec<u8> = match image_data.format {
                ImageFormat::Rgba8 => raw.to_vec(),
                ImageFormat::Bgra8 => raw.chunks_exact(4).flat_map(|c| [c[2], c[1], c[0], c[3]]).collect(),
                _ => return,
            };
            self.current_ops.push(Op::PushTransform(affine_to_transform(transform)));
            self.current_ops.push(Op::DrawImage {
                pixels,
                width: image_data.width,
                height: image_data.height,
                draw_width,
                draw_height,
            });
            self.current_ops.push(Op::Pop);
            return;
        }

        let rule = match style {
            Fill::NonZero => FillRule::NonZero,
            Fill::EvenOdd => FillRule::EvenOdd,
        };

        let fill = match &brush_ref {
            anyrender::Paint::Gradient(gradient) => gradient_to_krilla_fill(gradient, brush_transform, rule),
            _ => brush_to_fill(brush_ref, 1.0, rule),
        };

        let path = PathData::from_shape(shape);
        self.current_ops.push(Op::PushTransform(affine_to_transform(transform)));
        self.current_ops.push(Op::Fill(path, fill));
        self.current_ops.push(Op::Pop);
    }

    fn draw_glyphs<'a, 's: 'a>(
        &'s mut self,
        font: &'a FontData,
        font_size: f32,
        _hint: bool,
        _normalized_coords: &'a [NormalizedCoord],
        style: impl Into<StyleRef<'a>>,
        brush: impl Into<PaintRef<'a>>,
        brush_alpha: f32,
        transform: Affine,
        _glyph_transform: Option<Affine>,
        glyphs: impl Iterator<Item = AnyGlyph>,
    ) {
        let rule = match style.into() {
            StyleRef::Fill(f) => match f {
                Fill::NonZero => FillRule::NonZero,
                Fill::EvenOdd => FillRule::EvenOdd,
            },
            StyleRef::Stroke(_) => FillRule::NonZero,
        };
        let fill = brush_to_fill(brush.into(), brush_alpha, rule);
        let orig_font_ptr = font.data.data().as_ptr() as usize;
        let bytes = Arc::new(font.data.data().to_vec());
        let stored_glyphs: Vec<StoredGlyph> = glyphs
            .map(|g| StoredGlyph {
                id: g.id,
                x: g.x,
                y: -g.y,
            })
            .collect();

        if stored_glyphs.is_empty() {
            return;
        }

        self.current_ops.push(Op::DrawGlyphs {
            bytes,
            orig_font_ptr,
            index: font.index,
            size: font_size,
            transform: affine_to_transform(transform),
            fill,
            glyphs: stored_glyphs,
        });
    }

    fn draw_box_shadow(&mut self, _transform: Affine, _rect: kurbo::Rect, _brush: Color, _radius: f64, _std_dev: f64) {}
}
