use egui::{
    Align2, CentralPanel, Color32, Context, FontId, Key, Painter, Pos2, Rect, Sense, Stroke, Ui,
    Vec2,
};
pub struct Graph {
    data: Vec<f32>,
    offset: Vec2,
    zoom: f32,
    width: f32,
}

impl Graph {
    pub fn new(data: Vec<f32>, width: f32) -> Self {
        let offset = Vec2::splat(0.0);
        let zoom = 1.0;
        Self {
            data,
            offset,
            zoom,
            width,
        }
    }
    pub fn update(&mut self, ctx: &Context) {
        CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::from_rgb(255, 255, 255)))
            .show(ctx, |ui| self.plot_main(ctx, ui));
    }
    fn plot_main(&mut self, ctx: &Context, ui: &Ui) {
        let painter = ui.painter();
        let rect = ctx.available_rect();
        let (width, height) = (rect.width(), rect.height());
        let offset = Vec2::new(width / 2.0, height / 2.0);
        self.keybinds(ui, offset);
        self.plot(painter, width, height, offset, ui);
        self.make_lines(painter, width, height);
    }
    fn plot(&self, painter: &Painter, width: f32, height: f32, offset: Vec2, ui: &Ui) {
        for (i, y) in self.data.iter().enumerate() {
            let x = i as f32 / (self.data.len() - 1) as f32 * self.width - self.width / 2.0;
            let pos = Pos2::new(
                x * width / self.width,
                -*y * height / (self.width * height / width),
            );
            let rect =
                Rect::from_center_size((pos + self.offset + offset) * self.zoom, Vec2::splat(3.0));
            if ui.is_rect_visible(rect) {
                painter.rect_filled(rect, 0.0, Color32::from_rgb(255, 0, 0));
            }
        }
    }
    fn make_lines(&self, painter: &Painter, width: f32, height: f32) {
        let ni = (self.width / self.zoom) as isize;
        let n = ni.max(1);
        let delta = width / self.width;
        let s = (-self.offset.x / delta).ceil() as isize;
        let ny = (n as f32 * height / width).ceil() as isize;
        let offset = self.offset.y + height / 2.0 - (ny as f32 / 2.0).floor() * delta;
        let sy = (-offset / delta).ceil() as isize;
        for i in s..s + n {
            let x = i as f32 * delta;
            painter.line_segment(
                [
                    Pos2::new((x + self.offset.x) * self.zoom, 0.0),
                    Pos2::new((x + self.offset.x) * self.zoom, height),
                ],
                Stroke::new(1.0, Color32::from_rgb(0, 0, 0)),
            );
        }
        if ni == n {
            let i = (n as f32 / 2.0 * self.zoom) as isize;
            let x = if (s..=s + n).contains(&i) {
                i as f32 * delta
            } else {
                -self.offset.x / self.zoom
            };
            for j in sy..sy + ny {
                let y = j as f32 * delta;
                painter.text(
                    Pos2::new(x + self.offset.x, y + offset) * self.zoom,
                    Align2::LEFT_TOP,
                    (ny / 2 - j).to_string(),
                    FontId::monospace(16.0),
                    Color32::from_rgb(0, 0, 0),
                );
            }
        }
        for i in sy..sy + ny {
            let y = i as f32 * delta;
            painter.line_segment(
                [
                    Pos2::new(0.0, (y + offset) * self.zoom),
                    Pos2::new(width, (y + offset) * self.zoom),
                ],
                Stroke::new(1.0, Color32::from_rgb(0, 0, 0)),
            );
        }
        if ni == n {
            let i = ny / 2;
            let y = if (sy..=sy + ny).contains(&i) {
                i as f32 * delta
            } else {
                -offset / self.zoom
            };
            for j in s..s + n {
                let x = j as f32 * delta;
                painter.text(
                    Pos2::new(x + self.offset.x, y + offset) * self.zoom,
                    Align2::LEFT_TOP,
                    (j - (n as f32 / 2.0 * self.zoom) as isize).to_string(),
                    FontId::monospace(16.0),
                    Color32::from_rgb(0, 0, 0),
                );
            }
        }
    }
    fn keybinds(&mut self, ui: &Ui, offset: Vec2) {
        let response = ui.interact(
            ui.available_rect_before_wrap(),
            ui.id().with("map_interact"),
            Sense::drag(),
        );
        if response.dragged() {
            self.offset += response.drag_delta() / self.zoom;
        }
        ui.input(|i| {
            if i.key_pressed(Key::A) || i.key_pressed(Key::ArrowLeft) {
                self.offset.x += 64.0 / self.zoom
            }
            if i.key_pressed(Key::D) || i.key_pressed(Key::ArrowRight) {
                self.offset.x -= 64.0 / self.zoom
            }
            if i.key_pressed(Key::W) || i.key_pressed(Key::ArrowUp) {
                self.offset.y += 64.0 / self.zoom
            }
            if i.key_pressed(Key::S) || i.key_pressed(Key::ArrowDown) {
                self.offset.y -= 64.0 / self.zoom
            }
            if i.key_pressed(Key::Q) {
                self.zoom /= 2.0;
                self.offset = (self.offset + offset) * 2.0 - offset;
            }
            if i.key_pressed(Key::E) {
                self.zoom *= 2.0;
                self.offset = (self.offset + offset) / 2.0 - offset;
            }
            if i.key_pressed(Key::T) {
                self.offset = Vec2::splat(0.0);
                self.zoom = 1.0;
            }
        });
    }
}
