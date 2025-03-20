use egui::{CentralPanel, Color32, Context, Painter, Pos2, Rect, Stroke, Ui, Vec2};
pub fn plot(ctx: &Context, data: Vec<f32>) {
    CentralPanel::default()
        .frame(egui::Frame::default().fill(Color32::from_rgb(255, 255, 255)))
        .show(ctx, |ui| plot_main(ctx, ui, data));
}
fn plot_main(ctx: &Context, ui: &Ui, data: Vec<f32>) {
    let painter = ui.painter();
    let rect = ctx.available_rect();
    let (width, height) = (rect.width(), rect.height());
    let offset = Vec2::new(width / 2.0, height / 2.0);
    let n = 16.0;
    for (i, y) in data.iter().enumerate() {
        let x = i as f32 / (data.len() - 1) as f32 * n - n / 2.0;
        let pos = Pos2::new(x * width / n, -*y * height / ((n + 1.0) * height / width));
        painter.rect_filled(
            Rect::from_center_size(pos + offset, Vec2::splat(3.0)),
            0.0,
            Color32::from_rgb(0, 0, 0),
        );
    }
    make_lines(painter, width, height)
}
fn make_lines(painter: &Painter, width: f32, height: f32) {
    let n = 17;
    for i in 0..n {
        let x = i as f32 * width / 16.0;
        painter.line_segment(
            [Pos2::new(x, 0.0), Pos2::new(x, height)],
            Stroke::new(1.0, Color32::from_rgb(0, 0, 0)),
        );
    }
    let n = n as f32 * height / width;
    for i in 0..=n.ceil() as usize {
        let y = (i / 2) as f32 * height / n;
        let y = height / 2.0 + if i % 2 == 0 { y } else { -y };
        painter.line_segment(
            [Pos2::new(0.0, y), Pos2::new(width, y)],
            Stroke::new(1.0, Color32::from_rgb(0, 0, 0)),
        );
    }
}
