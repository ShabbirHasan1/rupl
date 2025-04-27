use crate::types::{Color, Image, Pos, Vec2};
#[cfg(feature = "egui")]
pub(crate) struct Painter<'a> {
    painter: &'a egui::Painter,
    line: Line,
}
#[cfg(feature = "egui")]
impl<'a> Painter<'a> {
    pub(crate) fn new(ui: &'a egui::Ui, fast: bool, size: usize) -> Self {
        Self {
            painter: ui.painter(),
            line: if fast {
                Line::Fast(FastLine {
                    line: Default::default(),
                    size,
                })
            } else {
                Line::Slow(SlowLine {
                    line: Vec::with_capacity(size),
                    color: Color::new(0, 0, 0),
                    size,
                })
            },
        }
    }
    pub(crate) fn line_segment(&mut self, p0: [Pos; 2], p2: &Color) {
        self.line.line(p0, p2, self.painter)
    }
    pub(crate) fn save(&mut self) {
        self.line.draw(self.painter)
    }
    pub(crate) fn rect_filled(&self, p0: Pos, p2: &Color) {
        let rect = egui::Rect::from_center_size(p0.to_pos2(), egui::Vec2::splat(3.0));
        self.painter.rect_filled(rect, 0.0, p2.to_col());
    }
    pub(crate) fn image(&self, p0: &Image, pos: Vec2) {
        let d = egui::Rect::from_points(&[egui::Pos2::new(0.0, 0.0), pos.to_pos2()]);
        let a = egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0));
        let c = egui::Color32::WHITE;
        self.painter.image(p0.0.id(), d, a, c);
    }
    pub(crate) fn hline(&self, p0: f32, p1: f32, p2: f32, p3: &Color) {
        if p1.is_finite() {
            self.painter.hline(
                egui::Rangef::new(0.0, p0),
                p1,
                egui::Stroke::new(p2, p3.to_col()),
            );
        }
    }
    pub(crate) fn vline(&self, p0: f32, p1: f32, p2: f32, p3: &Color) {
        if p0.is_finite() {
            self.painter.vline(
                p0,
                egui::Rangef::new(0.0, p1),
                egui::Stroke::new(p2, p3.to_col()),
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
    ) {
        self.painter.text(
            p0.to_pos2(),
            p1.into(),
            p2,
            egui::FontId::monospace(font_size),
            p4.to_col(),
        );
    }
}
#[cfg(feature = "skia")]
pub(crate) struct Painter {
    canvas: skia_safe::Surface,
    line: Line,
    font: skia_safe::Font,
    points: Point,
    fast: bool,
    anti_alias: bool,
}
#[cfg(feature = "skia")]
impl Painter {
    pub(crate) fn new(
        width: u32,
        height: u32,
        background: Color,
        font: skia_safe::Font,
        fast: bool,
        size: usize,
        anti_alias: bool,
    ) -> Self {
        let mut canvas =
            skia_safe::surfaces::raster_n32_premul((width as i32, height as i32)).unwrap();
        canvas.canvas().clear(background.to_col());
        Self {
            canvas,
            line: if fast {
                Line::Fast(FastLine {
                    line: Default::default(),
                })
            } else {
                Line::Slow(SlowLine {
                    line: Default::default(),
                    color: Color::new(0, 0, 0),
                    last: None,
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
        }
    }
    pub(crate) fn line_segment(&mut self, p0: [Pos; 2], p2: &Color) {
        self.line.line(
            p0,
            p2,
            if self.fast {
                None
            } else {
                Some(self.canvas.canvas())
            },
        );
    }
    fn draw(&mut self) {
        self.line.draw(self.canvas.canvas(), self.anti_alias);
    }
    fn draw_pts(&mut self) {
        self.points.draw(self.canvas.canvas());
    }
    #[cfg(not(feature = "skia-png"))]
    pub(crate) fn save<T>(&mut self, buffer: &mut T)
    where
        T: std::ops::DerefMut<Target = [u32]>,
    {
        self.draw();
        self.draw_pts();
        if let Some(pm) = self.canvas.peek_pixels() {
            let px = pm.pixels::<u32>().unwrap();
            buffer.copy_from_slice(px);
        }
    }
    #[cfg(feature = "skia-png")]
    pub(crate) fn save(&mut self, format: &ImageFormat) -> Data {
        self.draw();
        self.draw_pts();
        Data {
            data: self
                .canvas
                .image_snapshot()
                .encode(None, format.into(), None)
                .unwrap(),
        }
    }
    pub(crate) fn rect_filled(&mut self, p0: Pos, p2: &Color) {
        self.points.point(
            p0,
            p2,
            if self.fast {
                None
            } else {
                Some(self.canvas.canvas())
            },
        );
    }
    pub(crate) fn image(&mut self, p0: &Image, pos: Vec2) {
        let mut paint = skia_safe::Paint::default();
        paint.set_anti_alias(self.anti_alias);
        self.canvas.canvas().draw_image_rect(
            p0,
            None,
            skia_safe::Rect::new(0.0, 0.0, pos.x as f32, pos.y as f32),
            &paint,
        );
    }
    pub(crate) fn hline(&mut self, p0: f32, p1: f32, p2: f32, p3: &Color) {
        if p1.is_finite() {
            if p2 != 1.0 {
                self.canvas.canvas().draw_line(
                    Pos::new(0.0, p1).to_pos2(),
                    Pos::new(p0, p1).to_pos2(),
                    &make_paint(p2, p3, false, false),
                );
            } else {
                self.line.line(
                    [Pos::new(0.0, p1), Pos::new(p0, p1)],
                    p3,
                    if self.fast {
                        None
                    } else {
                        Some(self.canvas.canvas())
                    },
                );
            }
        }
    }
    pub(crate) fn vline(&mut self, p0: f32, p1: f32, p2: f32, p3: &Color) {
        if p0.is_finite() {
            if p2 != 1.0 {
                self.canvas.canvas().draw_line(
                    Pos::new(p0, 0.0).to_pos2(),
                    Pos::new(p0, p1).to_pos2(),
                    &make_paint(p2, p3, false, false),
                );
            } else {
                self.line.line(
                    [Pos::new(p0, 0.0), Pos::new(p0, p1)],
                    p3,
                    if self.fast {
                        None
                    } else {
                        Some(self.canvas.canvas())
                    },
                );
            }
        }
    }
    pub(crate) fn text(&mut self, p0: Pos, p1: crate::types::Align, p2: &str, p4: &Color) {
        let mut pos = p0.to_pos2();
        let paint = make_paint(1.0, p4, false, false);
        let strs = p2.split('\n').collect::<Vec<&str>>();
        let mut body = |s: &str| {
            let rect = self.font.measure_str(s, Some(&paint)).1;
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
                self.canvas.canvas().draw_str(s, pos, &self.font, &paint);
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
    }
}
#[cfg(feature = "tiny-skia")]
pub(crate) struct Painter {
    canvas: tiny_skia::Pixmap,
    line: Line,
    fast: bool,
    anti_alias: bool,
}
#[cfg(feature = "tiny-skia")]
impl Painter {
    pub(crate) fn new(
        width: u32,
        height: u32,
        background: Color,
        fast: bool,
        anti_alias: bool,
    ) -> Self {
        let mut canvas = tiny_skia::Pixmap::new(width, height).unwrap();
        canvas.fill(background.to_col());
        Self {
            canvas,
            line: if fast {
                Line::Fast(FastLine {
                    line: Default::default(),
                })
            } else {
                Line::Slow(SlowLine {
                    line: Default::default(),
                    color: Color::new(0, 0, 0),
                    last: None,
                })
            },
            fast,
            anti_alias,
        }
    }
    pub(crate) fn line_segment(&mut self, p0: [Pos; 2], p2: &Color) {
        self.line.line(
            p0,
            p2,
            if self.fast {
                None
            } else {
                Some(&mut self.canvas)
            },
        );
    }
    fn draw(&mut self) {
        std::mem::replace(&mut self.line, Line::None).draw(&mut self.canvas, self.anti_alias);
    }
    #[cfg(not(feature = "skia-png"))]
    pub(crate) fn save<T>(&mut self, buffer: &mut T)
    where
        T: std::ops::DerefMut<Target = [u32]>,
    {
        self.draw();
        for (dst, p) in buffer.iter_mut().zip(self.canvas.pixels()) {
            *dst = (p.red() as u32) << 16 | (p.green() as u32) << 8 | p.blue() as u32;
        }
    }
    #[cfg(feature = "skia-png")]
    pub(crate) fn save(&mut self, format: &ImageFormat) -> Data {
        self.draw(); //TODO
        self.draw_pts();
        Data {
            data: self
                .canvas
                .image_snapshot()
                .encode(None, format.into(), None)
                .unwrap(),
        }
    }
    pub(crate) fn rect_filled(&mut self, p0: Pos, p2: &Color) {
        self.canvas.fill_rect(
            tiny_skia::Rect::from_ltrb(p0.x - 1.0, p0.y - 1.0, p0.x + 1.0, p0.y + 1.0).unwrap(),
            &make_paint(3.0, p2, false, true),
            tiny_skia::Transform::default(),
            None,
        );
    }
    pub(crate) fn image(&mut self, p0: &Image, _pos: Vec2) {
        self.canvas.draw_pixmap(
            0,
            0,
            p0.0.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            tiny_skia::Transform::default(),
            None,
        );
    }
    pub(crate) fn hline(&mut self, p0: f32, p1: f32, p2: f32, p3: &Color) {
        if p1.is_finite() {
            if p2 != 1.0 {
                let mut path = tiny_skia::PathBuilder::new();
                path.move_to(0.0, p1);
                path.line_to(p0, p1);
                let path = path.finish().unwrap();
                self.canvas.stroke_path(
                    &path,
                    &make_paint(p2, p3, false, false),
                    &tiny_skia::Stroke::default(),
                    tiny_skia::Transform::default(),
                    None,
                );
            } else {
                self.line.line(
                    [Pos::new(0.0, p1), Pos::new(p0, p1)],
                    p3,
                    if self.fast {
                        None
                    } else {
                        Some(&mut self.canvas)
                    },
                );
            }
        }
    }
    pub(crate) fn vline(&mut self, p0: f32, p1: f32, p2: f32, p3: &Color) {
        if p0.is_finite() {
            if p2 != 1.0 {
                let mut path = tiny_skia::PathBuilder::new();
                path.move_to(p0, 0.0);
                path.line_to(p0, p1);
                let path = path.finish().unwrap();
                self.canvas.stroke_path(
                    &path,
                    &make_paint(p2, p3, false, false),
                    &tiny_skia::Stroke::default(),
                    tiny_skia::Transform::default(),
                    None,
                );
            } else {
                self.line.line(
                    [Pos::new(p0, 0.0), Pos::new(p0, p1)],
                    p3,
                    if self.fast {
                        None
                    } else {
                        Some(&mut self.canvas)
                    },
                );
            }
        }
    }
}
#[cfg(feature = "skia-png")]
pub struct Data {
    data: skia_safe::Data,
}
#[cfg(feature = "skia-png")]
impl Data {
    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_bytes()
    }
}
#[cfg(feature = "skia-png")]
pub enum ImageFormat {
    Bmp,
    Gif,
    Ico,
    Jpeg,
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
#[cfg(feature = "skia-png")]
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
fn make_paint(_p1: f32, p2: &Color, alias: bool, _fill: bool) -> tiny_skia::Paint {
    let mut p = tiny_skia::Paint::default();
    p.set_color(p2.to_col());
    p.anti_alias = alias;
    /*p.set_stroke_width(p1);
    p.set_style(tiny_skia::PaintStyle::Stroke);
    if fill {
        p.set_stroke_cap(tiny_skia::PaintCap::Square);
    }
    p.set_anti_alias(alias);*/
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
            Line::Fast(FastLine { line }) => {
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
            Line::Slow(SlowLine { line, last, color }) => {
                if p2 == color && last.map(|l| p0[0].close(l)).unwrap_or(false) {
                    if !last.unwrap().close(p0[1]) {
                        line.line_to(p0[1].to_pos2());
                    }
                } else {
                    if last.is_some() {
                        canvas
                            .unwrap()
                            .draw_path(line, &make_paint(1.0, color, true, false));
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
            Line::Fast(FastLine { line }) => {
                for (color, path) in line {
                    canvas.draw_path(path, &make_paint(1.0, color, anti_alias, false));
                }
            }
            Line::Slow(SlowLine { line, color, .. }) => {
                if line.last_pt().is_some() {
                    canvas.draw_path(line, &make_paint(1.0, color, anti_alias, false));
                }
            }
        }
    }
}
#[cfg(feature = "tiny-skia")]
impl Line {
    fn line(&mut self, p0: [Pos; 2], p2: &Color, canvas: Option<&mut tiny_skia::Pixmap>) {
        match self {
            Line::Fast(FastLine { line }) => {
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
            Line::Slow(SlowLine { line, last, color }) => {
                if p2 == color && last.map(|l| p0[0].close(l)).unwrap_or(false) {
                    if !last.unwrap().close(p0[1]) {
                        line.line_to(p0[1].x, p0[1].y);
                    }
                } else {
                    if last.is_some() {
                        let path = std::mem::take(line).finish().unwrap();
                        canvas.unwrap().stroke_path(
                            &path,
                            &make_paint(1.0, color, true, false),
                            &tiny_skia::Stroke::default(),
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
            Line::Fast(FastLine { line }) => {
                for (color, path) in line {
                    let path = path.finish().unwrap();
                    canvas.stroke_path(
                        &path,
                        &make_paint(1.0, &color, anti_alias, false),
                        &tiny_skia::Stroke::default(),
                        tiny_skia::Transform::default(),
                        None,
                    );
                }
            }
            Line::Slow(SlowLine { line, color, .. }) => {
                if line.last_point().is_some() {
                    let path = line.finish().unwrap();
                    canvas.stroke_path(
                        &path,
                        &make_paint(1.0, &color, anti_alias, false),
                        &tiny_skia::Stroke::default(),
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
            Line::Fast(FastLine { line, size }) => {
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
                        egui::Stroke::new(1.0, p2.to_col()),
                    );
                    line.clear();
                    line.push(p0[0].to_pos2());
                    line.push(p0[1].to_pos2());
                }
            }
            Line::Slow(SlowLine { line, color, size }) => {
                let last = line.last().map(|l| Pos::new(l.x, l.y));
                if p2 == color && last.map(|l| l.close(p0[0])).unwrap_or(false) {
                    if !last.unwrap().close(p0[1]) {
                        line.push(p0[1].to_pos2());
                    }
                } else {
                    if !line.is_empty() {
                        painter.line(
                            std::mem::replace(line, Vec::with_capacity(*size)),
                            egui::Stroke::new(1.0, color.to_col()),
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
            Line::Fast(FastLine { line, .. }) => {
                for (color, line) in line {
                    painter.line(std::mem::take(line), egui::Stroke::new(1.0, color.to_col()));
                }
            }
            Line::Slow(SlowLine { line, color, .. }) => {
                painter.line(std::mem::take(line), egui::Stroke::new(1.0, color.to_col()));
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
                            &make_paint(3.0, color, false, true),
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
                        &make_paint(3.0, color, false, true),
                    );
                }
            }
            Point::Slow(SlowPoint { points, color }) => {
                if !points.is_empty() {
                    canvas.draw_points(
                        skia_safe::canvas::PointMode::Points,
                        points,
                        &make_paint(3.0, color, false, true),
                    );
                }
            }
        }
    }
}
