use egui::{Context, FontData, FontDefinitions, FontFamily};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use rupl::types::{Complex, Graph, GraphType, UpdateResult};
use std::fs;
fn main() -> eframe::Result {
    eframe::run_native(
        "eplot",
        eframe::NativeOptions {
            ..Default::default()
        },
        Box::new(|cc| {
            let mut fonts = FontDefinitions::default();
            fonts.font_data.insert(
                "notosans".to_owned(),
                std::sync::Arc::new(FontData::from_static(include_bytes!("../notosans.ttf"))),
            );
            fonts
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .insert(0, "notosans".to_owned());
            fonts
                .families
                .get_mut(&FontFamily::Monospace)
                .unwrap()
                .insert(0, "notosans".to_owned());
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(App::new()))
        }),
    )
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
        let plot = Graph::new(
            vec![generate_3dc(-2.0, -2.0, 2.0, 2.0, 256)],
            //vec![generate(-2.0, 2.0, 256)],
            true,
            -2.0,
            2.0,
        );
        Self { plot }
    }
    fn main(&mut self, ctx: &Context) {
        match self.plot.update(ctx) {
            UpdateResult::Width(s, e, p) => {
                let plot = generate(s, e, (p * 1024.0) as usize);
                self.plot.set_data(vec![plot]);
            }
            UpdateResult::Width3D(sx, sy, ex, ey, p) => {
                let plot = generate_3dc(sx, sy, ex, ey, (p * 256.0) as usize);
                self.plot.set_data(vec![plot]);
            }
            UpdateResult::None => {}
        }
    }
}
fn to_complex(c: &str) -> Complex {
    if !c.contains('i') {
        Complex::Real(c.parse::<f64>().unwrap_or(0.0))
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
                if c.eq_ignore_ascii_case("in") {
                    f64::INFINITY
                } else if c.eq_ignore_ascii_case("na") {
                    f64::NAN
                } else {
                    c.parse::<f64>().unwrap_or(0.0)
                }
            })
            .collect::<Vec<f64>>();
        let s = if n { -l[0] } else { l[0] };
        if l.len() == 1 {
            Complex::Imag(s)
        } else {
            Complex::Complex(s, if r { -l[1] } else { l[1] })
        }
    }
}
#[allow(dead_code)]
fn grab_width(f: &str, start: f64, end: f64) -> GraphType {
    GraphType::Width(
        fs::read_to_string(f)
            .unwrap()
            .trim()
            .replace(['{', '}'], "")
            .split(',')
            .map(to_complex)
            .collect::<Vec<Complex>>(),
        start,
        end,
    )
}
#[allow(dead_code)]
fn grab_width3d(f: &str, startx: f64, starty: f64, endx: f64, endy: f64) -> GraphType {
    GraphType::Width3D(
        fs::read_to_string(f)
            .unwrap()
            .trim()
            .replace(['{', '}'], "")
            .replace('\n', ",")
            .split(',')
            .map(to_complex)
            .collect::<Vec<Complex>>(),
        startx,
        starty,
        endx,
        endy,
    )
}
#[allow(dead_code)]
fn grab_coord(f: &str) -> GraphType {
    GraphType::Coord(
        fs::read_to_string(f)
            .unwrap()
            .trim()
            .replace(['{', '}'], "")
            .split('\n')
            .map(|c| {
                let a = c.split(',').map(to_complex).collect::<Vec<Complex>>();
                (real(a[0]), a[1])
            })
            .collect::<Vec<(f64, Complex)>>(),
    )
}
#[allow(dead_code)]
fn real(c: Complex) -> f64 {
    match c {
        Complex::Real(y) => y,
        Complex::Imag(_) => 0.0,
        Complex::Complex(y, _) => y,
    }
}
#[allow(dead_code)]
fn generate_3d(startx: f64, starty: f64, endx: f64, endy: f64, len: usize) -> GraphType {
    let mut data = Vec::new();
    for j in 0..=len {
        let j = j as f64 / len as f64;
        for i in 0..=len {
            let i = i as f64 / len as f64;
            let x = startx + i * (endx - startx);
            let y = starty + j * (endy - starty);
            let v = (x.powi(3) + y).exp();
            data.push(Complex::Real(v))
        }
    }
    GraphType::Width3D(data, startx, starty, endx, endy)
}
#[allow(dead_code)]
fn generate_3dc(startx: f64, starty: f64, endx: f64, endy: f64, len: usize) -> GraphType {
    let data = (0..=len)
        .into_par_iter()
        .flat_map(|j| {
            let j = j as f64 / len as f64;
            let y = starty + j * (endy - starty);
            let yr = y * y;
            (0..=len)
                .into_par_iter()
                .map(|i| {
                    let i = i as f64 / len as f64;
                    let x = startx + i * (endx - startx);
                    let re = (x * x + yr).recip();
                    let r = (x * re).sin() * (y * re).cosh();
                    let i = -(x * re).cos() * (y * re).sinh();
                    Complex::Complex(r, i)
                })
                .collect::<Vec<Complex>>()
        })
        .collect::<Vec<Complex>>();
    GraphType::Width3D(data, startx, starty, endx, endy)
}
#[allow(dead_code)]
fn generate(start: f64, end: f64, len: usize) -> GraphType {
    let mut data = Vec::new();
    for i in 0..=len {
        let i = i as f64 / len as f64;
        let x = start + i * (end - start);
        let r = x.cos();
        let i = x.sin();
        data.push(Complex::Complex(r, i))
    }
    GraphType::Width(data, start, end)
}
