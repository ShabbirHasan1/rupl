use std::fs;
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native("eplot", options, Box::new(|_cc| Ok(Box::new(App::new()))))
}

struct App {
    data: Vec<f32>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.main(ctx, self.data.clone());
    }
}

impl App {
    fn new() -> Self {
        let data = fs::read_to_string("data")
            .unwrap()
            .trim()
            .split(',')
            .map(|c| c.parse::<f32>().unwrap())
            .collect::<Vec<f32>>();
        Self { data }
    }
    fn main(&self, ctx: &egui::Context, data: Vec<f32>) {
        eplot::plot(ctx, data)
    }
}
