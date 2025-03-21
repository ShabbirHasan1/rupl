use egui::Context;
use eplot::{Complex, Graph};
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
        let plot = Graph::new(vec![grab("data4"), grab("data5"), grab("data6")], 8.0);
        Self { plot }
    }
    fn main(&mut self, ctx: &Context) {
        self.plot.update(ctx);
    }
}
fn grab(f: &str) -> Vec<Complex> {
    fs::read_to_string(f)
        .unwrap()
        .trim()
        .split(',')
        .map(|c| {
            if !c.contains('i') {
                Complex::Real(c.parse::<f32>().unwrap())
            } else {
                let n = c.starts_with('-');
                let c = if n {
                    &c.chars().skip(1).take(c.len() - 2).collect::<String>()
                } else {
                    &c.chars().take(c.len() - 1).collect::<String>()
                };
                let r = c.contains('-');
                let l = c
                    .split(['-', '+'])
                    .map(|c| {
                        if c.to_ascii_lowercase() == "in" {
                            f32::INFINITY
                        } else if c.to_ascii_uppercase() == "na" {
                            f32::NAN
                        } else {
                            c.parse::<f32>().unwrap()
                        }
                    })
                    .collect::<Vec<f32>>();
                let s = if n { -l[0] } else { l[0] };
                if l.len() == 1 {
                    Complex::Imag(s)
                } else {
                    Complex::Complex(s, if r { -l[1] } else { l[1] })
                }
            }
        })
        .collect::<Vec<Complex>>()
}
