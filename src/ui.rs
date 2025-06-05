use crate::types::{Color, Image, Pos, Vec2};
#[cfg(all(feature = "serde", feature = "skia"))]
use serde::{Deserialize, Serialize};
#[cfg(feature = "egui")]
pub(crate) struct Painter<'a> {
    painter: &'a egui::Painter,
    line: Line,
    line_width: f32,
    pub offset: Pos,
}
#[cfg(feature = "egui")]
impl<'a> Painter<'a> {
    pub(crate) fn new(
        ui: &'a egui::Ui,
        fast: bool,
        size: usize,
        line_width: f32,
        offset: Pos,
    ) -> Self {
        Self {
            painter: ui.painter(),
            line: if fast {
                Line::Fast(FastLine {
                    line: Default::default(),
                    size,
                    line_width,
                })
            } else {
                Line::Slow(SlowLine {
                    line: Vec::with_capacity(size),
                    color: Color::new(0, 0, 0),
                    size,
                    line_width,
                })
            },
            line_width,
            offset,
        }
    }
    pub fn circle(&mut self, p0: Pos, r: f32, p2: &Color, p3: f32) {
        self.painter.circle_stroke(
            (self.offset + p0).to_pos2(),
            r,
            egui::Stroke::new(p3, p2.to_col()),
        );
    }
    pub(crate) fn highlight(&mut self, xi: f32, yi: f32, xf: f32, yf: f32, color: &Color) {
        self.painter.rect_filled(
            egui::Rect::from_points(&[egui::Pos2::new(xi, yi), egui::Pos2::new(xf, yf)]),
            0.0,
            color.to_col(),
        );
    }
    pub(crate) fn clear_offset(&mut self, screen: Vec2, background: &Color) {
        self.painter.rect_filled(
            egui::Rect::from_points(&[
                egui::Pos2::new(0.0, 0.0),
                egui::Pos2::new(self.offset.x, screen.y as f32),
            ]),
            0.0,
            background.to_col(),
        );
    }
    pub(crate) fn clear_below(&mut self, screen: Vec2, background: &Color) {
        self.painter.rect_filled(
            egui::Rect::from_points(&[
                egui::Pos2::new(0.0, screen.x as f32),
                egui::Pos2::new(screen.x as f32, screen.y as f32),
            ]),
            0.0,
            background.to_col(),
        );
    }
    pub(crate) fn line_segment(&mut self, p0: [Pos; 2], width: f32, p2: &Color) {
        let p0 = p0.map(|p| {
            self.offset
                + Pos {
                    x: p.x + 0.5,
                    y: p.y + 0.5,
                }
        });
        if width == self.line_width {
            self.line.line(p0, p2, self.painter)
        } else {
            self.painter.line_segment(
                p0.map(|p| p.to_pos2()),
                egui::Stroke::new(width, p2.to_col()),
            );
        }
    }
    pub(crate) fn save(&mut self) {
        self.line.draw(self.painter)
    }
    pub(crate) fn rect_filled(&self, p0: Pos, p2: &Color) {
        let rect =
            egui::Rect::from_center_size((self.offset + p0).to_pos2(), egui::Vec2::splat(5.0));
        self.painter.rect_filled(rect, 0.0, p2.to_col());
    }
    pub(crate) fn image(&self, p0: &Image, pos: Vec2) {
        let d = egui::Rect::from_points(&[
            self.offset.to_pos2(),
            (self.offset + pos.to_pos()).to_pos2(),
        ]);
        let a = egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0));
        let c = egui::Color32::WHITE;
        self.painter.image(p0.0.id(), d, a, c);
    }
    pub(crate) fn hline(&self, p0: f32, p1: f32, p3: &Color) {
        if p1.is_finite() {
            self.painter.hline(
                egui::Rangef::new(self.offset.x, p0 + self.offset.x),
                p1 + self.offset.y,
                egui::Stroke::new(1.0, p3.to_col()),
            );
        }
    }
    pub(crate) fn vline(&self, p0: f32, p1: f32, p3: &Color) {
        if p0.is_finite() {
            self.painter.vline(
                p0 + self.offset.x,
                egui::Rangef::new(self.offset.y, p1 + self.offset.y),
                egui::Stroke::new(1.0, p3.to_col()),
            );
        }
    }
    pub(crate) fn text(
        &self,
        p0: Pos,
        p1: crate::types::Align,
        p2: &str,
        p4: &Color,
        font_size: f32,
    ) -> f32 {
        self.painter
            .text(
                (p0 + self.offset).to_pos2(),
                p1.into(),
                p2,
                egui::FontId::monospace(font_size),
                p4.to_col(),
            )
            .width()
    }
}
#[cfg(feature = "skia")]
pub(crate) struct Painter<'a> {
    surface: &'a mut skia_safe::Surface,
    line: Line,
    font: &'a Option<skia_safe::Font>,
    points: Point,
    fast: bool,
    anti_alias: bool,
    line_width: f32,
    pub offset: Pos,
}
#[cfg(feature = "skia")]
impl<'a> Painter<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        surface: &'a mut skia_safe::Surface,
        background: Color,
        font: &'a Option<skia_safe::Font>,
        fast: bool,
        size: usize,
        anti_alias: bool,
        line_width: f32,
        offset: Pos,
    ) -> Self {
        surface.canvas().clear(background.to_col());
        Self {
            surface,
            line: if fast {
                Line::Fast(FastLine {
                    line: Default::default(),
                    line_width,
                })
            } else {
                Line::Slow(SlowLine {
                    line: Default::default(),
                    color: Color::new(0, 0, 0),
                    last: None,
                    line_width,
                })
            },
            points: if fast {
                Point::Fast(FastPoint {
                    points: Default::default(),
                    size,
                })
            } else {
                Point::Slow(SlowPoint {
                    points: Vec::with_capacity(size),
                    color: Color::new(0, 0, 0),
                })
            },
            font,
            fast,
            anti_alias,
            line_width,
            offset,
        }
    }
    pub(crate) fn line_segment(&mut self, p0: [Pos; 2], width: f32, p2: &Color) {
        let p0 = p0.map(|p| {
            self.offset
                + Pos {
                    x: p.x + 0.5,
                    y: p.y + 0.5,
                }
        });
        if width == self.line_width {
            self.line.line(
                p0,
                p2,
                if self.fast {
                    None
                } else {
                    Some(self.surface.canvas())
                },
            );
        } else {
            self.surface.canvas().draw_line(
                p0[0].to_pos2(),
                p0[1].to_pos2(),
                &make_paint(width, p2, true, false),
            );
        }
    }
    pub fn draw(&mut self) {
        self.line.draw(self.surface.canvas(), self.anti_alias);
    }
    pub fn draw_pts(&mut self) {
        self.points.draw(self.surface.canvas());
    }
    pub fn clear(&mut self) {
        self.line.clear()
    }
    pub fn clear_pts(&mut self) {
        self.points.clear()
    }
    pub fn circle(&mut self, p0: Pos, r: f32, p2: &Color, p3: f32) {
        self.surface.canvas().draw_circle(
            (self.offset + p0).to_pos2(),
            r,
            &make_paint(p3, p2, true, false),
        );
    }
    pub(crate) fn finish(&mut self, clear: bool) {
        self.draw();
        self.draw_pts();
        if clear {
            self.clear();
            self.clear_pts();
        }
    }
    pub(crate) fn highlight(&mut self, xi: f32, yi: f32, xf: f32, yf: f32, color: &Color) {
        let mut paint = make_paint(1.0, color, false, true);
        paint.set_style(skia_safe::PaintStyle::Fill);
        self.surface
            .canvas()
            .draw_rect(skia_safe::Rect::from_ltrb(xi, yi, xf, yf), &paint);
    }
    pub(crate) fn clear_offset(&mut self, screen: Vec2, background: &Color) {
        let mut paint = make_paint(1.0, background, false, true);
        paint.set_style(skia_safe::PaintStyle::Fill);
        self.surface.canvas().draw_rect(
            skia_safe::Rect::from_ltrb(0.0, 0.0, self.offset.x, screen.y as f32),
            &paint,
        );
    }
    pub(crate) fn clear_below(&mut self, screen: Vec2, background: &Color) {
        let mut paint = make_paint(1.0, background, false, true);
        paint.set_style(skia_safe::PaintStyle::Fill);
        self.surface.canvas().draw_rect(
            skia_safe::Rect::from_ltrb(0.0, screen.x as f32, screen.x as f32, screen.y as f32),
            &paint,
        );
    }
    #[cfg(any(feature = "arboard", not(feature = "skia-vulkan")))]
    pub(crate) fn save<T>(&mut self, buffer: &mut T)
    where
        T: std::ops::DerefMut<Target = [u32]>,
    {
        if let Some(pm) = self.surface.canvas().peek_pixels() {
            let px = pm.pixels::<u32>().unwrap();
            buffer.copy_from_slice(px);
        }
    }
    pub(crate) fn save_img(&mut self, format: &ImageFormat) -> Data {
        Data {
            data: self
                .surface
                .image_snapshot()
                .encode(None, format.into(), None)
                .unwrap(),
        }
    }
    pub(crate) fn rect_filled(&mut self, p0: Pos, p2: &Color) {
        let p0 = self.offset
            + Pos {
                x: p0.x + 0.5,
                y: p0.y + 0.5,
            };
        self.points.point(
            p0,
            p2,
            if self.fast {
                None
            } else {
                Some(self.surface.canvas())
            },
        );
    }
    pub(crate) fn image(&mut self, p0: &Image, pos: Vec2) {
        if self.anti_alias {
            let mut paint = skia_safe::Paint::default();
            paint.set_anti_alias(self.anti_alias);
            self.surface.canvas().draw_image_rect_with_sampling_options(
                p0,
                None,
                skia_safe::Rect::new(
                    self.offset.x,
                    self.offset.y,
                    self.offset.x + pos.x as f32,
                    self.offset.y + pos.y as f32,
                ),
                skia_safe::SamplingOptions::new(
                    skia_safe::FilterMode::Linear,
                    skia_safe::MipmapMode::Linear,
                ),
                &paint,
            );
        } else {
            self.surface.canvas().draw_image_rect(
                p0,
                None,
                skia_safe::Rect::new(
                    self.offset.x,
                    self.offset.y,
                    self.offset.x + pos.x as f32,
                    self.offset.y + pos.y as f32,
                ),
                &skia_safe::Paint::default(),
            );
        }
    }
    pub(crate) fn hline(&mut self, p0: f32, p1: f32, p3: &Color) {
        if p1.is_finite() {
            self.surface.canvas().draw_line(
                (self.offset + Pos::new(0.0, p1 + 0.5)).to_pos2(),
                (self.offset + Pos::new(p0, p1 + 0.5)).to_pos2(),
                &make_paint(1.0, p3, false, false),
            );
        }
    }
    pub(crate) fn vline(&mut self, p0: f32, p1: f32, p3: &Color) {
        if p0.is_finite() {
            self.surface.canvas().draw_line(
                (self.offset + Pos::new(p0 + 0.5, 0.0)).to_pos2(),
                (self.offset + Pos::new(p0 + 0.5, p1)).to_pos2(),
                &make_paint(1.0, p3, false, false),
            );
        }
    }
    pub(crate) fn text(&mut self, p0: Pos, p1: crate::types::Align, p2: &str, p4: &Color) -> f32 {
        let Some(font) = self.font else {
            return 0.0;
        };
        let mut pos = (self.offset + p0).to_pos2();
        let paint = make_paint(1.0, p4, false, false);
        let strs = p2.split('\n').collect::<Vec<&str>>();
        let mut body = |s: &str| {
            let rect = font.measure_str(s, Some(&paint)).1;
            let (width, height) = (rect.width(), rect.height());
            if !s.is_empty() {
                let mut pos = pos;
                match p1 {
                    crate::types::Align::CenterBottom
                    | crate::types::Align::CenterCenter
                    | crate::types::Align::CenterTop => pos.x -= width / 2.0,
                    crate::types::Align::LeftBottom
                    | crate::types::Align::LeftCenter
                    | crate::types::Align::LeftTop => {}
                    crate::types::Align::RightBottom
                    | crate::types::Align::RightCenter
                    | crate::types::Align::RightTop => pos.x -= width,
                }
                match p1 {
                    crate::types::Align::CenterCenter
                    | crate::types::Align::LeftCenter
                    | crate::types::Align::RightCenter => pos.y += height / 2.0,
                    crate::types::Align::CenterBottom
                    | crate::types::Align::LeftBottom
                    | crate::types::Align::RightBottom => {}
                    crate::types::Align::CenterTop
                    | crate::types::Align::LeftTop
                    | crate::types::Align::RightTop => pos.y += height,
                }
                self.surface.canvas().draw_str(s, pos, font, &paint);
            }
            match p1 {
                crate::types::Align::CenterTop
                | crate::types::Align::RightTop
                | crate::types::Align::LeftTop => {
                    pos.y += height;
                }
                crate::types::Align::CenterBottom
                | crate::types::Align::RightBottom
                | crate::types::Align::LeftBottom => {
                    pos.y -= height;
                }
                crate::types::Align::CenterCenter
                | crate::types::Align::RightCenter
                | crate::types::Align::LeftCenter => {
                    pos.y += height / 2.0;
                }
            }
        };
        match p1 {
            crate::types::Align::CenterTop
            | crate::types::Align::RightTop
            | crate::types::Align::LeftTop => {
                for s in strs {
                    body(s)
                }
            }
            crate::types::Align::CenterBottom
            | crate::types::Align::RightBottom
            | crate::types::Align::LeftBottom => {
                for s in strs.iter().rev() {
                    body(s)
                }
            }
            crate::types::Align::CenterCenter
            | crate::types::Align::RightCenter
            | crate::types::Align::LeftCenter => {
                for s in strs {
                    body(s)
                }
            }
        }
        font.measure_str(p2, None).0
    }
}
#[cfg(feature = "tiny-skia")]
pub(crate) struct Painter<'a> {
    canvas: tiny_skia::Pixmap,
    font: &'a Option<bdf2::Font>,
    line: Line,
    fast: bool,
    anti_alias: bool,
    line_width: f32,
    pub offset: Pos,
}
#[cfg(feature = "tiny-skia")]
impl<'a> Painter<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        width: u32,
        height: u32,
        background: Color,
        fast: bool,
        anti_alias: bool,
        line_width: f32,
        offset: Pos,
        font: &'a Option<bdf2::Font>,
    ) -> Self {
        let mut canvas = tiny_skia::Pixmap::new(width, height).unwrap();
        canvas.fill(background.to_col());
        Self {
            canvas,
            font,
            line: if fast {
                Line::Fast(FastLine {
                    line: Default::default(),
                    line_width,
                })
            } else {
                Line::Slow(SlowLine {
                    line: Default::default(),
                    color: Color::new(0, 0, 0),
                    last: None,
                    line_width,
                })
            },
            fast,
            anti_alias,
            line_width,
            offset,
        }
    }
    pub(crate) fn line_segment(&mut self, p0: [Pos; 2], width: f32, p2: &Color) {
        let p0 = p0.map(|p| {
            self.offset
                + Pos {
                    x: p.x + 0.5,
                    y: p.y + 0.5,
                }
        });
        if width == self.line_width {
            self.line.line(
                p0,
                p2,
                if self.fast {
                    None
                } else {
                    Some(&mut self.canvas)
                },
            );
        } else {
            let mut path = tiny_skia::PathBuilder::new();
            path.move_to(p0[0].x, p0[0].y);
            path.line_to(p0[1].x, p0[1].y);
            let path = path.finish().unwrap();
            let stroke = tiny_skia::Stroke {
                width,
                ..Default::default()
            };
            self.canvas.stroke_path(
                &path,
                &make_paint(p2, true),
                &stroke,
                tiny_skia::Transform::default(),
                None,
            )
        }
    }
    pub fn draw(&mut self, draw: bool) {
        let new = if draw {
            match self.line {
                Line::Fast(_) => Line::Fast(FastLine {
                    line: Default::default(),
                    line_width: self.line_width,
                }),
                Line::Slow(_) => Line::Slow(SlowLine {
                    line: Default::default(),
                    line_width: self.line_width,
                    color: Color::new(0, 0, 0),
                    last: None,
                }),
                Line::None => Line::None,
            }
        } else {
            Line::None
        };
        let line = std::mem::replace(&mut self.line, new);
        line.draw(&mut self.canvas, self.anti_alias);
    }
    pub fn circle(&mut self, p0: Pos, r: f32, p2: &Color, width: f32) {
        let mut path = tiny_skia::PathBuilder::new();
        path.push_circle(self.offset.x + p0.x, self.offset.y + p0.y, r);
        let path = path.finish().unwrap();
        self.canvas.stroke_path(
            &path,
            &make_paint(p2, true),
            &tiny_skia::Stroke {
                width,
                ..Default::default()
            },
            tiny_skia::Transform::default(),
            None,
        );
    }
    pub(crate) fn save<T>(&mut self, buffer: &mut T)
    where
        T: std::ops::DerefMut<Target = [u32]>,
    {
        let slice: &[tiny_skia::PremultipliedColorU8] = self.canvas.pixels();
        let slice: &[u32] = bytemuck::cast_slice(slice);
        buffer.copy_from_slice(slice);
    }
    pub(crate) fn save_png(&mut self) -> Vec<u8> {
        self.canvas.encode_png().unwrap_or_default()
    }
    pub(crate) fn rect_filled(&mut self, p0: Pos, p2: &Color) {
        self.canvas.fill_rect(
            tiny_skia::Rect::from_ltrb(
                self.offset.x + p0.x - 1.0,
                self.offset.y + p0.y - 1.0,
                self.offset.x + p0.x + 2.0,
                self.offset.y + p0.y + 2.0,
            )
            .unwrap(),
            &make_paint(p2, true),
            tiny_skia::Transform::default(),
            None,
        );
    }
    pub(crate) fn highlight(&mut self, xi: f32, yi: f32, xf: f32, yf: f32, color: &Color) {
        self.canvas.fill_rect(
            tiny_skia::Rect::from_ltrb(xi, yi, xf, yf).unwrap(),
            &make_paint(color, false),
            tiny_skia::Transform::default(),
            None,
        );
    }
    pub(crate) fn clear_offset(&mut self, screen: Vec2, background: &Color) {
        self.canvas.fill_rect(
            tiny_skia::Rect::from_ltrb(0.0, 0.0, self.offset.x, screen.y as f32).unwrap(),
            &make_paint(background, false),
            tiny_skia::Transform::default(),
            None,
        );
    }
    pub(crate) fn clear_below(&mut self, screen: Vec2, background: &Color) {
        self.canvas.fill_rect(
            tiny_skia::Rect::from_ltrb(0.0, screen.x as f32, screen.x as f32, screen.y as f32)
                .unwrap(),
            &make_paint(background, false),
            tiny_skia::Transform::default(),
            None,
        );
    }
    pub(crate) fn image(&mut self, p0: &Image, pos: Vec2) {
        let mut paint = tiny_skia::PixmapPaint::default();
        if self.anti_alias {
            paint.quality = tiny_skia::FilterQuality::Bilinear
        }
        let sx = pos.x as f32 / p0.0.width() as f32;
        let sy = pos.y as f32 / p0.0.height() as f32;
        self.canvas.draw_pixmap(
            0,
            0,
            p0.0.as_ref(),
            &paint,
            tiny_skia::Transform::from_row(sx, 0.0, 0.0, sy, self.offset.x, self.offset.y),
            None,
        );
    }
    pub(crate) fn hline(&mut self, p0: f32, p1: f32, p3: &Color) {
        if p1.is_finite() {
            let mut path = tiny_skia::PathBuilder::new();
            path.move_to(self.offset.x, self.offset.y + p1 + 0.5);
            path.line_to(self.offset.x + p0, self.offset.y + p1 + 0.5);
            let path = path.finish().unwrap();
            self.canvas.stroke_path(
                &path,
                &make_paint(p3, false),
                &tiny_skia::Stroke::default(),
                tiny_skia::Transform::default(),
                None,
            );
        }
    }
    pub(crate) fn vline(&mut self, p0: f32, p1: f32, p3: &Color) {
        if p0.is_finite() {
            let mut path = tiny_skia::PathBuilder::new();
            path.move_to(self.offset.x + p0 + 0.5, self.offset.y);
            path.line_to(self.offset.x + p0 + 0.5, self.offset.y + p1);
            let path = path.finish().unwrap();
            self.canvas.stroke_path(
                &path,
                &make_paint(p3, false),
                &tiny_skia::Stroke::default(),
                tiny_skia::Transform::default(),
                None,
            );
        }
    }
    fn draw_str(
        &mut self,
        s: &str,
        pos: Pos,
        fc: &std::collections::HashMap<char, tiny_skia::Pixmap>,
    ) {
        let (mut pxi, pyi) = (pos.x.round() as i32, pos.y.round() as i32);
        let pyi = pyi + 3;
        let paint = tiny_skia::PixmapPaint::default();
        let transform = tiny_skia::Transform::default();
        for c in s.chars() {
            let pm = fc.get(&c).unwrap();
            self.canvas.draw_pixmap(
                pxi,
                pyi - pm.height() as i32,
                pm.as_ref(),
                &paint,
                transform,
                None,
            );
            pxi += pm.width() as i32;
        }
    }
    pub(crate) fn text(
        &mut self,
        p0: Pos,
        p1: crate::types::Align,
        p2: &str,
        fc: &std::collections::HashMap<char, tiny_skia::Pixmap>,
    ) -> f32 {
        let Some(font) = self.font else {
            return 0.0;
        };
        let mut pos = self.offset + p0;
        let strs = p2.split('\n').collect::<Vec<&str>>();
        let mut body = |s: &str| {
            let (width, height) = get_bounds(font, s);
            if !s.is_empty() {
                let mut pos = pos;
                match p1 {
                    crate::types::Align::CenterBottom
                    | crate::types::Align::CenterCenter
                    | crate::types::Align::CenterTop => pos.x -= width / 2.0,
                    crate::types::Align::LeftBottom
                    | crate::types::Align::LeftCenter
                    | crate::types::Align::LeftTop => {}
                    crate::types::Align::RightBottom
                    | crate::types::Align::RightCenter
                    | crate::types::Align::RightTop => pos.x -= width,
                }
                match p1 {
                    crate::types::Align::CenterCenter
                    | crate::types::Align::LeftCenter
                    | crate::types::Align::RightCenter => pos.y += height / 2.0,
                    crate::types::Align::CenterBottom
                    | crate::types::Align::LeftBottom
                    | crate::types::Align::RightBottom => {}
                    crate::types::Align::CenterTop
                    | crate::types::Align::LeftTop
                    | crate::types::Align::RightTop => pos.y += height,
                }
                self.draw_str(s, pos, fc);
            }
            match p1 {
                crate::types::Align::CenterTop
                | crate::types::Align::RightTop
                | crate::types::Align::LeftTop => {
                    pos.y += height;
                }
                crate::types::Align::CenterBottom
                | crate::types::Align::RightBottom
                | crate::types::Align::LeftBottom => {
                    pos.y -= height;
                }
                crate::types::Align::CenterCenter
                | crate::types::Align::RightCenter
                | crate::types::Align::LeftCenter => {
                    pos.y += height / 2.0;
                }
            }
        };
        match p1 {
            crate::types::Align::CenterTop
            | crate::types::Align::RightTop
            | crate::types::Align::LeftTop => {
                for s in strs {
                    body(s)
                }
            }
            crate::types::Align::CenterBottom
            | crate::types::Align::RightBottom
            | crate::types::Align::LeftBottom => {
                for s in strs.iter().rev() {
                    body(s)
                }
            }
            crate::types::Align::CenterCenter
            | crate::types::Align::RightCenter
            | crate::types::Align::LeftCenter => {
                for s in strs {
                    body(s)
                }
            }
        }
        get_bounds(font, p2).0
    }
}
#[cfg(feature = "tiny-skia")]
fn get_bounds(font: &bdf2::Font, s: &str) -> (f32, f32) {
    let (w, h) = char_dimen(font);
    let vec = s.split('\n').collect::<Vec<&str>>();
    let len = vec.iter().map(|a| a.len()).max().unwrap_or(0);
    ((w * len) as f32, (h * vec.len()) as f32)
}
#[cfg(feature = "tiny-skia")]
pub(crate) fn char_dimen(font: &bdf2::Font) -> (usize, usize) {
    let a = font.glyphs().get(&'a').unwrap();
    (a.width() as usize, a.height() as usize)
}
#[cfg(feature = "skia")]
pub struct Data {
    pub data: skia_safe::Data,
}
#[cfg(feature = "tiny-skia")]
pub struct Data {
    pub data: Vec<u8>,
}
#[cfg(feature = "skia")]
impl Data {
    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_bytes()
    }
}
#[cfg(feature = "tiny-skia")]
impl Data {
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg(feature = "skia")]
#[derive(Clone, Debug, Copy, Default)]
pub enum ImageFormat {
    Bmp,
    Gif,
    Ico,
    Jpeg,
    #[default]
    Png,
    Wbmp,
    Webp,
    Pkm,
    Ktx,
    Astc,
    Dng,
    Heif,
    Avif,
    Jpegxl,
}
#[cfg(feature = "skia")]
impl From<&ImageFormat> for skia_safe::EncodedImageFormat {
    fn from(value: &ImageFormat) -> Self {
        match value {
            ImageFormat::Bmp => skia_safe::EncodedImageFormat::BMP,
            ImageFormat::Gif => skia_safe::EncodedImageFormat::GIF,
            ImageFormat::Ico => skia_safe::EncodedImageFormat::ICO,
            ImageFormat::Jpeg => skia_safe::EncodedImageFormat::JPEG,
            ImageFormat::Png => skia_safe::EncodedImageFormat::PNG,
            ImageFormat::Wbmp => skia_safe::EncodedImageFormat::WBMP,
            ImageFormat::Webp => skia_safe::EncodedImageFormat::WEBP,
            ImageFormat::Pkm => skia_safe::EncodedImageFormat::PKM,
            ImageFormat::Ktx => skia_safe::EncodedImageFormat::KTX,
            ImageFormat::Astc => skia_safe::EncodedImageFormat::ASTC,
            ImageFormat::Dng => skia_safe::EncodedImageFormat::DNG,
            ImageFormat::Heif => skia_safe::EncodedImageFormat::HEIF,
            ImageFormat::Avif => skia_safe::EncodedImageFormat::AVIF,
            ImageFormat::Jpegxl => skia_safe::EncodedImageFormat::JPEGXL,
        }
    }
}
#[cfg(feature = "skia")]
fn make_paint(p1: f32, p2: &Color, alias: bool, fill: bool) -> skia_safe::Paint {
    let mut p = skia_safe::Paint::new(p2.to_col(), None);
    p.set_stroke_width(p1);
    p.set_style(skia_safe::PaintStyle::Stroke);
    if fill {
        p.set_stroke_cap(skia_safe::PaintCap::Square);
    }
    p.set_anti_alias(alias);
    p
}
#[cfg(feature = "tiny-skia")]
fn make_paint(p2: &Color, alias: bool) -> tiny_skia::Paint {
    let mut p = tiny_skia::Paint::default();
    p.set_color(p2.to_col());
    p.anti_alias = alias;
    p
}
struct FastLine {
    #[cfg(feature = "skia")]
    line: std::collections::HashMap<Color, skia_safe::Path>,
    #[cfg(feature = "tiny-skia")]
    line: std::collections::HashMap<Color, tiny_skia::PathBuilder>,
    #[cfg(feature = "egui")]
    line: std::collections::HashMap<Color, Vec<egui::Pos2>>,
    #[cfg(feature = "egui")]
    size: usize,
    line_width: f32,
}
struct SlowLine {
    #[cfg(feature = "skia")]
    line: skia_safe::Path,
    #[cfg(any(feature = "skia", feature = "tiny-skia"))]
    last: Option<Pos>,
    #[cfg(feature = "tiny-skia")]
    line: tiny_skia::PathBuilder,
    #[cfg(feature = "egui")]
    line: Vec<egui::Pos2>,
    #[cfg(feature = "egui")]
    size: usize,
    color: Color,
    line_width: f32,
}
enum Line {
    Fast(FastLine),
    Slow(SlowLine),
    #[cfg(feature = "tiny-skia")]
    None,
}
#[cfg(feature = "skia")]
impl Line {
    fn line(&mut self, p0: [Pos; 2], p2: &Color, canvas: Option<&skia_safe::Canvas>) {
        match self {
            Line::Fast(FastLine { line, .. }) => {
                let path = line.entry(*p2).or_insert({
                    let mut path = skia_safe::Path::new();
                    path.move_to(p0[0].to_pos2());
                    path
                });
                let last = path.last_pt().unwrap();
                let last = Pos::new(last.x, last.y);
                let a = !p0[0].close(last);
                let b = !p0[1].close(last);
                if b {
                    if a {
                        path.move_to(p0[0].to_pos2());
                    }
                    path.line_to(p0[1].to_pos2());
                } else if a {
                    path.line_to(p0[0].to_pos2());
                }
            }
            Line::Slow(SlowLine {
                line,
                last,
                color,
                line_width,
            }) => {
                if p2 == color && last.map(|l| p0[0].close(l)).unwrap_or(false) {
                    if !last.unwrap().close(p0[1]) {
                        line.line_to(p0[1].to_pos2());
                    }
                } else {
                    if last.is_some() {
                        canvas
                            .unwrap()
                            .draw_path(line, &make_paint(*line_width, color, true, false));
                    }
                    *line = skia_safe::Path::new();
                    line.move_to(p0[0].to_pos2());
                    line.line_to(p0[1].to_pos2());
                    *color = *p2;
                }
                *last = Some(p0[1])
            }
        }
    }
    fn draw(&self, canvas: &skia_safe::Canvas, anti_alias: bool) {
        match self {
            Line::Fast(FastLine { line, line_width }) => {
                for (color, path) in line {
                    canvas.draw_path(path, &make_paint(*line_width, color, anti_alias, false));
                }
            }
            Line::Slow(SlowLine {
                line,
                color,
                line_width,
                ..
            }) => {
                if line.last_pt().is_some() {
                    canvas.draw_path(line, &make_paint(*line_width, color, anti_alias, false));
                }
            }
        }
    }
    fn clear(&mut self) {
        match self {
            Line::Fast(FastLine { line, .. }) => line.clear(),
            Line::Slow(SlowLine { line, .. }) => {
                let last = line.last_pt();
                *line = skia_safe::Path::new();
                if let Some(last) = last {
                    line.move_to(last);
                }
            }
        }
    }
}
#[cfg(feature = "tiny-skia")]
impl Line {
    fn line(&mut self, p0: [Pos; 2], p2: &Color, canvas: Option<&mut tiny_skia::Pixmap>) {
        match self {
            Line::Fast(FastLine { line, .. }) => {
                let path = line.entry(*p2).or_insert({
                    let mut path = tiny_skia::PathBuilder::new();
                    path.move_to(p0[0].x, p0[0].y);
                    path
                });
                let last = path.last_point().unwrap();
                let last = Pos::new(last.x, last.y);
                let a = !p0[0].close(last);
                let b = !p0[1].close(last);
                if b {
                    if a {
                        path.move_to(p0[0].x, p0[0].y);
                    }
                    path.line_to(p0[1].x, p0[1].y);
                } else if a {
                    path.line_to(p0[0].x, p0[0].y);
                }
            }
            Line::Slow(SlowLine {
                line,
                last,
                color,
                line_width,
            }) => {
                if p2 == color && last.map(|l| p0[0].close(l)).unwrap_or(false) {
                    if !last.unwrap().close(p0[1]) {
                        line.line_to(p0[1].x, p0[1].y);
                    }
                } else {
                    if last.is_some() {
                        let path = std::mem::take(line).finish().unwrap();
                        let stroke = tiny_skia::Stroke {
                            width: *line_width,
                            ..Default::default()
                        };
                        canvas.unwrap().stroke_path(
                            &path,
                            &make_paint(color, true),
                            &stroke,
                            tiny_skia::Transform::default(),
                            None,
                        );
                    }
                    line.move_to(p0[0].x, p0[0].y);
                    line.line_to(p0[1].x, p0[1].y);
                    *color = *p2;
                }
                *last = Some(p0[1])
            }
            Line::None => {}
        }
    }
    fn draw(self, canvas: &mut tiny_skia::Pixmap, anti_alias: bool) {
        match self {
            Line::Fast(FastLine { line, line_width }) => {
                for (color, path) in line {
                    let path = path.finish().unwrap();
                    let stroke = tiny_skia::Stroke {
                        width: line_width,
                        ..Default::default()
                    };
                    canvas.stroke_path(
                        &path,
                        &make_paint(&color, anti_alias),
                        &stroke,
                        tiny_skia::Transform::default(),
                        None,
                    );
                }
            }
            Line::Slow(SlowLine {
                line,
                color,
                line_width,
                ..
            }) => {
                if line.last_point().is_some() {
                    let stroke = tiny_skia::Stroke {
                        width: line_width,
                        ..Default::default()
                    };
                    let path = line.finish().unwrap();
                    canvas.stroke_path(
                        &path,
                        &make_paint(&color, anti_alias),
                        &stroke,
                        tiny_skia::Transform::default(),
                        None,
                    );
                }
            }
            Line::None => {}
        }
    }
}
#[cfg(feature = "egui")]
impl Line {
    fn line(&mut self, p0: [Pos; 2], p2: &Color, painter: &egui::Painter) {
        match self {
            Line::Fast(FastLine {
                line,
                size,
                line_width,
            }) => {
                let line = line.entry(*p2).or_insert_with(|| {
                    let mut vec = Vec::with_capacity(*size);
                    vec.push(p0[0].to_pos2());
                    vec
                });
                let last = line.last().unwrap();
                let last = Pos::new(last.x, last.y);
                if last.close(p0[0]) {
                    if !last.close(p0[1]) {
                        line.push(p0[1].to_pos2());
                    }
                } else {
                    painter.line(
                        std::mem::replace(line, Vec::with_capacity(*size)),
                        egui::Stroke::new(*line_width, p2.to_col()),
                    );
                    line.clear();
                    line.push(p0[0].to_pos2());
                    line.push(p0[1].to_pos2());
                }
            }
            Line::Slow(SlowLine {
                line,
                color,
                size,
                line_width,
            }) => {
                let last = line.last().map(|l| Pos::new(l.x, l.y));
                if p2 == color && last.map(|l| l.close(p0[0])).unwrap_or(false) {
                    if !last.unwrap().close(p0[1]) {
                        line.push(p0[1].to_pos2());
                    }
                } else {
                    if !line.is_empty() {
                        painter.line(
                            std::mem::replace(line, Vec::with_capacity(*size)),
                            egui::Stroke::new(*line_width, color.to_col()),
                        );
                    }
                    line.push(p0[0].to_pos2());
                    line.push(p0[1].to_pos2());
                    *color = *p2
                }
            }
        }
    }
    fn draw(&mut self, painter: &egui::Painter) {
        match self {
            Line::Fast(FastLine {
                line, line_width, ..
            }) => {
                for (color, line) in line {
                    painter.line(
                        std::mem::take(line),
                        egui::Stroke::new(*line_width, color.to_col()),
                    );
                }
            }
            Line::Slow(SlowLine {
                line,
                color,
                line_width,
                ..
            }) => {
                painter.line(
                    std::mem::take(line),
                    egui::Stroke::new(*line_width, color.to_col()),
                );
            }
        }
    }
}
#[cfg(feature = "skia")]
struct FastPoint {
    points: std::collections::HashMap<Color, Vec<skia_safe::Point>>,
    size: usize,
}
#[cfg(feature = "skia")]
struct SlowPoint {
    points: Vec<skia_safe::Point>,
    color: Color,
}
#[cfg(feature = "skia")]
enum Point {
    Fast(FastPoint),
    Slow(SlowPoint),
}
#[cfg(feature = "skia")]
impl Point {
    fn point(&mut self, p0: Pos, p2: &Color, canvas: Option<&skia_safe::Canvas>) {
        match self {
            Point::Fast(FastPoint { points, size }) => {
                points
                    .entry(*p2)
                    .or_insert(Vec::with_capacity(*size))
                    .push(p0.to_pos2());
            }
            Point::Slow(SlowPoint { points, color }) => {
                if p2 != color || points.is_empty() {
                    if !points.is_empty() {
                        canvas.unwrap().draw_points(
                            skia_safe::canvas::PointMode::Points,
                            points,
                            &make_paint(5.0, color, true, true),
                        );
                        points.clear();
                    }
                    *color = *p2
                }
                if points
                    .last()
                    .map(|p| !p0.close(Pos::new(p.x, p.y)))
                    .unwrap_or(true)
                {
                    points.push(p0.to_pos2())
                }
            }
        }
    }
    fn draw(&self, canvas: &skia_safe::Canvas) {
        match self {
            Point::Fast(FastPoint { points, .. }) => {
                for (color, points) in points {
                    canvas.draw_points(
                        skia_safe::canvas::PointMode::Points,
                        points,
                        &make_paint(5.0, color, true, true),
                    );
                }
            }
            Point::Slow(SlowPoint { points, color }) => {
                if !points.is_empty() {
                    canvas.draw_points(
                        skia_safe::canvas::PointMode::Points,
                        points,
                        &make_paint(5.0, color, true, true),
                    );
                }
            }
        }
    }
    fn clear(&mut self) {
        match self {
            Point::Fast(FastPoint { points, .. }) => points.clear(),
            Point::Slow(SlowPoint { points, .. }) => points.clear(),
        }
    }
}
