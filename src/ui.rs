use crate::types::{Align, Color, Pos, Texture, Vec2};
#[cfg(feature = "egui")]
pub(crate) struct Painter<'a> {
    painter: &'a egui::Painter,
    line: Vec<egui::Pos2>,
    color: Option<Color>,
    last: Option<Pos>,
    width: f32,
}
#[cfg(feature = "egui")]
impl<'a> Painter<'a> {
    pub(crate) fn new(ui: &'a egui::Ui) -> Self {
        Self {
            painter: ui.painter(),
            line: Vec::new(),
            color: None,
            last: None,
            width: 0.0,
        }
    }
    pub(crate) fn line_segment(&mut self, p0: [Pos; 2], p1: f32, p2: &Color) {
        if Some(*p2) == self.color && Some(p0[0]) == self.last && self.width == p1 {
            self.line.push(p0[1].to_pos2())
        } else {
            self.save();
            self.width = p1;
            self.line.push(p0[0].to_pos2());
            self.line.push(p0[1].to_pos2());
            self.color = Some(*p2)
        }
    }
    pub(crate) fn save(&mut self) {
        if let Some(col) = self.color {
            self.painter.line(
                std::mem::take(&mut self.line),
                egui::Stroke::new(self.width, col.to_col()),
            );
        }
    }
    pub(crate) fn rect_filled(&self, p0: Pos, p1: f32, p2: &Color) {
        let rect = egui::Rect::from_center_size(p0.to_pos2(), egui::Vec2::splat(3.0));
        self.painter.rect_filled(rect, p1, p2.to_col());
    }
    pub(crate) fn image(&self, p0: Texture, pos: Vec2) {
        let d = egui::Rect::from_points(&[egui::Pos2::new(0.0, 0.0), pos.to_pos2()]);
        let a = egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0));
        let c = egui::Color32::WHITE;
        self.painter.image(p0.texture, d, a, c);
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
    line: skia_safe::Path,
    paint: Option<skia_safe::Paint>,
    color: Option<Color>,
    last: Option<Pos>,
    width: f32,
}
#[cfg(feature = "skia")]
impl Painter {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        let canvas = skia_safe::surfaces::raster_n32_premul((width as i32, height as i32)).unwrap();
        Self {
            canvas,
            line: skia_safe::Path::new(),
            paint: None,
            color: None,
            last: None,
            width: 0.0,
        }
    }
    pub(crate) fn line_segment(&mut self, p0: [Pos; 2], p1: f32, p2: &Color) {
        let c = Some(*p2) == self.color;
        if c && Some(p0[0]) == self.last && self.width == p1 {
            self.line.line_to(p0[1].to_pos2());
        } else {
            self.save();
            self.width = p1;
            self.line = skia_safe::Path::new();
            self.line.move_to(p0[0].to_pos2());
            self.line.line_to(p0[1].to_pos2());
            if !c || self.width != p1 {
                self.color = Some(*p2);
                let mut p = skia_safe::Paint::new(p2.to_col(), None);
                p.set_stroke_width(p1);
                self.paint = Some(p)
            }
        }
    }
    pub(crate) fn save(&mut self) -> Vec<u8> {
        if let Some(paint) = &self.paint {
            self.canvas.canvas().draw_path(&self.line, paint);
        }
        let info = self.canvas.image_info();
        let size = info.width() as usize * info.height() as usize * 16;
        let mut pixels = vec![0; size];
        self.canvas.read_pixels(
            &info,
            &mut pixels,
            info.width() as usize * 4,
            skia_safe::IPoint::new(0, 0),
        );
        pixels
    }
    pub(crate) fn rect_filled(&mut self, p0: Pos, p1: f32, p2: &Color) {}
    pub(crate) fn image(&mut self, p0: Texture, pos: Vec2) {}
    pub(crate) fn hline(&mut self, p0: f32, p1: f32, p2: f32, p3: &Color) {}
    pub(crate) fn vline(&mut self, p0: f32, p1: f32, p2: f32, p3: &Color) {}
    pub(crate) fn text(&mut self, p0: Pos, p1: Align, p2: String, p4: &Color) {}
}
