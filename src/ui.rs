use crate::types::{Align, Color, Image, Pos, Vec2};
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
            if !self.last.unwrap().close(p0[1]) {
                self.line.push(p0[1].to_pos2());
                self.last = Some(p0[1]);
            }
        } else {
            self.save();
            self.width = p1;
            self.line.push(p0[0].to_pos2());
            self.line.push(p0[1].to_pos2());
            self.last = Some(p0[1]);
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
    line: skia_safe::Path, //TODO make vec
    color: Option<Color>,
    last: Option<Pos>,
    width: f32,
    font: skia_safe::Font,
    points: Vec<skia_safe::Point>,
    point_color: Option<Color>,
}
#[cfg(feature = "skia")]
impl Painter {
    pub(crate) fn new(width: u32, height: u32, background: Color, font: skia_safe::Font) -> Self {
        let mut canvas =
            skia_safe::surfaces::raster_n32_premul((width as i32, height as i32)).unwrap();
        canvas.canvas().clear(background.to_col());
        Self {
            canvas,
            line: skia_safe::Path::new(),
            points: Vec::new(),
            point_color: None,
            color: None,
            last: None,
            width: 0.0,
            font,
        }
    }
    pub(crate) fn line_segment(&mut self, p0: [Pos; 2], p1: f32, p2: &Color) {
        if Some(*p2) == self.color
            && self.last.map(|l| l.close(p0[0])).unwrap_or(false)
            && self.width == p1
        {
            if !self.last.unwrap().close(p0[1]) {
                self.line.line_to(p0[1].to_pos2());
                self.last = Some(p0[1]);
            }
        } else {
            self.draw();
            self.width = p1;
            self.line = skia_safe::Path::new();
            self.line.move_to(p0[0].to_pos2());
            self.line.line_to(p0[1].to_pos2());
            self.last = Some(p0[1]);
            self.color = Some(*p2);
        }
    }
    fn draw(&mut self) {
        if let Some(paint) = &self.color {
            self.canvas
                .canvas()
                .draw_path(&self.line, &make_paint(self.width, paint, true, false));
        }
    }
    fn draw_pts(&mut self) {
        if !self.points.is_empty() {
            self.canvas.canvas().draw_points(
                skia_safe::canvas::PointMode::Points,
                &self.points,
                &make_paint(3.0, &self.point_color.unwrap(), false, true),
            );
        }
    }
    pub(crate) fn save(
        &mut self,
        buffer: &mut softbuffer::Buffer<
            std::rc::Rc<winit::window::Window>,
            std::rc::Rc<winit::window::Window>,
        >,
    ) {
        self.draw();
        self.draw_pts();
        if let Some(pm) = self.canvas.peek_pixels() {
            let px = pm.pixels::<u32>().unwrap();
            buffer.copy_from_slice(px)
        }
    }
    pub(crate) fn rect_filled(&mut self, p0: Pos, p2: &Color) {
        if Some(p2) != self.point_color.as_ref() {
            self.draw_pts();
            self.points.clear();
            self.point_color = Some(*p2)
        }
        if self
            .points
            .last()
            .map(|p| !p0.close(Pos::new(p.x, p.y)))
            .unwrap_or(true)
        {
            self.points.push(p0.to_pos2())
        }
    }
    pub(crate) fn image(&mut self, p0: &Image, pos: Vec2) {
        self.canvas.canvas().draw_image(p0, pos.to_pos2(), None);
    }
    pub(crate) fn hline(&mut self, p0: f32, p1: f32, p2: f32, p3: &Color) {
        self.canvas.canvas().draw_line(
            Pos::new(0.0, p1).to_pos2(),
            Pos::new(p0, p1).to_pos2(),
            &make_paint(p2, p3, false, false),
        );
    }
    pub(crate) fn vline(&mut self, p0: f32, p1: f32, p2: f32, p3: &Color) {
        self.canvas.canvas().draw_line(
            Pos::new(p0, 0.0).to_pos2(),
            Pos::new(p0, p1).to_pos2(),
            &make_paint(p2, p3, false, false),
        );
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
