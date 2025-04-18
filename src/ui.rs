use crate::types::{Align, Color, Pos, Texture, Vec2};
pub(crate) struct Painter<'a> {
    #[cfg(feature = "egui")]
    painter: &'a egui::Painter,
    #[cfg(feature = "skia")]
    painter: &'a f64
}
impl<'a> Painter<'a> {
    #[cfg(feature = "egui")]
    pub(crate) fn new(ui: &'a egui::Ui) -> Self {
        Self {
            painter: ui.painter(),
        }
    }
    pub(crate) fn line_segment(&self, p0: [Pos; 2], p1: f32, p2: &Color) {
        #[cfg(feature = "egui")]{
            self.painter
                .line_segment(p0.map(|p| p.to_pos2()), egui::Stroke::new(p1, p2.to_col()));
        }
    }
    pub(crate) fn rect_filled(&self, p0: Pos, p1: f32, p2: &Color) {
        #[cfg(feature = "egui")] {
        let rect = egui::Rect::from_center_size(p0.to_pos2(), egui::Vec2::splat(3.0));
        self.painter.rect_filled(rect, p1, p2.to_col());}
    }
    pub(crate) fn image(&self, p0: Texture, pos: Vec2) {
    #[cfg(feature = "egui")]
    {
        let d = egui::Rect::from_points(&[egui::Pos2::new(0.0, 0.0), pos.to_pos2()]);
        let a = egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0));
        let c = egui::Color32::WHITE;
        self.painter.image(p0.texture, d, a, c);
    }
    }
    pub(crate) fn hline(&self, p0: f32, p1: f32, p2: f32, p3: &Color) {
    #[cfg(feature = "egui")]{
        self.painter.hline(
            egui::Rangef::new(0.0, p0),
            p1,
            egui::Stroke::new(p2, p3.to_col()),
        );}
    }
    pub(crate) fn vline(&self, p0: f32, p1: f32, p2: f32, p3: &Color) {
    #[cfg(feature = "egui")]{
        self.painter.vline(
            p0,
            egui::Rangef::new(0.0, p1),
            egui::Stroke::new(p2, p3.to_col()),
        );}
    }
    pub(crate) fn text(&self, p0: Pos, p1: Align, p2: String, p4: &Color) {
    #[cfg(feature = "egui")]
    {self.painter.text(
            p0.to_pos2(),
            p1.into(),
            p2,
            egui::FontId::monospace(16.0),
            p4.to_col(),
        );}
    }
}
