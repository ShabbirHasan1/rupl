use egui::{CentralPanel, Color32, Context, Key, Painter, Pos2, Rect, Stroke, Ui, Vec2};
pub struct Graph {
    data: Vec<f32>,
    offset: Vec2,
    zoom: f32,
}

impl Graph {
    pub fn new(data: Vec<f32>) -> Self {
        let offset = Vec2::splat(0.0);
        let zoom = 1.0;
        Self { data, offset, zoom }
    }
    pub fn update(&mut self, ctx: &Context) {
        CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::from_rgb(255, 255, 255)))
            .show(ctx, |ui| self.plot_main(ctx, ui));
    }
    fn plot_main(&mut self, ctx: &Context, ui: &Ui) {
        let painter = ui.painter();
        let rect = ctx.available_rect();
        let offset = Vec2::new(rect.width() / 2.0, rect.height() / 2.0);
        let (width, height) = (rect.width(), rect.height());
        let n = 16.0;
        self.make_lines(painter, width, height);
        for (i, y) in self.data.iter().enumerate() {
            let x = i as f32 / (self.data.len() - 1) as f32 * n - n / 2.0;
            let pos = Pos2::new(x * width / n, -*y * height / ((n + 1.0) * height / width));
            let rect =
                Rect::from_center_size((pos + self.offset + offset) * self.zoom, Vec2::splat(3.0));
            if ui.is_rect_visible(rect) {
                painter.rect_filled(rect, 0.0, Color32::from_rgb(255, 0, 0));
            }
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
                self.zoom /= 1.5;
                self.offset = (self.offset + offset) * 1.5 - offset;
            }
            if i.key_pressed(Key::E) {
                self.zoom *= 1.5;
                self.offset = (self.offset + offset) / 1.5 - offset;
            }
        });
    }
    fn make_lines(&self, painter: &Painter, width: f32, height: f32) {
        let k = 17.0;
        let n = (17.0 / self.zoom) as usize;
        for i in 0..n {
            let x = i as f32 * width / (k - 1.0);
            painter.line_segment(
                [
                    Pos2::new((x + self.offset.x) * self.zoom, 0.0),
                    Pos2::new((x + self.offset.x) * self.zoom, height),
                ],
                Stroke::new(1.0, Color32::from_rgb(0, 0, 0)),
            );
        }
        let k = k * height / width;
        let n = n as f32 * height / width;
        for i in 0..=n.ceil() as usize {
            let y = (i / 2) as f32 * height / k;
            let y = height / 2.0 + if i % 2 == 0 { y } else { -y };
            painter.line_segment(
                [
                    Pos2::new(0.0, (y + self.offset.y) * self.zoom),
                    Pos2::new(width, (y + self.offset.y) * self.zoom),
                ],
                Stroke::new(1.0, Color32::from_rgb(0, 0, 0)),
            );
        }
    }
}
