use crate::types::{Align, Color, Image, Pos, Vec2};
#[cfg(feature = "egui")]
pub(crate) struct Painter<'a> {
    painter: &'a egui::Painter,
    line: Line,
}
#[cfg(feature = "egui")]
impl<'a> Painter<'a> {
    pub(crate) fn new(ui: &'a egui::Ui, fast: bool) -> Self {
        Self {
            painter: ui.painter(),
            line: if fast {
                Line::Fast(FastLine {
                    line: Default::default(),
                })
            } else {
                Line::Slow(SlowLine {
                    line: Default::default(),
                    color: Color::new(0, 0, 0),
                })
            },
        }
    }
    pub(crate) fn line_segment(&mut self, p0: [Pos; 2], p1: f32, p2: &Color) {
        if p1 != 1.0 {
            self.painter
                .line_segment(p0.map(|l| l.to_pos2()), egui::Stroke::new(p1, p2.to_col()));
        } else {
            self.line.line(p0, p2, self.painter)
        }
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
        self.painter.hline(
            egui::Rangef::new(0.0, p0),
            p1,
            egui::Stroke::new(p2, p3.to_col()),
        );
    }
    pub(crate) fn vline(&self, p0: f32, p1: f32, p2: f32, p3: &Color) {
        self.painter.vline(
            p0,
            egui::Rangef::new(0.0, p1),
            egui::Stroke::new(p2, p3.to_col()),
        );
    }
    pub(crate) fn text(&self, p0: Pos, p1: Align, p2: String, p4: &Color) {
        self.painter.text(
            p0.to_pos2(),
            p1.into(),
            p2,
            egui::FontId::monospace(16.0),
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
}
#[cfg(feature = "skia")]
impl Painter {
    pub(crate) fn new(
        width: u32,
        height: u32,
        background: Color,
        font: skia_safe::Font,
        fast: bool,
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
                })
            } else {
                Point::Slow(SlowPoint {
                    points: Default::default(),
                    color: Color::new(0, 0, 0),
                })
            },
            font,
            fast,
        }
    }
    pub(crate) fn line_segment(&mut self, p0: [Pos; 2], p1: f32, p2: &Color) {
        if p1 != 1.0 {
            self.canvas.canvas().draw_line(
                p0[0].to_pos2(),
                p0[1].to_pos2(),
                &make_paint(p1, p2, true, false),
            );
        } else {
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
    }
    fn draw(&mut self) {
        self.line.draw(self.canvas.canvas());
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
        self.canvas.canvas().draw_image(p0, pos.to_pos2(), None);
    }
    pub(crate) fn hline(&mut self, p0: f32, p1: f32, p2: f32, p3: &Color) {
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
    pub(crate) fn vline(&mut self, p0: f32, p1: f32, p2: f32, p3: &Color) {
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
    pub(crate) fn text(&mut self, p0: Pos, p1: Align, p2: String, p4: &Color) {
        let mut pos = p0.to_pos2();
        pos.x += 2.0;
        pos.y -= 2.0;
        self.canvas.canvas().draw_str_align(
            p2,
            pos,
            &self.font,
            &make_paint(1.0, p4, false, false),
            p1.into(),
        );
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
struct FastLine {
    #[cfg(feature = "skia")]
    line: std::collections::HashMap<Color, skia_safe::Path>,
    #[cfg(feature = "egui")]
    line: std::collections::HashMap<Color, Vec<egui::Pos2>>,
}
struct SlowLine {
    #[cfg(feature = "skia")]
    line: skia_safe::Path,
    #[cfg(feature = "egui")]
    line: Vec<egui::Pos2>,
    color: Color,
    #[cfg(feature = "skia")]
    last: Option<Pos>,
}
enum Line {
    Fast(FastLine),
    Slow(SlowLine),
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
    fn draw(&self, canvas: &skia_safe::Canvas) {
        match self {
            Line::Fast(FastLine { line }) => {
                for (color, path) in line {
                    canvas.draw_path(path, &make_paint(1.0, color, true, false));
                }
            }
            Line::Slow(SlowLine { line, color, .. }) => {
                if line.last_pt().is_some() {
                    canvas.draw_path(line, &make_paint(1.0, color, true, false));
                }
            }
        }
    }
}
#[cfg(feature = "egui")]
impl Line {
    fn line(&mut self, p0: [Pos; 2], p2: &Color, painter: &egui::Painter) {
        match self {
            Line::Fast(FastLine { line }) => {
                let line = line.entry(*p2).or_insert(vec![p0[0].to_pos2()]);
                let last = line.last().unwrap();
                let last = Pos::new(last.x, last.y);
                if last.close(p0[0]) {
                    if !last.close(p0[1]) {
                        line.push(p0[1].to_pos2());
                    }
                } else {
                    painter.line(std::mem::take(line), egui::Stroke::new(1.0, p2.to_col()));
                    line.push(p0[0].to_pos2());
                    line.push(p0[1].to_pos2());
                }
            }
            Line::Slow(SlowLine { line, color }) => {
                let last = line.last().map(|l| Pos::new(l.x, l.y));
                if p2 == color && last.map(|l| l.close(p0[0])).unwrap_or(false) {
                    if !last.unwrap().close(p0[1]) {
                        line.push(p0[1].to_pos2());
                    }
                } else {
                    if !line.is_empty() {
                        painter.line(std::mem::take(line), egui::Stroke::new(1.0, color.to_col()));
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
            Line::Fast(FastLine { line }) => {
                for (color, line) in line {
                    painter.line(std::mem::take(line), egui::Stroke::new(1.0, color.to_col()));
                }
            }
            Line::Slow(SlowLine { line, color }) => {
                painter.line(std::mem::take(line), egui::Stroke::new(1.0, color.to_col()));
            }
        }
    }
}
#[cfg(feature = "skia")]
struct FastPoint {
    points: std::collections::HashMap<Color, Vec<skia_safe::Point>>,
}
#[cfg(feature = "skia")]
struct SlowPoint {
    points: Vec<skia_safe::Point>, //TODO with_capacity, disable fast lines in 2d
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
            Point::Fast(FastPoint { points }) => {
                points.entry(*p2).or_default().push(p0.to_pos2());
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
            Point::Fast(FastPoint { points }) => {
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
