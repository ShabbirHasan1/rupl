use egui::Context;
use eplot::Graph;
use std::fs;
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native("eplot", options, Box::new(|_cc| Ok(Box::new(App::new()))))
}

struct App {
    plot: Graph,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.main(ctx);
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
        let plot = Graph::new(data);
        Self { plot }
    }
    fn main(&mut self, ctx: &Context) {
        self.plot.update(ctx)
    }
}
