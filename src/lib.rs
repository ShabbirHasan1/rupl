use egui::{CentralPanel, Color32, Context, Pos2, Stroke, Ui, Vec2};
pub fn plot(ctx: &Context, data: Vec<f64>) {
    CentralPanel::default()
        .frame(egui::Frame::default().fill(Color32::from_rgb(255, 255, 255)))
        .show(ctx, |ui| plot_main(ctx, ui, data));
}
fn plot_main(ctx: &Context, ui: &Ui, data: Vec<f64>) {
    let painter = ui.painter();
    let rect = ctx.available_rect();
    let (width, height) = (rect.width(), rect.height());
    let offset = Vec2::new(width / 2.0, height / 2.0);
    for (i, y) in data.iter().enumerate() {
        let x = i as f32 / 1000.0 * 16.0 - 8.0;
        let pos = Pos2::new(x * rect.width() / 16.0, -*y as f32 * rect.height() / 2.0);
        painter.circle(
            pos + offset,
            2.0,
            Color32::from_rgb(0, 0, 0),
            Stroke::default(),
        );
    }
    painter.line_segment(
        [Pos2::new(0.0, height / 2.0), Pos2::new(width, height / 2.0)],
        Stroke::new(2.0, Color32::from_rgb(0, 0, 0)),
    );
    painter.line_segment(
        [Pos2::new(width / 2.0, 0.0), Pos2::new(width / 2.0, height)],
        Stroke::new(2.0, Color32::from_rgb(0, 0, 0)),
    );
}
