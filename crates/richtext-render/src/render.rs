use parley::layout::PositionedLayoutItem;
use swash::scale::image::Content;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::zeno::{Format, Vector};
use swash::FontRef;
use tiny_skia::{Color as SkiaColor, FillRule, Paint, PathBuilder, PixmapMut, Rect, Transform};

use crate::layout::{Color, LayoutBlock, CODE_BLOCK_PAD};

pub struct Renderer {
    scale_cx: ScaleContext,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            scale_cx: ScaleContext::new(),
        }
    }

    pub fn paint_blocks(&mut self, pixmap: &mut PixmapMut<'_>, blocks: &[LayoutBlock]) {
        for block in blocks {
            if block.is_hr {
                draw_hr(
                    pixmap,
                    block.x,
                    block.y,
                    pixmap.width() as f32 - block.x * 2.0,
                );
                continue;
            }

            // Draw background (code blocks)
            if let Some(bg) = block.bg {
                let w = pixmap.width() as f32 - block.x + CODE_BLOCK_PAD;
                let h = block.layout.height() + CODE_BLOCK_PAD * 2.0;
                let bg_x = block.x - CODE_BLOCK_PAD;
                let bg_y = block.y - CODE_BLOCK_PAD;
                if let Some(rect) = Rect::from_xywh(bg_x, bg_y, w, h) {
                    let mut paint = Paint::default();
                    paint.set_color(SkiaColor::from_rgba8(bg.r, bg.g, bg.b, bg.a));
                    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                }
            }

            // Draw bullet
            if let Some(ref bullet) = block.bullet {
                self.draw_bullet_text(pixmap, bullet, block.bullet_x, block.y, &block.layout);
            }

            // Render glyphs
            self.paint_layout(pixmap, &block.layout, block.x, block.y);
        }
    }

    fn paint_layout(
        &mut self,
        pixmap: &mut PixmapMut<'_>,
        layout: &parley::layout::Layout<Color>,
        offset_x: f32,
        offset_y: f32,
    ) {
        for line in layout.lines() {
            for item in line.items() {
                match item {
                    PositionedLayoutItem::GlyphRun(glyph_run) => {
                        let run = glyph_run.run();
                        let color = glyph_run.style().brush;
                        let font = run.font();
                        let font_size = run.font_size();
                        let _synthesis = run.synthesis();

                        let font_ref = FontRef::from_index(font.data.as_ref(), font.index as usize);
                        let font_ref = match font_ref {
                            Some(f) => f,
                            None => continue,
                        };

                        let mut scaler = self
                            .scale_cx
                            .builder(font_ref)
                            .size(font_size)
                            .hint(true)
                            .build();

                        let baseline = glyph_run.baseline();
                        let glyph_offset = glyph_run.offset();

                        // Handle strikethrough
                        let style = glyph_run.style();
                        if style.strikethrough.is_some() {
                            let run_x = offset_x + glyph_offset;
                            let strike_y = offset_y + baseline - font_size * 0.3;
                            let run_width: f32 = glyph_run.glyphs().map(|g| g.advance).sum();
                            if let Some(rect) = Rect::from_xywh(run_x, strike_y, run_width, 1.0) {
                                let mut paint = Paint::default();
                                paint.set_color(SkiaColor::from_rgba8(
                                    color.r, color.g, color.b, color.a,
                                ));
                                pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                            }
                        }

                        for glyph in glyph_run.glyphs() {
                            let image = Render::new(&[
                                Source::ColorOutline(0),
                                Source::ColorBitmap(StrikeWith::BestFit),
                                Source::Outline,
                            ])
                            .format(Format::Alpha)
                            .offset(Vector::new(glyph.x, glyph.y))
                            .render(&mut scaler, glyph.id as u16);

                            let image = match image {
                                Some(img) => img,
                                None => continue,
                            };

                            let gx =
                                (offset_x + glyph_offset + glyph.x + image.placement.left as f32)
                                    as i32;
                            let gy =
                                (offset_y + baseline + glyph.y - image.placement.top as f32) as i32;

                            match image.content {
                                Content::Mask => {
                                    blit_mask(
                                        pixmap,
                                        &image.data,
                                        image.placement.width as i32,
                                        image.placement.height as i32,
                                        gx,
                                        gy,
                                        color,
                                    );
                                }
                                Content::SubpixelMask | Content::Color => {
                                    blit_mask(
                                        pixmap,
                                        &image.data,
                                        image.placement.width as i32,
                                        image.placement.height as i32,
                                        gx,
                                        gy,
                                        color,
                                    );
                                }
                            }
                        }
                    }
                    PositionedLayoutItem::InlineBox(_) => {}
                }
            }
        }
    }

    fn draw_bullet_text(
        &mut self,
        pixmap: &mut PixmapMut<'_>,
        _bullet: &str,
        x: f32,
        y: f32,
        ref_layout: &parley::layout::Layout<Color>,
    ) {
        // Draw a simple bullet circle instead of text to avoid needing a second layout
        let first_baseline = ref_layout
            .lines()
            .next()
            .and_then(|line| {
                line.items().next().map(|item| match item {
                    PositionedLayoutItem::GlyphRun(gr) => gr.baseline(),
                    PositionedLayoutItem::InlineBox(_) => 0.0,
                })
            })
            .unwrap_or(12.0);

        let cx = x + 4.0;
        let cy = y + first_baseline - 4.0;

        if _bullet.starts_with('\u{2022}') {
            // Unordered: draw a filled circle
            if let Some(path) = {
                let mut pb = PathBuilder::new();
                pb.push_circle(cx, cy, 3.0);
                pb.finish()
            } {
                let mut paint = Paint::default();
                paint.set_color(SkiaColor::from_rgba8(30, 30, 30, 255));
                pixmap.fill_path(
                    &path,
                    &paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
        } else {
            // Ordered: render as number glyphs — for simplicity, draw a small rect
            if let Some(rect) = Rect::from_xywh(cx - 3.0, cy - 3.0, 6.0, 6.0) {
                let mut paint = Paint::default();
                paint.set_color(SkiaColor::from_rgba8(30, 30, 30, 255));
                pixmap.fill_rect(rect, &paint, Transform::identity(), None);
            }
        }
    }
}

fn draw_hr(pixmap: &mut PixmapMut<'_>, x: f32, y: f32, width: f32) {
    let hr_y = y + 10.0;
    if let Some(rect) = Rect::from_xywh(x, hr_y, width, 1.0) {
        let mut paint = Paint::default();
        paint.set_color(SkiaColor::from_rgba8(180, 180, 180, 255));
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
}

/// Alpha-blend a glyph mask onto the pixmap.
fn blit_mask(
    pixmap: &mut PixmapMut<'_>,
    data: &[u8],
    width: i32,
    height: i32,
    x: i32,
    y: i32,
    color: Color,
) {
    let px_width = pixmap.width() as i32;
    let px_height = pixmap.height() as i32;
    let pixels = pixmap.pixels_mut();

    for row in 0..height {
        let py = y + row;
        if py < 0 || py >= px_height {
            continue;
        }
        for col in 0..width {
            let px = x + col;
            if px < 0 || px >= px_width {
                continue;
            }
            let alpha = data[(row * width + col) as usize];
            if alpha == 0 {
                continue;
            }
            let idx = (py * px_width + px) as usize;
            let dst = pixels[idx];
            let dst_r = dst.red();
            let dst_g = dst.green();
            let dst_b = dst.blue();

            let a = alpha as u16;
            let inv_a = 255 - a;
            let r = ((color.r as u16 * a + dst_r as u16 * inv_a) / 255) as u8;
            let g = ((color.g as u16 * a + dst_g as u16 * inv_a) / 255) as u8;
            let b = ((color.b as u16 * a + dst_b as u16 * inv_a) / 255) as u8;

            pixels[idx] = tiny_skia::PremultipliedColorU8::from_rgba(r, g, b, 255).unwrap();
        }
    }
}
